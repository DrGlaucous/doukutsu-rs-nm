/*
use std::any::Any;
use std::borrow::BorrowMut;
//use std::cell::RefCell;
use std::cell::{RefCell, UnsafeCell, RefMut};
use std::mem;
use std::vec::Vec;
use std::rc::Rc;

//use rust_wasm_graphics_lib::canvas::{self, Canvas};
//use rust_wasm_graphics_lib::drawing;
//use rust_wasm_graphics_lib::types::{self, ARGBColour, UVWrapMode};

use imgui::{DrawData, TextureId, Ui};
use image::{DynamicImage, ImageBuffer, Pixel, RgbaImage, RgbImage, Rgba};



use crate::framework::ui::init_imgui;

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendEventLoop, BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::{GameResult, GameError};
use crate::framework::graphics::BlendMode;

use crate::game::Game;

//use SDL context (easier)
#[cfg(feature = "backend-sdl")]
use crate::framework::backend_sdl2::SDL2Context;
#[cfg(feature = "backend-sdl")]
use sdl2::render::Texture;
#[cfg(feature = "backend-sdl")]
use sdl2::pixels::PixelFormatEnum;


//use openGL context (harder)
#[cfg(any(feature = "backend-glutin", feature = "backend-horizon"))]
use crate::framework::render_opengl::{GLContext, OpenGLRenderer};
*/

///////////////////////////////////////////////////////////////////


use core::mem;
use std::any::Any;
//use std::borrow::Borrow;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::c_void;
use std::io::Read;
use std::ops::{Deref, DerefMut, Add, Sub, Mul, Div};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::vec::Vec;
use num_traits::Num;


use imageproc::drawing::Blend;
use imgui::internal::RawWrapper;
use imgui::sys::{ImGuiKey_Backspace, ImGuiKey_Delete, ImGuiKey_Enter};
use imgui::{ConfigFlags, DrawCmd, DrawData, DrawIdx, DrawVert, Key, MouseCursor, TextureId, Ui};
use sdl2::controller::GameController;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::mouse::{Cursor, SystemCursor};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, TextureQuery, WindowCanvas};
use sdl2::rwops::RWops;
use sdl2::surface::Surface;
use sdl2::video::GLProfile;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::video::WindowSurfaceRef;
use sdl2::{controller, keyboard, pixels, EventPump, GameControllerSubsystem, Sdl, VideoSubsystem};


use rust_wasm_graphics_lib::canvas::{self, Canvas};
use rust_wasm_graphics_lib::drawing;
use rust_wasm_graphics_lib::types::{self, ARGBColour, UVVertex, UVWrapMode};
use winapi::shared::winerror::SPAPI_E_CANT_REMOVE_DEVINST;

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendEventLoop, BackendGamepad, BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand,
    VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::filesystem;
use crate::framework::gamepad::{Axis, Button, GamepadType};
use crate::framework::graphics::BlendMode;
use crate::framework::keyboard::ScanCode;
#[cfg(feature = "render-opengl")]
use crate::framework::render_opengl::{GLContext, OpenGLRenderer};
use crate::framework::ui::init_imgui;
use crate::game::shared_game_state::WindowMode;
use crate::game::Game;
use crate::game::GAME_SUSPENDED;

use crate::framework::backend_sdl2::SDL2Context;
///////////////////////////////////////////////////////////////////

//using these instead of the Canvas library's built-in blit functions because we don't need the extra features and we need different blend modes
fn trim_oob_rects(
    rect_src: &Rect<isize>,
    dest_width: isize,
    dest_height: isize,
    x: isize,
    y: isize,
) -> Option<(Rect<isize>, isize, isize)> {

    //handle OOB cases
    if x >= dest_width as isize //left corner is beyond canvas' right side
    || y >= dest_height as isize //bottom
    || (x + rect_src.width() as isize) < 0 //right corner is beyond canvas' left side
    || (y + rect_src.height() as isize) < 0 //top
    || rect_src.width() == 0 //source rect is 0
    || rect_src.height() == 0
    {
        return None;
    }

    //do this so we can edit these values
    let mut rect_src = rect_src.clone();
    let (mut x, mut y) = (x, y);

    //OOB trimming

    //left/top trimming
    if x < 0 {
        rect_src.left -= x;
        x -= x;
    }
    if y < 0 {
        rect_src.top -= y;
        y -= y;
    }

    //right/bottom trimming
    if x + rect_src.width() > dest_width as isize {
        let diff = x + rect_src.width() - dest_width as isize;
        rect_src.right -= diff;
    }
    if y + rect_src.height() > dest_height as isize {
        let diff = y + rect_src.height() - dest_height as isize;
        rect_src.bottom -= diff;
    }

    //rect is now guaranteed to be between [0-dest_rect_width/height]
    Some((rect_src, x, y))


}


