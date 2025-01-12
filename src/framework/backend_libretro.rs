use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::c_void;
use std::io::Read;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;
use num_traits::Num;

// //new libretro stuff (copied from example)
// use libretro_rs::c_utf8::c_utf8;
// use libretro_rs::retro::env::{Init, UnloadGame};
// use libretro_rs::retro::pixel::{Format, XRGB8888};
// //log conflicts, we need to explicitly include everything
// //use libretro_rs::retro::*;
// use libretro_rs::retro::{av, cores, device, env, error, fs, game, log as retro_log, mem as retro_mem, str};
// use libretro_rs::retro::av::*;
// use libretro_rs::retro::cores::*;
// use libretro_rs::{ext, libretro_core};

use imgui::{DrawData, TextureId, Ui};

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendEventLoop, BackendRenderer, BackendGamepad, BackendShader, BackendTexture, SpriteBatchCommand, VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::{GameResult, GameError};
use crate::framework::gamepad::GamepadType;
use crate::framework::graphics::BlendMode;
use crate::input::touch_controls::TouchPoint;


//gl stuff
use crate::framework::render_opengl::{GLContext, OpenGLRenderer, GlVersionInfo};
use crate::framework::gl;

use crate::game::shared_game_state::SharedGameState;
use crate::game::Game;

use super::keyboard::ScanCode;
use super::gamepad::{Button, Axis};

#[derive(PartialEq)]
pub enum RenderMode {
    None,
    OpenGL(u32, u32),
    OpenGLES,
    Software,
}

pub struct LibretroBackend;

impl LibretroBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(LibretroBackend))
    }
    //special initializers without dynamic traits
    pub fn new_nd() -> GameResult<Box<LibretroBackend>> {
        Ok(Box::new(LibretroBackend))
    }

    pub fn create_event_loop_nd(&self, _ctx: &Context,
        get_current_framebuffer: fn() -> usize,
        get_proc_address: fn(&str) -> *const c_void,
        render_mode: RenderMode,
    ) -> GameResult<Box<LibretroEventLoop>> {
        Ok(LibretroEventLoop::new(get_current_framebuffer, get_proc_address, render_mode).unwrap())
    }

}

impl Backend for LibretroBackend {
    fn create_event_loop(&self, _ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        Err(GameError::CommandLineError(("This function should not be called with this backend!".to_owned())))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}


pub struct LibretroEventLoop {
    refs: Rc<RefCell<LibretroContext>>,
    render_mode: RenderMode,
    touchpad_context: LibretroTouchpad,
}

//holds things like openGL renderer, keystrokes, and audio? (maybe?)
//is basically a datapack struct to feed info to various functions in the form of a void()
struct LibretroContext {
    get_current_framebuffer: fn() -> usize,
    get_proc_address: fn(&str) -> *const c_void,
}

impl LibretroEventLoop {

    pub fn new(
        get_current_framebuffer: fn() -> usize,
        get_proc_address: fn(&str) -> *const c_void,
        render_mode: RenderMode,
    ) -> GameResult<Box<LibretroEventLoop>>
    {
        let event_loop = LibretroEventLoop {
            refs: Rc::new(RefCell::new(LibretroContext{
                get_current_framebuffer,
                get_proc_address,
            })),
            render_mode: render_mode,
            touchpad_context: LibretroTouchpad::new(10),
        };

        Ok(Box::new(event_loop))
    }


    //destroy the context's renderer (because the frontend's environment has changed)
    pub fn destroy_renderer(&self, state_ref: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        ctx.renderer = None;

        //wipe all old textures
        state_ref.texture_set.unload_all();

        Ok(())
    }

    //called on init and whenever the frontend's environment has changed (immediately after destroy_renderer)
    pub fn rebuild_renderer(&self, state_ref: &mut SharedGameState, ctx: &mut Context, width: u32, height: u32) -> GameResult {
        ctx.renderer = Some(self.new_renderer(ctx)?);
        self.handle_resize(state_ref, ctx, width, height)
    }

    pub fn handle_resize(&self, state_ref: &mut SharedGameState, ctx: &mut Context, width: u32, height: u32) -> GameResult {
        ctx.screen_size = (width as f32, height as f32);
        
        if let Some(renderer) = &ctx.renderer {
            if let Ok(imgui) = renderer.imgui() {
                imgui.io_mut().display_size = [ctx.screen_size.0, ctx.screen_size.1];
            }
        }
        state_ref.handle_resize(ctx);

        Ok(())
    }