fn blend_pixel (src_val: u32, des_val: u32, blend_mode: BlendMode, color: (u8, u8, u8, u8)) -> u32 {

    //color format: RGBA
    //u32 color format: ARGB

    match blend_mode {
        BlendMode::None => {
            return src_val;
        },
        BlendMode::Add => {

            let val4 = ((des_val >> (8 * 3)) & 0xFF) * color.3 as u32 / 0xFF;
            
            let val3 = (((des_val >> (8 * 2)) as u8).saturating_add(((src_val >> (8 * 2)) as u8)) as u32) * color.0 as u32 * 0xFF as u32 / 0xFE01;
            let val2 = (((des_val >> (8 * 1)) as u8).saturating_add(((src_val >> (8 * 1)) as u8)) as u32) * color.1 as u32 * 0xFF as u32 / 0xFE01;
            let val1 = (((des_val >> (8 * 0)) as u8).saturating_add(((src_val >> (8 * 0)) as u8)) as u32) * color.2 as u32 * 0xFF as u32 / 0xFE01;

            return (val4 << 8*3) | (val3 << 8*2) | (val2 << 8*1) | (val1 << 8*0);
        },
        BlendMode::Alpha => {
            //sdl blend
            //let mut des_val = canv_dst.buffer_mut()[d_idx];
            //let mut src_val = sour_buffer[s_idx];


            //[3][2][1][0]
            //[A][R][G][B]

            //[Rs]*[As]/0xFF+[Rd]*0xFF/[As]

            // //FLOATING POINT OPERATIONS (known to work)
            // let mut f_src_a = ((src_val >> (8 * 3)) & 0xFF) as f32 / 255.0; 
            // let mut f_src_r = ((src_val >> (8 * 2)) & 0xFF) as f32 / 255.0; 
            // let mut f_src_g = ((src_val >> (8 * 1)) & 0xFF) as f32 / 255.0; 
            // let mut f_src_b = ((src_val >> (8 * 0)) & 0xFF) as f32 / 255.0; 

            // let mut f_dst_a = ((des_val >> (8 * 3)) & 0xFF) as f32 / 255.0; 
            // let mut f_dst_r = ((des_val >> (8 * 2)) & 0xFF) as f32 / 255.0; 
            // let mut f_dst_g = ((des_val >> (8 * 1)) & 0xFF) as f32 / 255.0; 
            // let mut f_dst_b = ((des_val >> (8 * 0)) & 0xFF) as f32 / 255.0; 

            // let mut val4f = f_src_a + (f_dst_a * (1.0 - f_src_a));
            // let mut val3f = (f_src_r * f_src_a) + (f_dst_r * (1.0 - f_src_a));
            // let mut val2f = (f_src_g * f_src_a) + (f_dst_g * (1.0 - f_src_a));
            // let mut val1f = (f_src_b * f_src_a) + (f_dst_b * (1.0 - f_src_a));

            // let mut val4 = ((val4f * 255.0) * color.a) as u32;
            // let mut val3 = ((val3f * 255.0) * color.r * color.a) as u32;
            // let mut val2 = ((val2f * 255.0) * color.g * color.a) as u32;
            // let mut val1 = ((val1f * 255.0) * color.b * color.a) as u32;

            // let mut val42 = val4 as f32;
            // let mut val32 = val3 as f32;
            // let mut val22 = val2 as f32;
            // let mut val12 = val1 as f32;

            // return (val4 << 8*3) | (val3 << 8*2) | (val2 << 8*1) | (val1 << 8*0);
            // //END


            // //try the same thing with only integer math
            let src_a = ((src_val >> (8 * 3)) & 0xFF); 
            let src_r = ((src_val >> (8 * 2)) & 0xFF); 
            let src_g = ((src_val >> (8 * 1)) & 0xFF); 
            let src_b = ((src_val >> (8 * 0)) & 0xFF); 

            let dst_a = ((des_val >> (8 * 3)) & 0xFF); 
            let dst_r = ((des_val >> (8 * 2)) & 0xFF); 
            let dst_g = ((des_val >> (8 * 1)) & 0xFF); 
            let dst_b = ((des_val >> (8 * 0)) & 0xFF); 

            let val4u = (src_a + (dst_a * (0xFF - src_a) / 0xFF )) * color.3 as u32 / 0xFF;
            let val3u = ((src_r * src_a / 0xFF) + (dst_r * (0xFF - src_a) / 0xFF)) * color.0 as u32 * color.3 as u32 / 0xFE01;
            let val2u = ((src_g * src_a / 0xFF) + (dst_g * (0xFF - src_a) / 0xFF)) * color.1 as u32 * color.3 as u32 / 0xFE01;
            let val1u = ((src_b * src_a / 0xFF) + (dst_b * (0xFF - src_a) / 0xFF)) * color.2 as u32 * color.3 as u32 / 0xFE01;

            return (val4u << 8*3) | (val3u << 8*2) | (val2u << 8*1) | (val1u << 8*0);

        },
        BlendMode::Multiply => {
            //d-rs maps this to sdl's mod function, but sdl also has an actual multiply function
            //let des_val = canv_dst.buffer_mut()[d_idx];
            //let src_val = sour_buffer[s_idx];

            // //debug halting function
            // if (des_val & 0xFF00) != 0 && (src_val & 0xFF00) != 0 {
            //     let mut detsrc = ((des_val >> (8 * 1)) & 0xFF);
            //     let mut detsrc2 = ((src_val >> (8 * 1)) & 0xFF);
            //     detsrc *= detsrc2;
            //     detsrc2 = detsrc / 0xFF;
            // }

            let val4 = ((des_val >> (8 * 3)) & 0xFF) * color.3 as u32 / 0xFF;
            
            let val3 = (((des_val >> (8 * 2)) & 0xFF) * ((src_val >> (8 * 2)) & 0xFF) / 0xFF) * color.0 as u32 * color.3 as u32 / 0xFE01;
            let val2 = (((des_val >> (8 * 1)) & 0xFF) * ((src_val >> (8 * 1)) & 0xFF) / 0xFF) * color.1 as u32 * color.3 as u32 / 0xFE01;
            let val1 = (((des_val >> (8 * 0)) & 0xFF) * ((src_val >> (8 * 0)) & 0xFF) / 0xFF) * color.2 as u32 * color.3 as u32 / 0xFE01;

            return (val4 << 8*3) | (val3 << 8*2) | (val2 << 8*1) | (val1 << 8*0);

        },
    }
    
}


//see if this is ligher weight than the triangle functions (most likely: yes)
pub fn draw_to_canvas_rect(
    canv_dst: &mut Canvas,
    canv_src: &Canvas,
    rect_src: &Rect<isize>,
    x: isize,
    y: isize,
    mode: BlendMode,
    color_mod: Color,
) {



    //trim rect or break out if the rect is OOB (function returns "none")
    let (rect_src, x, y) = if let Some(rcc) = trim_oob_rects(rect_src, canv_dst.width() as isize, canv_dst.height() as isize, x, y) {rcc}
    else {return};
    
    //let dest_buffer = canv_dst.buffer_mut();
    let sour_buffer = canv_src.buffer();

    //we break color out like this so we don't have to calculate u8 from float values every time
    let (cmr, cmg, cmb, cma) = color_mod.to_rgba();


    for iy in 0..(rect_src.height() as usize) {
        for ix in 0..(rect_src.width() as usize) {

            let dy = iy + y as usize;
            let dx =  ix + x as usize;

            let sy = iy + rect_src.top as usize;
            let sx =  ix + rect_src.left as usize;

            let d_idx = canv_dst.buffer_index(dx, dy);
            let s_idx = canv_src.buffer_index(sx, sy);

            canv_dst.buffer_mut()[d_idx] = blend_pixel(
                sour_buffer[s_idx],
                canv_dst.buffer_mut()[d_idx],
                mode,
                (cmr, cmg, cmb, cma));
            
        }
    }



    //same as a pre-exsisting function, but inlined so its faster
    //index of buffer from source canvas
    //let src_width = canv_src.width();
    //let bf_idx = (rect_src.top as usize * src_width) + rect_src.left as usize;


}

// long map(long x, long in_min, long in_max, long out_min, long out_max) {
//     return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
//   }

#[inline]
fn map<T: Num + PartialOrd + Copy>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

#[inline]
fn buffer_index<T: Num + PartialOrd + Copy>(x: T, y: T, width: T) -> T {
    (y * width) + x
}

pub fn draw_to_canvas_scaled(
    canv_dst: &mut Canvas,
    canv_src: &Canvas,
    rect_dst: &Rect<isize>,
    rect_src: &Rect<isize>,
    flip_x: bool,
    flip_y: bool,   
    mode: BlendMode,
    color_mod: Color, 
) {

    //trim rect or break out if the rect is OOB (function returns "none")
    let (trimmed_rect_dst, x, y) = if let Some(rcc) = trim_oob_rects(
        rect_dst,
        canv_dst.width() as isize,
        canv_dst.height() as isize,
        rect_dst.left, rect_dst.top) {rcc}
    else {return};





    //let dest_buffer = canv_dst.buffer_mut();
    let sour_buffer = canv_src.buffer();

    let (cmr, cmg, cmb, cma) = color_mod.to_rgba();



    for iy in 0..(trimmed_rect_dst.height() as usize) {
        for ix in 0..(trimmed_rect_dst.width() as usize) {

            //destination coordinates
            let dy = iy + y as usize;
            let dx =  ix + x as usize;

            //source coordinates (scale these)
            //let sy = iy + rect_src.top as usize;
            //let sx =  ix + rect_src.left as usize;

            let sy = map((iy as isize) + trimmed_rect_dst.top - rect_dst.top, 0, rect_dst.height(), 0, rect_src.height()) as usize + rect_src.top as usize;;
            let sx = map((ix as isize) + trimmed_rect_dst.left - rect_dst.left, 0, rect_dst.width(), 0, rect_src.width()) as usize + rect_src.left as usize;

            //(y * self.width) + x
            let d_idx = canv_dst.buffer_index(dx, dy);
            let s_idx = canv_src.buffer_index(sx, sy);

            canv_dst.buffer_mut()[d_idx] = blend_pixel(
                sour_buffer[s_idx],
                canv_dst.buffer_mut()[d_idx],
                mode,
                (cmr, cmg, cmb, cma));
            
        }
    }


}