    //like run(), but called repeatedly
    pub fn update(&mut self, state_ref: &mut SharedGameState, game: &mut Game, ctx: &mut Context, micros: u64)
    {
        //let state_ref = unsafe { &mut *game.state.get() };

        //tick the gamepads' rumble timers
        let gamepad_count = {ctx.gamepad_context.get_gamepads().len()};
        for gamepad_index in 0..gamepad_count
        {
            if let Some(gamepad) = ctx.gamepad_context.get_gamepad_by_index_mut(gamepad_index) {
                if let Some(libretro_gp_context) = gamepad.controller
                    .as_any_mut()
                    .downcast_mut::<LibretroGamepad>() {
                
                    libretro_gp_context.tick(micros);
                }
            }
        }


        game.update(ctx, micros).unwrap();

        if state_ref.shutdown {
            log::info!("Shutting down...");
            //TODO: tell core to halt execution

            return;
        }

        if state_ref.next_scene.is_some() {
            mem::swap(&mut game.scene, &mut state_ref.next_scene);
            state_ref.next_scene = None;
            game.scene.as_mut().unwrap().init(state_ref, ctx).unwrap();
            game.loops = 0;
            state_ref.frame_time = 0.0;
        }
        //std::thread::sleep(std::time::Duration::from_millis(10));

        match game.draw(ctx)
        {
            Ok(_)=>{},
            Err(e)=>{log::error!("{}", e)}
        }


    }

    //takes input from libretro callbacks and pushes it into the engine
    pub fn update_keys(&mut self, ctx: &mut Context, key_id: ScanCode, key_state: bool)
    {
        ctx.keyboard_context.set_key(key_id, key_state);
    } 
    pub fn update_gamepad_key(&mut self, ctx: &mut Context, id: u16, button_id: Button, button_state: bool)
    {
        ctx.gamepad_context.set_button(id as u32, button_id, button_state);
    }
    pub fn update_gamepad_axis(&mut self, ctx: &mut Context, id: u16, axis_id: Axis, value: i16)
    {
        let new_value = (value as f64) / i16::MAX as f64; //normalize axis input
        ctx.gamepad_context.set_axis_value(id as u32, axis_id, new_value);
        ctx.gamepad_context.update_axes(id as u32);
    }

    pub fn add_gamepad(&mut self,
        state_ref: &mut SharedGameState,
        ctx: &mut Context,
        id: u32,
        rumble_fn: Option<fn (controller_port: u32, effect: u16, strengh: u16) -> bool>,
        ) {
        log::info!("Connected gamepad: {} (ID: {})", "Retropad", id);

        let axis_sensitivity = state_ref.settings.get_gamepad_axis_sensitivity(id);
        ctx.gamepad_context.add_gamepad(LibretroGamepad::new(id, rumble_fn), axis_sensitivity);
        ctx.gamepad_context.set_gamepad_type(id, GamepadType::Retropad);
    }

    pub fn remove_gamepad(&mut self, ctx: &mut Context, id: u16) {
        ctx.gamepad_context.remove_gamepad(id as u32);
    }

    pub fn update_touchpad(&mut self, x: i16, y: i16, id: u16) {
        self.touchpad_context.set_point(x, y, id);
    }
    pub fn finalize_touchpad(&mut self, state_ref: &mut SharedGameState) {
        self.touchpad_context.finalize_points();

        self.touchpad_context.push_points(state_ref);
    }


}

//not really used, since there are many special functions inside the libretroEventLoop
impl BackendEventLoop for LibretroEventLoop {

    //run is unused. See update() instead
    fn run(&mut self, _game: &mut Game, _ctx: &mut Context) { }

    //initialize the renderers for imgui and main
    fn new_renderer(&self, ctx: *mut Context) -> GameResult<Box<dyn BackendRenderer>> {


        let mut imgui = imgui::Context::create();
        imgui.io_mut().display_size = [640.0, 480.0];
        imgui.fonts().build_alpha8_texture();


        //test
        //let mut benders_shiny_metal_ass = (self.refs.borrow().get_current_framebuffer)();
        //let frys_face = benders_shiny_metal_ass + 1;
        //return Ok(Box::new(LibretroRenderer(RefCell::new(imgui))));

        //turn refs into a raw pointer
        let refs = self.refs.clone();
        let user_data = Rc::into_raw(refs) as *mut c_void;

        //load example:
        //let gl = gl::Gles2::load_with(|ptr| (gl_context.get_proc_address)(&mut gl_context.user_data, ptr));


        //function to use in order to refresh the buffer

        //these are responsible for turning a data dump over user_data into addresses avalable to the backend
        unsafe fn get_proc_address(user_data: &mut *mut c_void, name: &str) -> *const c_void {
            //pull a struct out of user_data pointer
            let refs = Rc::from_raw(*user_data as *mut RefCell<LibretroContext>);

            let result = {
                let refs = &mut *refs.as_ptr();//*refs.get();

                (refs.get_proc_address)(name)
            };
            *user_data = Rc::into_raw(refs) as *mut c_void;


            //return result
            result
        }

        unsafe fn swap_buffers(_user_data: &mut *mut c_void) {
            //libretro doesn't use this: do nothing
        }

        unsafe fn get_current_buffer(user_data: &mut *mut c_void) -> usize {
            let refs = Rc::from_raw(*user_data as *mut RefCell<LibretroContext>);

            let cur_fb: usize;
            {
                let refs = &mut *refs.as_ptr();//*refs.get();

                cur_fb = (refs.get_current_framebuffer)()
            }

            *user_data = Rc::into_raw(refs) as *mut c_void;
            cur_fb
        }


        let gl_version = match &self.render_mode {
            RenderMode::OpenGL(maj, min) => GlVersionInfo::OpenGL(*maj, *min),
            RenderMode::OpenGLES => GlVersionInfo::OpenGLES,
            _ => GlVersionInfo::OpenGL(0,0), // this case should never be reached
        };

        let gl_context = GLContext { gl_version, is_sdl: false, get_proc_address, swap_buffers, get_current_buffer, user_data, ctx };
        //let gl_context = GLContext { gles2_mode: (self.render_mode == RenderMode::OpenGlES), is_sdl: false, get_proc_address, swap_buffers, get_current_buffer, user_data, ctx };

        //Err(super::error::GameError::CommandLineError(("Not Done Yet!".to_owned())))//=>{log::error!("not done yet!")}
        Ok(Box::new(OpenGLRenderer::new(gl_context, UnsafeCell::new(imgui))))

    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}


struct LibretroGamepad {
    id: u32,
    rumble_fn: Option<fn (controller_port: u32, effect: u16, strengh: u16) -> bool>,

    // used to convert duration into on/off commands since that's how retroarch works
    rumble_on: bool,
    duration_us: u64,
}

impl LibretroGamepad {
    pub fn new(id: u32, rumble_fn: Option<fn (_: u32, _: u16, _: u16) -> bool>) -> Box<dyn BackendGamepad> {
        Box::new(LibretroGamepad {
            id,
            rumble_fn,
            rumble_on: false,
            duration_us: 0,
        })
    }

    //micros (us)
    pub fn tick(&mut self, delta_time: u64) {

        // halt rumble once it times out
        if self.duration_us == 0 {
            // condition to avoid constant calls to the rumble context (works without this measure, but may slow the game down)
            if self.rumble_on {
                self.rumble_on = false;
                self.set_rumble(0, 0, 0);
            }
        } else {
            self.duration_us = self.duration_us.saturating_sub(delta_time);
        }
    }

}

impl BackendGamepad for LibretroGamepad {

    fn set_rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) -> GameResult {
        
        if let Some(rumble_fn) = self.rumble_fn{
            let _ = rumble_fn(self.id as u32, 0, low_freq); // set low freq. rumble speed
            let _ = rumble_fn(self.id as u32, 1, high_freq); // set high freq. rumble speed
        }
        self.duration_us = duration_ms as u64 * 1000; // change units
        self.rumble_on = (low_freq > 0 || high_freq > 0);

        Ok(())
    }

    fn instance_id(&self) -> u32 {
        self.id
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

}


//used to implement some important methods (like touchdown, moved, and touchup) that libretro doesn't do
#[derive(Clone, PartialEq)]
enum TouchpointState {
    Started,
    Moved,
    Ended,
}
#[derive(Clone)]
struct LibretroTouchpoint {
    x: f32,
    y: f32,
    id: i32,
    state: TouchpointState,
    updated: bool,
}
impl LibretroTouchpoint {
    pub fn new() -> LibretroTouchpoint {
        LibretroTouchpoint {
            x: 0.0,
            y: 0.0,
            id: -1, //-1 if slot is empty
            state: TouchpointState::Ended,
            updated: false,


        }
    }
}
struct LibretroTouchpad {
    touchpoints: Vec<LibretroTouchpoint>,
    last_touchpoints: Vec<LibretroTouchpoint>,