pub fn draw_to_canvas_fill(
    canv_dst: &mut Canvas,
    rect_src: &Rect<isize>,
    blend_mode: BlendMode,
    color_mod: Color,
    color: Color,
) {

    //drawing::rect::fill_rect(c, col, x1, y1, x2, y2)
    //see: drawing::rect::fill_rect

    // Split buffer into chunks of "width", skip until the top of the rectangle and iterate over
    // all rows (y2 - y1) to get each scanline.  Then, set pixels in the scanline in the interval
    // [x1,x2].
    // c.buffer_mut()
    // .as_mut_slice()
    // .chunks_mut(w)
    // .skip(y1)
    // .take(y2 - y1 + 1)
    // .for_each(|scanline|
    //     scanline.iter_mut()
    //         .skip(x1)
    //         .take(x2 - x1 + 1)
    //         .for_each(|x| *x = col)


    //source color (with ARGB format)
    let (cmr, cmg, cmb, cma) = color.to_rgba();
    let clr = u32::from_be_bytes([cma, cmr, cmg, cmb]);

    //dest color (with tuple format)
    let blend_color= color_mod.to_rgba();

    let rect_src = trim_oob_rects(rect_src, canv_dst.width() as isize, canv_dst.height() as isize, rect_src.left, rect_src.top);

    if let Some((rect_src, _, _)) = rect_src {
        let width = canv_dst.width();

        canv_dst.buffer_mut()
            .as_mut_slice()
            .chunks_mut(width)
            .skip(rect_src.top as usize)
            .take((rect_src.bottom - rect_src.top + 1) as usize)
            .for_each(|scanline|
                scanline.iter_mut()
                    .skip(rect_src.left as usize)
                    .take((rect_src.right - rect_src.left + 1) as  usize)
                    .for_each(|x| *x = blend_pixel(clr, *x, blend_mode, blend_color)));
    }

}




pub struct SoftwareTexture {
    width: u16,
    height: u16,
    commands: Vec<SpriteBatchCommand>,

    texture: Rc<RefCell<Canvas>>, //the texture's pseronal canvas
    //texture: Option<Canvas>,

    target_canvas: Rc<RefCell<Rc<RefCell<Canvas>>>>, //refrence to the renderer's main canvas (or the target canvas)

    color_mod: Color,
    alpha_mod: u8,
    blend_mode: Rc<RefCell<BlendMode>>,

}
impl SoftwareTexture {

    // pub fn new() -> SoftwareTexture
    // {
    //     SoftwareTexture{
    //         width: 0,
    //         height: 0,
    //         commands: Vec::new(),
    //         texture: None,
    //         canvas: RefCell::new(value)
    //     }
    // }

}
impl BackendTexture for SoftwareTexture {
    // return the size of this texture.
    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    // push a draw command to the draw command stack
    fn add(&mut self, command: SpriteBatchCommand) {
        self.commands.push(command);
    }

    //delete all commands from the draw command stack
    fn clear(&mut self) {
        self.commands.clear();
    }

    // put stuff from this texture onto the renderer's framebuffer
    fn draw(&mut self) -> GameResult<()> {

        let self_texture = self.texture.borrow_mut();

        //todo: texture setup (things like blend mode, etc.)
        for command in &self.commands {
            match command{
                SpriteBatchCommand::DrawRect(src, dst) => {


                    //let src = Rect::new(src.left as f64 - 0.5, src.top as f64 - 0.5, src.right as f64 - 0.5, src.bottom as f64 - 0.5);
                    //let dst: Rect<isize> = Rect::new((dst.left - 0.5) as isize, (dst.top - 0.5) as isize, (dst.right - 0.5) as isize, (dst.bottom - 0.5) as isize);
                    
                    
                    let src = Rect::new(src.left as f64, src.top as f64, src.right as f64, src.bottom as f64);
                    let dst: Rect<isize> = Rect::new((dst.left) as isize, (dst.top) as isize, (dst.right) as isize, (dst.bottom) as isize);

                    //draw it out of triangles, the openGL way.

                    //if let Some(mut self_texture) = self.texture.as_mut()
                    {
                        let layer_1 = self.target_canvas.borrow_mut();
                        let mut upstream_canvas = layer_1.borrow_mut();
                        {
                            //drawing::shape::textured_triangle(upstream_canvas.deref_mut(), &mut self_texture,
                            //    &vertices[0], &vertices[1], &vertices[2], UVWrapMode::Clamp);
                            //drawing::shape::textured_triangle(upstream_canvas.deref_mut(), &mut self_texture,
                            //    &vertices[3], &vertices[4], &vertices[5], UVWrapMode::Clamp);

                            let mut stopper = 0.0;
                            if src.left as u32 == 32 && src.top as u32 == 16 && src.right as u32 == 48 && src.bottom as u32 == 32 {
                                let mut stopper = src.left;
                            }

                            let src = Rect::new(src.left as isize, src.top as isize, src.right as isize, src.bottom as isize);
                            // draw_to_canvas_rect(&mut upstream_canvas,
                            //     &self_texture,
                            //     &src, dst.left, dst.top,
                            //     self.blend_mode.borrow().to_owned(),
                            //     Color::from_rgba(0xFF, 0xFF, 0xFF, 0xFF),
                            // );

                            draw_to_canvas_scaled(&mut upstream_canvas,
                                &self_texture,
                                &dst,
                                &src,
                                false,
                                false,
                                self.blend_mode.borrow().to_owned(),
                                Color::from_rgba(0xFF, 0xFF, 0xFF, 0xFF),
                            );



                        }
                    }
                    

                }

                SpriteBatchCommand::DrawRectTinted(src, dst, color) => {

                    let src = Rect::new(src.left as f64, src.top as f64, src.right as f64, src.bottom as f64);
                    let dst: Rect<isize> = Rect::new((dst.left) as isize, (dst.top) as isize, (dst.right) as isize, (dst.bottom) as isize);

                    //if let Some(mut self_texture) = self.texture.as_mut()
                    {
                        let layer_1 = self.target_canvas.borrow_mut();
                        let mut upstream_canvas = layer_1.borrow_mut();
                        {
                            

                            if src.left as u32 == 32 && src.top as u32 == 16 && src.right as u32 == 48 && src.bottom as u32 == 32 {
                                let mut stopper = src.left;
                            }

                            let src = Rect::new(src.left as isize, src.top as isize, src.right as isize, src.bottom as isize);
                            // draw_to_canvas_rect(&mut upstream_canvas,
                            //     &self_texture,
                            //     &src, dst.left, dst.top,
                            //     self.blend_mode.borrow().to_owned(),
                            //     color.clone(),
                            // );

                            draw_to_canvas_scaled(&mut upstream_canvas,
                                     &self_texture,
                                     &dst,
                                     &src,
                                     false,
                                     false,
                                     self.blend_mode.borrow().to_owned(),
                                     color.clone(),
                            );

                        }
                    }

                }

                _ =>{

                    let mut command_op = command;
                    if 2 > 3 {
                        let mut cc = command_op.clone();
                    }

                }
            }
        }

        Ok(())
    }

    //return sub-type
    fn as_any(&self) -> &dyn Any {
        self
    }
}


//structure:
/*
A canvas holds all direct draw calls,
On present, we push the stuff in the canvas out to the window

Textures hold refrences to the canvas and put themselves directly on it when their "draw" function is called

*/

//these structs are specific to each backend: they hold just enough of the parent refrence to copy our bitmap buffer to the respective backend
//bal: backend abstraction layer
pub trait BalPresent {
    fn push_out(&mut self, image: &Canvas) -> GameResult {  
        Ok(())
    }

    fn dimensions(&self) -> (u32, u32) {
        (0, 0)
    }


    fn as_any(&self) -> &dyn Any;

}

pub struct BalSdl {
    pub sdl_refs : Rc<RefCell<SDL2Context>>, //refrence to the window we're drawing to
    //note: Textures are hardware accelerated, Surfaces are software accelerated
    //texture: Texture, //input hole to put our rendered images (since we're doing software rendering, we'll want a surface for this)
    surface: Surface<'static>,

    //unused other than to create a window surface refrence
    event_pump: Rc<RefCell<EventPump>>,

    width: u32, //width and height of the texture
    height: u32,
}

impl BalSdl {
    pub fn new(refs: Rc<RefCell<SDL2Context>>, event_pump: Rc<RefCell<EventPump>>) -> GameResult<Box<dyn BalPresent>> {


        let (width, height) = refs.borrow_mut().window.window().size();

        // //create the texture to hold the things we will be sending out
        // let mut texture = refs
        //     .borrow_mut()
        //     .window
        //     .texture_creator()
        //     .create_texture_streaming(PixelFormatEnum::RGBA32, width, height)
        //     .map_err(|e| {
        //         log::info!("{}", e.to_string());
        //         GameError::RenderError(e.to_string())
        //     })?;

        let masks = PixelFormatEnum::BGRA32;
        let mut surface = Surface::new(width, height, masks)
            .map_err(|e| {
            log::info!("{}", e.to_string());
            GameError::RenderError(e.to_string())
        })?;

        Ok(Box::new(BalSdl {
            //texture: texture,
            surface: surface,
            event_pump: event_pump,
            sdl_refs: refs,
            width: width,
            height: height,
        }))
    }


    pub fn resize_texture(&mut self, width: u32, height: u32) -> GameResult {


        /* 
        // textures work, but surfaces are better for software rendering
        // //create a new texture with the proper size
        // let mut texture_opt = self.sdl_refs
        //     .borrow_mut() //for some reason, this does not want to make a RefMut
        //     .window
        //     .texture_creator()
        //     .create_texture_streaming(PixelFormatEnum::RGBA32, width, height)
        //     .map_err(|e| {
        //         log::info!("{}", e.to_string());
        //         GameError::RenderError(e.to_string())
        //     })?;

        // //swap new and old textures before destroying the old one
        // mem::swap(&mut self.texture, &mut texture_opt);
        // unsafe {texture_opt.destroy();}
        */


        //resize SDL surface
        let masks = PixelFormatEnum::BGRA32;
        let mut surface = Surface::new(width, height, masks)
            .map_err(|e| {
            log::info!("{}", e.to_string());
            GameError::RenderError(e.to_string())
        })?;
        mem::swap(&mut self.surface, &mut surface);
        //don't believe we need to destroy the surface when it leaves context since it doesn't use VRAM
        drop(surface);



        self.width = width;
        self.height = height;

        Ok(())
    }




}

impl BalPresent for BalSdl {
    