    point_count: u16,

    last_point_count: u16,

}

impl LibretroTouchpad {
    pub fn new(max_points: u16) -> LibretroTouchpad{
        LibretroTouchpad {
            touchpoints: vec![LibretroTouchpoint::new(); max_points as usize],
            last_touchpoints: vec![LibretroTouchpoint::new(); max_points as usize],

            point_count: 0,

            last_point_count: 0,

        }
    }

    pub fn set_point(&mut self, x: i16, y: i16, idx: u16) {


        //check though used ids list to see if that id is active (what index?)
        //if not active, find the first '-1' index and activate it with this id
        //if yes active, update state and location (normalize it)

        //let touchpoints = if self.active_tp {&mut self.touchpoint_set_0} else {&mut self.touchpoint_set_1};

        let idx = idx as usize;

        if idx < self.touchpoints.len() {
            
            //new touchpoint, register an unused ID
            if self.touchpoints[idx].id == -1 {

                let mut new_id = 0;
                //iterate through touchpoints to see what they have
                for mut odx in 0..self.touchpoints.len() {
                    //found a match, can't use, so try the next ID
                    if new_id == self.touchpoints[odx].id {
                        new_id += 1;
                        odx = 0;
                    }
                }
                self.touchpoints[idx].id = new_id;
                self.touchpoints[idx].state = TouchpointState::Started;

            } else {
                self.touchpoints[idx].state = TouchpointState::Moved;
            }

            self.touchpoints[idx].x = x as f32;
            self.touchpoints[idx].y = y as f32;
            self.touchpoints[idx].updated = true;

            self.point_count += 1;
        }


    }

    pub fn finalize_points(&mut self){


        //iterate through point list and knock off any that are "ended"
        for curr_tp in self.touchpoints.iter_mut() {
            if curr_tp.state == TouchpointState::Ended && curr_tp.id != -1 {
                curr_tp.id = -1;
            }
        }

        //see if current_point_count matches last_point count
        //if it is less, find the point that was lifted and set its state to "ended", to be knocked off next cycle
        
        //note: point IDs are not static! each time a point is lifted, we play a round of musical chairs with IDs
        if self.point_count < self.last_point_count {

            //find the closest point to each new point.
            //when all are found, there will be an index in the old points that is refrenced by none

            //for each valid ID in our new pointset, give it the ID of the closest old point
            for curr_tp in self.touchpoints.iter_mut() {

                curr_tp.id = -1; //since indexes changed, current IDs don't mean anything anymore
                if !curr_tp.updated {continue;}

                curr_tp.updated = false; //reset these while we're here

                let mut winner_dist: f32 = f32::MAX;
                for last_tp in self.last_touchpoints.iter_mut() {
                    if last_tp.id == -1 {continue;}
                    let dist = Self::get_dist(curr_tp.x, curr_tp.y, last_tp.x, last_tp.y);
                    //update winner credentials
                    if dist < winner_dist {
                        curr_tp.id = last_tp.id;
                        winner_dist = dist;

                        last_tp.updated = true; //re-using this variable to determine which old points were refrenced
                    }
                }
            }

            //find the old point refrenced by none
            for (idx, last_tp) in self.last_touchpoints.iter_mut().enumerate() {
                if last_tp.updated || last_tp.id == -1 {continue;}

                // let mut has_match = false;
                // for curr_tp in self.touchpoints.iter_mut() {
                //     if last_tp.id == curr_tp.id {
                //         has_match = true;
                //         break;
                //     } //ignore IDs that match
                // }
                // if has_match {continue;}

                //point matched by none, set current state to ended
                //this point will be knocked off next cycle
                //self.touchpoints[idx].state = TouchpointState::Ended;
                
                //put that point at the end of the list
                for curr_tp in self.touchpoints.iter_mut() {
                    if curr_tp.id == -1 {
                        (curr_tp.id, curr_tp.x, curr_tp.y) = (last_tp.id, last_tp.x, last_tp.y);
                        curr_tp.state = TouchpointState::Ended;
                        break;
                    }
                }


            }


        } else {
            //reset update value
            for curr_tp in self.touchpoints.iter_mut() {curr_tp.updated = false;}
        }

        //update old touchpoints (clone_from is better than clone here since it shouldn't allocate a third vector)
        self.last_touchpoints.clone_from(&self.touchpoints);

        self.last_point_count = self.point_count;
        self.point_count = 0;

        self.print_clicked();
    }