    //copy the image buffer to the backend's presenter
    fn push_out(&mut self, image: &Canvas) -> GameResult {
        

        let (width, height) = (image.width() as u32, image.height() as u32);

        //resize outoging canvas if needed
        if(width != self.width || height != self.height){
            self.resize_texture(width, height);
        }

        let mut refs = self.sdl_refs.as_ref().borrow_mut();

        let data = image.buffer();
        
        {
            use std::borrow::BorrowMut;
            if let Result::Ok(mut surf) = refs.borrow_mut().window.window().surface(self.event_pump.borrow().deref())
            {
                //surface.height();
                //surf.size();

                let rect = sdl2::rect::Rect::new(
                    0 as i32,
                    0 as i32,
                    surf.width() as u32,
                    surf.height() as u32);
                let color = pixels::Color::RGBA(255, 255, 255, 255);
                self.surface.fill_rect(rect, color);

                self.surface.set_color_mod(color);
                self.surface.set_alpha_mod(255);
                self.surface.set_blend_mode(sdl2::render::BlendMode::None);


                self.surface.with_lock_mut(|buffer: &mut [u8]| {
                    for y in 0..(height as usize) {
                        for x in 0..(width as usize) {
                            let offset = (y *(width as usize) + x) * 4;
                            let data_offset = (y * width as usize + x);// * 4;
                            
                            //in data format (input canvas):
                            //argb, but in little endian format:
                            //[b][g][r][a]
        
                            //out data format:
                            //rgba, individual bytes
                            //[r][g][b][a]
                            let data_1 = (data[data_offset] & 0xFF) as u8;
        
                            buffer[offset + 0] = (data[data_offset] & 0xFF) as u8;
                            buffer[offset + 1] = (data[data_offset] >> 8 & 0xFF) as u8; //data[data_offset + 1];
                            buffer[offset + 2] = (data[data_offset] >> 16 & 0xFF) as u8; //data[data_offset + 2];
                            buffer[offset + 3] = (data[data_offset] >> 24 & 0xFF) as u8; //data[data_offset + 3]; 
                        }
                    }
                });

                //.map_err(|e| GameError::RenderError(e.to_string()))?;
                // let color = pixels::Color::RGBA(255, 255, 0, 255);
                // surf.fill_rect(rect, color);

                let src_rc = Some(sdl2::rect::Rect::new(
                    0 as i32,
                    0 as i32,
                    self.surface.width() as u32,
                    self.surface.height() as u32));
        
                let dst_rc = Some(sdl2::rect::Rect::new(
                    0 as i32,
                    0 as i32,
                    surf.width() as u32,
                    surf.height() as u32));

                let _ = self.surface.blit_scaled(src_rc, &mut surf, dst_rc);

                surf.update_window(); //updates window and keeps ref (software-equivalent of present())
                //surf.finish(); //updates window and deletes ref (we don't want this)
            }

        }

        Ok(())


        /*
        //let lenn = data.len();
        // self.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        //     for y in 0..(height as usize) {
        //         for x in 0..(width as usize) {
        //             let offset = y * pitch + x * 4;
        //             let data_offset = (y * width as usize + x);// * 4;
                    
        //             //in data format (input canvas):
        //             //argb, but in little endian format:
        //             //[b][g][r][a]

        //             //out data format:
        //             //rgba, individual bytes
        //             //[r][g][b][a]

        //             buffer[offset + 2] = (data[data_offset] & 0xFF) as u8;
        //             buffer[offset + 1] = (data[data_offset] >> 8 & 0xFF) as u8; //data[data_offset + 1];
        //             buffer[offset + 0] = (data[data_offset] >> 16 & 0xFF) as u8; //data[data_offset + 2];
        //             buffer[offset + 3] = (data[data_offset] >> 24 & 0xFF) as u8; //data[data_offset + 3]; 
        //         }
        //     }
        // })
        // .map_err(|e| GameError::RenderError(e.to_string()))?;

        // self.texture.set_color_mod(255, 255, 255);
        // self.texture.set_alpha_mod(255);

        // let canvas = refs.window.canvas();
        // //canvas.draw_rect(Rect::new(0, 0, 200, 200));

        // //canvas source
        // let src_rc = Some(sdl2::rect::Rect::new(
        //     0 as i32,
        //     0 as i32,
        //     width as u32,
        //     height as u32));

        // let dst_rc = Some(sdl2::rect::Rect::new(
        //     0 as i32,
        //     0 as i32,
        //     width as u32,
        //     height as u32));
        
        
        // //canvas.copy(&self.texture, src_rc, dst_rc);
        // //canvas.present();
        // //end TEXTURE method
        */


    }

    fn dimensions(&self) -> (u32, u32) {
        let mut refs = self.sdl_refs.as_ref().borrow_mut();
        refs.window.window().size()
    }
    fn as_any(&self) -> &dyn Any {
        self
    } 
}

pub struct SoftwareRenderer{

    //container for window-transer refrences
    presenter: Box<dyn BalPresent>,

    imgui: RefCell<imgui::Context>,



    //main screen buffer
    canvas: Rc<RefCell<Canvas>>,

    //target buffer (will often be the same as "canvas", but will change depending on what the target is)
    target_canvas: Rc<RefCell<Rc<RefCell<Canvas>>>>,

    //shared blend mode
    blend_mode: Rc<RefCell<BlendMode>>,

    //test: changing parameters
    timer: u32,

}


impl SoftwareRenderer {

    //we need this to take multiple refence types
    #[allow(clippy::new_ret_no_self)]
    pub fn new(refs_sdl: Option<Rc<RefCell<SDL2Context>>>, event_pump: Rc<RefCell<EventPump>>, size_hint: (u32, u32)) -> GameResult<Box<dyn BackendRenderer>> {
        
        let mut imgui = init_imgui()?;
        imgui.io_mut().display_size = [size_hint.0 as f32, size_hint.1 as f32];
        imgui.fonts().build_alpha8_texture();

        //todo: take other types of refs (implement things like BalGl)
        let presenter = BalSdl::new(refs_sdl.unwrap(), event_pump)?;

        let canvas = Rc::new(RefCell::new(Canvas::new(size_hint.0 as usize, size_hint.1 as usize)));
        let target_canvas =  Rc::new(RefCell::new(canvas.clone()));
        Ok(Box::new(SoftwareRenderer{
            presenter: presenter,
            imgui: RefCell::new(imgui),
            //buff: ImageBuffer::new(320, 240),
            timer: 0,
            canvas,
            target_canvas,
            blend_mode: Rc::new(RefCell::new(BlendMode::Alpha)),
        }))

    }
}

impl BackendRenderer for SoftwareRenderer {

    //return the name of the backend
    fn renderer_name(&self) -> String {
        "Software".to_owned()
    }

    //puts nothing but a single color on the window
    fn clear(&mut self, color: Color) {

        // let color = color.to_rgba();
        // let color = ARGBColour::new(color.3, color.0, color.1, color.2);
        // self.canvas.borrow_mut().clear(&color);

        let canvas = self.target_canvas.borrow_mut();
        let rect = Rect::new(0,0, canvas.borrow().width() as isize, canvas.borrow().height() as isize);
        draw_to_canvas_fill(
            canvas.borrow_mut().deref_mut(),
            &rect, self.blend_mode.as_ref().borrow().deref().clone(),
            Color::from_rgba_u32(0xFFFFFFFF), color);
        
    }

    //puts canvas on the window, called on every frame we want drawn
    fn present(&mut self) -> GameResult {

        //the way this works will have to change with the frontend we're using...

        //if SDL, we need to take our framebuffer and push it to the SDL canvas,
        //if glutin, we need to do likewise so that glutin will understand it... however that works        

        // let pitch = 2;
        // for y in 0..240 {
        //     for x in 0..320 {
        //         let offset = y * pitch + x * 3;
        //         self.buff[offset] = x as u8;
        //         self.buff[offset + 1] = y as u8;
        //         self.buff[offset + 2] = 0;
        //     }
        // }
        //buff foramt: [RGBRGBRGBRGB]
        
        //let imgbf: Option<ImageBuffer<image::Rgba<u8>, Vec<u8>>> = RgbaImage::from_raw(320, 240, self.buff);
        //let imgbf = image::load_from_memory(&self.buff)?;
        //let imgae_bf_fin: ImageBuffer<image::Rgba<u8>, Vec<u8>> = imgbf.to_rgba8();
        
        //let mut img: ImageBuffer<image::Rgba<u8>, Vec<u8>> = 
        //self.buff = img;


        //test: create an imagebuffer formatted image to push to the backend
        // let portion = (self.timer as f32 / 60.0).sin();
        // self.buff = ImageBuffer::from_fn(320, 240, |x, y| {
        //     if (x + y) % 2 == 0 {
        //         image::Rgba([0, 0, 0, 255])
        //     } else {
        //         image::Rgba([(255.0 * portion) as u8, 255, 255, 255])
        //     }
        // });
        // self.timer += 1;

        //check for changed parent canvas size and resize software canvas accordingly
        let curr_dims = (self.canvas.borrow().width() as u32, self.canvas.borrow().height() as u32);
        let bal_dims = self.presenter.dimensions();
        if curr_dims.0 != bal_dims.0 || curr_dims.1 != bal_dims.1 {

            //let ab = *self.canvas.borrow_mut();
            *self.canvas.borrow_mut() = Canvas::new(bal_dims.0 as usize, bal_dims.1 as usize);

            //self.canvas.borrow_mut().deref_mut() = Canvas::new(bal_dims.0 as usize, bal_dims.1 as usize);
        }

        self.presenter.push_out(self.canvas.borrow_mut().deref_mut());



        Ok(())
    }