    fn get_dist(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        ((x2 - x1).powf(2.0) + (y2 - y1).powf(2.0)).sqrt()
    }

    fn print_clicked(&self) {

        for curr_tp in self.touchpoints.iter() {
            if curr_tp.state == TouchpointState::Started {
                log::info!("Screen Clicked at {} - {}:{}", curr_tp.id, curr_tp.x, curr_tp.y);
            }
            if curr_tp.state == TouchpointState::Ended && curr_tp.id != -1 {
                log::info!("Screen Released at {} - {}:{}", curr_tp.id, curr_tp.x, curr_tp.y);
            }

        }

    }

    #[inline]
    fn map<T: Num + PartialOrd + Copy>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T {
        (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
    }

    pub fn push_points(&self, state_ref: &mut SharedGameState) {
        
        let mut controls = &mut state_ref.touch_controls;

        for touchpoint in &self.touchpoints {
            if touchpoint.id != -1 {

                let loc_x = Self::map(touchpoint.x as f32, i16::MIN as f32, i16::MAX as f32, 0.0, state_ref.canvas_size.0) as f64;
                let loc_y = Self::map(touchpoint.x as f32, i16::MIN as f32, i16::MAX as f32, 0.0, state_ref.canvas_size.1) as f64;
                match touchpoint.state {
                    TouchpointState::Started |
                    TouchpointState::Moved => {

                        if let Some(point) = controls.points.iter_mut().find(|p| p.id == touchpoint.id as u64) {
                            point.last_position = point.position;
                            point.position = (loc_x, loc_y);
                        } else {
                            controls.touch_id_counter = controls.touch_id_counter.wrapping_add(1);

                            let point = TouchPoint {
                                id: touchpoint.id as u64,
                                touch_id: controls.touch_id_counter,
                                position: (loc_x, loc_y),
                                last_position: (0.0, 0.0),
                            };
                            controls.points.push(point);

                            if touchpoint.state == TouchpointState::Started {
                                controls.clicks.push(point);
                            }
                        }

                    },
                    TouchpointState::Ended => {
                        controls.points.retain(|p| p.id != touchpoint.id as u64);
                        controls.clicks.retain(|p| p.id != touchpoint.id as u64);
                    },
                }
            }
        }
    }


}


//todo: fallback software renderer (not opengl)
//actually puts the stuff onto the screen, 
//render_opengl creates the textures beforehand
pub struct LibretroTexture(u16, u16);

impl BackendTexture for LibretroTexture {

    //get dimensions of texture
    fn dimensions(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    //add a set of rects to be rendered?
    fn add(&mut self, _command: SpriteBatchCommand) {

        let (tex_scale_x, tex_scale_y) = (1.0 / self.0 as f32, 1.0 / self.1 as f32);

    }

    fn clear(&mut self) {}

    fn draw(&mut self) -> GameResult<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct LibretroRenderer(RefCell<imgui::Context>);


impl BackendRenderer for LibretroRenderer {
    fn renderer_name(&self) -> String {
        "Retroarch".to_owned()
    }

    fn clear(&mut self, _color: Color) {



    }

    fn present(&mut self) -> GameResult {
        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(LibretroTexture(width, height)))
    }

    fn create_texture(&mut self, width: u16, height: u16, _data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(LibretroTexture(width, height)))
    }

    fn set_blend_mode(&mut self, _blend: BlendMode) -> GameResult {
        Ok(())
    }

    fn set_render_target(&mut self, _texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        Ok(())
    }

    fn draw_rect(&mut self, _rect: Rect<isize>, _color: Color) -> GameResult {
        Ok(())
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn set_clip_rect(&mut self, _rect: Option<Rect>) -> GameResult {
        Ok(())
    }

    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.0.as_ptr()) }
    }

    fn imgui_texture_id(&self, _texture: &Box<dyn BackendTexture>) -> GameResult<TextureId> {
        Ok(TextureId::from(0))
    }

    fn prepare_imgui(&mut self, _ui: &Ui) -> GameResult {
        Ok(())
    }

    fn render_imgui(&mut self, _draw_data: &DrawData) -> GameResult {
        Ok(())
    }

    fn draw_triangle_list(
        &mut self,
        _vertices: &[VertexData],
        _texture: Option<&Box<dyn BackendTexture>>,
        _shader: BackendShader,
    ) -> GameResult<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