    //canvas: Rc::new(RefCell::new(Canvas::new(size_hint.0 as usize, size_hint.1 as usize)))
    //produce a BackendTexture that can be altered
    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(
            SoftwareTexture{
                width,
                height,
                commands: vec![],
                texture: Rc::new(RefCell::new(Canvas::new(width as usize, height as usize))), //Some(Canvas::new(width as usize, height as usize)),
                target_canvas: self.target_canvas.clone(), //Rc::new(RefCell::new(Canvas::new(width as usize, height as usize))),
                color_mod: Color::from_rgba(0xFF, 0xFF, 0xFF, 0xFF),
                alpha_mod: 0xFF,
                blend_mode: self.blend_mode.clone(),
            }
        ))
    }

    //produce a BackendTexture that cannot be altered? (pre-filled with data)
    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        //todo: copy data to the image buffer

        //data format: rgba
        let mut canvas_buf = Canvas::new(width as usize, height as usize);

        //fill canvas with texture
        let vec_pointer = canvas_buf.buffer_mut();
        for x in 0..width as usize {
            for y in 0..height as usize {

                let index = y * width as usize + x;

                //data order format
                //[r][g][b][a] repeated

                //argb, but in little endian format:
                //[b][g][r][a]

                let (r, g, b, a) = 
                    (
                        data[index * 4] as u32,
                        data[index * 4 + 1] as u32,
                        data[index * 4 + 2] as u32,
                        data[index * 4 + 3] as u32
                    );

                let argb: u32 = (
                    b << 0 |
                    g << 8 |
                    r << 16 |
                    a << 24
                );

                vec_pointer[index] = argb;
            }
        }


        Ok(Box::new(
            SoftwareTexture{
                width,
                height,
                commands: vec![],
                texture: Rc::new(RefCell::new(canvas_buf)),
                target_canvas: self.target_canvas.clone(),
                color_mod: Color::from_rgba(0xFF, 0xFF, 0xFF, 0xFF),
                alpha_mod: 0xFF,
                blend_mode: self.blend_mode.clone(),
            }
        ))
    }

    //when a backend texture is copied from one to another, how the pixels are blended together
    //or is this when the final is copied onscreen...
    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        self.blend_mode.replace(blend);
        Ok(())
    }

    //tell the renderer where to draw to (texture, whose format is determined by BackendTexture), or with none default back to the canvas
    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {

        match texture {
            Some(texture) => {
                let software_texture = texture
                    .as_any()
                    .downcast_ref::<SoftwareTexture>()
                    .ok_or(GameError::RenderError("This texture was not created by Software backend.".to_string()))?;
                
                self.target_canvas.replace(software_texture.texture.clone());


                //tests
                //let yy = self.target_canvas.swap(other)
                //let yy = self.target_canvas.as_ptr();
                //*yy = software_texture.texture.clone();
                //let yy = self.target_canvas.;
                //self.target_canvas.as_ref() = software_texture.texture.as_ref();
            
            }
            None => {
                self.target_canvas.replace(self.canvas.clone());
            }
        }
        Ok(())
    }

    //fill the window buffer directly with a colored rectangle
    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {

        //todo: replace this with function that does blending
        //let color = color.to_rgba();
        //let color = ARGBColour::new(color.3, color.0, color.1, color.2);
        //drawing::rect::fill_rect(self.canvas.borrow_mut().deref_mut(), &color, rect.left, rect.top, rect.right, rect.bottom);

        //let mut image = RgbaImage::new(200, 200);
        //draw_filled_rect_mut(&mut self.buff, ImRect::at(130, 10).of_size(20, 20), color);

        let stage1 = self.target_canvas.borrow_mut();
        draw_to_canvas_fill(
            stage1.borrow_mut().deref_mut(),
            &rect, self.blend_mode.as_ref().borrow().deref().clone(),
            Color::from_rgba_u32(0xFFFFFFFF), color);


        //drawing::rect::fill_rect(stage1.borrow_mut().deref_mut(), &color, rect.left, rect.top, rect.right, rect.bottom);
            
        Ok(())
        

    }

    //draws a rectangle like draw_rect, but with lines instead
    fn draw_outline_rect(&mut self, rect: Rect<isize>, _line_width: usize, color: Color) -> GameResult {

        let rect: Rect<i32> = Rect{left: rect.left as i32, right: rect.right as i32, top: rect.top as i32, bottom: rect.bottom as i32};
        let color = color.to_rgba();
        let color = ARGBColour::new(color.3, color.0, color.1, color.2);
        
        //note: could have also used: drawing::rect::rect(c, col, x1, y1, x2, y2)

        //todo: line with
        drawing::shape::polygon(self.canvas.borrow_mut().deref_mut(), &color, true,
            vec![
            rect.left, rect.top,
            rect.right, rect.top,
            rect.right, rect.bottom,
            rect.left, rect.bottom,
            ]);

        Ok(())
    }

    //set the space where drawing stuff will be ignored on the canvas
    fn set_clip_rect(&mut self, _rect: Option<Rect>) -> GameResult {
        Ok(())
    }

    //return pointer to imgui's context
    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.imgui.as_ptr()) }
    }

    //return texture ID for imgui
    fn imgui_texture_id(&self, _texture: &Box<dyn BackendTexture>) -> GameResult<TextureId> {
        Ok(TextureId::from(0))
    }

    //not really needed...
    fn prepare_imgui(&mut self, _ui: &Ui) -> GameResult {
        Ok(())
    }

    //put all imgui data onto the canvas
    fn render_imgui(&mut self, _draw_data: &DrawData) -> GameResult {
        Ok(())
    }

    //draw a series of triangles, denoted by points
    fn draw_triangle_list(
        &mut self,
        _vertices: &[VertexData],
        _texture: Option<&Box<dyn BackendTexture>>,
        _shader: BackendShader,
    ) -> GameResult<()> {
        Ok(())
    }

    //return self, is this for getting the sub-type out of the trait?
    fn as_any(&self) -> &dyn Any {
        self
    }
}
