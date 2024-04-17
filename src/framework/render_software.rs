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
use std::ops::{Deref, DerefMut};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::vec::Vec;

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



//see if this is ligher weight than the triangle functions
pub fn draw_to_canvas_rect(
    canv_dst: &mut Canvas,
    canv_src: &Canvas,
    x: usize,
    y: usize,
    rect_src: &Rect<isize>,
) {
    //handle OOB cases
    if x >= canv_dst.width() ||
    y >= canv_dst.height() ||
    rect_src.width() == 0 ||
    rect_src.height() == 0 {
        return
    }

    let dest_buffer = canv_dst.buffer_mut();

    // dest_buffer
    //     .as_mut_slice()
    //     .chunks_mut(canv_dst.width())
    //     .skip(y) //skip over the unused top of the destination
    //     .take()



}







pub struct SoftwareTexture {
    width: u16,
    height: u16,
    commands: Vec<SpriteBatchCommand>,
    texture: Option<Canvas>, //the texture's pseronal canvas
    canvas: Rc<RefCell<Canvas>>, //refrence to the renderer's main canvas
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
        match &mut self.texture {
            None => Ok(()),
            Some(texture) => {

                //todo: texture setup (things like blend mode, etc.)
                for command in &self.commands {
                    match command{
                        SpriteBatchCommand::DrawRect(src, dst) => {


                            let src = Rect::new(src.left as f64, src.top as f64, src.right as f64, src.bottom as f64);
                            let dst: Rect<isize> = Rect::new(dst.left.round() as isize, dst.top.round() as isize, dst.right.round() as isize, dst.bottom.round() as isize);
                            //draw it out of triangles, the openGL way.

                            if let Some(mut self_texture) = self.texture.as_mut()
                            {
                                let (tex_scale_x, tex_scale_y) = (1.0 / self.width as f64, 1.0 / self.height as f64);

                                let vertices = [
                                    UVVertex::new(dst.left, dst.bottom, src.left * tex_scale_x, src.bottom * tex_scale_y),
                                    UVVertex::new(dst.left, dst.top, src.left * tex_scale_x, src.top * tex_scale_y),
                                    UVVertex::new(dst.right, dst.top, src.right * tex_scale_x, src.top * tex_scale_y),

                                    UVVertex::new(dst.left, dst.bottom, src.left * tex_scale_x, src.bottom * tex_scale_y),
                                    UVVertex::new(dst.right, dst.top, src.right * tex_scale_x, src.top * tex_scale_y),
                                    UVVertex::new(dst.right, dst.bottom, src.right * tex_scale_x, src.bottom * tex_scale_y),
                                ];

                                let mut upstream_canvas = self.canvas.borrow_mut();
                                {
                                    drawing::shape::textured_triangle(upstream_canvas.deref_mut(), &mut self_texture,
                                        &vertices[0], &vertices[1], &vertices[2], UVWrapMode::Clamp);
                                    drawing::shape::textured_triangle(upstream_canvas.deref_mut(), &mut self_texture,
                                        &vertices[3], &vertices[4], &vertices[5], UVWrapMode::Clamp);
                                }
                            }
                            

                        }
                        _ =>{}
                    }
                }

                Ok(())
            }
        }
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
    texture: Texture, //input hole to put our rendered images
    surface: Surface<'static>,

    width: u32, //width and height of the texture
    height: u32,
}

impl BalSdl {
    pub fn new(refs: Rc<RefCell<SDL2Context>>) -> GameResult<Box<dyn BalPresent>> {


        let (width, height) = refs.borrow_mut().window.window().size();

        //create the texture to hold the things we will be sending out
        let mut texture = refs
            .borrow_mut()
            .window
            .texture_creator()
            .create_texture_streaming(PixelFormatEnum::RGBA32, width, height)
            .map_err(|e| {
                log::info!("{}", e.to_string());
                GameError::RenderError(e.to_string())
            })?;

        let masks = PixelFormatEnum::BGRA32;
        let mut surface = Surface::new(width, height, masks)
            .map_err(|e| {
            log::info!("{}", e.to_string());
            GameError::RenderError(e.to_string())
        })?;


        Ok(Box::new(BalSdl {
            texture: texture,
            surface: surface,
            sdl_refs: refs,
            width: width,
            height: height,
        }))
    }


    pub fn resize_texture(&mut self, width: u32, height: u32) -> GameResult {


        //create a new texture with the proper size
        let mut texture_opt = self.sdl_refs
            .borrow_mut() //for some reason, this does not want to make a RefMut
            .window
            .texture_creator()
            .create_texture_streaming(PixelFormatEnum::RGBA32, width, height)
            .map_err(|e| {
                log::info!("{}", e.to_string());
                GameError::RenderError(e.to_string())
            })?;

        //swap new and old textures before destroying the old one
        mem::swap(&mut self.texture, &mut texture_opt);
        unsafe {texture_opt.destroy();}

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
        let lenn = data.len();

        self.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..(height as usize) {
                for x in 0..(width as usize) {
                    let offset = y * pitch + x * 4;
                    let data_offset = (y * width as usize + x);// * 4;
                    
                    //in data format (input canvas):
                    //argb, but in little endian format:
                    //[b][g][r][a]

                    //out data format:
                    //rgba, individual bytes
                    //[r][g][b][a]

                    buffer[offset + 2] = (data[data_offset] & 0xFF) as u8;
                    buffer[offset + 1] = (data[data_offset] >> 8 & 0xFF) as u8; //data[data_offset + 1];
                    buffer[offset + 0] = (data[data_offset] >> 16 & 0xFF) as u8; //data[data_offset + 2];
                    buffer[offset + 3] = (data[data_offset] >> 24 & 0xFF) as u8; //data[data_offset + 3]; 
                }
            }
        })
        .map_err(|e| GameError::RenderError(e.to_string()))?;

        self.texture.set_color_mod(255, 255, 255);
        self.texture.set_alpha_mod(255);

        let canvas = refs.window.canvas();
        //canvas.draw_rect(Rect::new(0, 0, 200, 200));

        //canvas source
        let src_rc = Some(sdl2::rect::Rect::new(
            0 as i32,
            0 as i32,
            width as u32,
            height as u32));

        let dst_rc = Some(sdl2::rect::Rect::new(
            0 as i32,
            0 as i32,
            width as u32,
            height as u32));
        
        
        canvas.copy(&self.texture, src_rc, dst_rc);

        canvas.present();



        Ok(())


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
    //buff: ImageBuffer<image::Rgba<u8>, Vec<u8>>, //Vec<u8>, //Vec<RgbaImage>,


    canvas: Rc<RefCell<Canvas>>,

    //test: changing parameters
    timer: u32,

}


impl SoftwareRenderer {

    //we need this to take multiple refence types
    #[allow(clippy::new_ret_no_self)]
    pub fn new(refs_sdl: Option<Rc<RefCell<SDL2Context>>>, size_hint: (u32, u32)) -> GameResult<Box<dyn BackendRenderer>> {
        
        let mut imgui = init_imgui()?;
        imgui.io_mut().display_size = [size_hint.0 as f32, size_hint.1 as f32];
        imgui.fonts().build_alpha8_texture();

        //todo: take other types of refs (implement things like BalGl)
        let presenter = BalSdl::new(refs_sdl.unwrap())?;

        Ok(Box::new(SoftwareRenderer{
            presenter: presenter,
            imgui: RefCell::new(imgui),
            //buff: ImageBuffer::new(320, 240),
            timer: 0,
            canvas: Rc::new(RefCell::new(Canvas::new(size_hint.0 as usize, size_hint.1 as usize)))
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
        let color = color.to_rgba();
        let color = ARGBColour::new(color.3, color.0, color.1, color.2);
        self.canvas.borrow_mut().clear(&color);
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


        self.presenter.push_out(self.canvas.borrow_mut().deref_mut());

        //check for changed parent canvas size and resize software canvas accordingly
        let curr_dims = (self.canvas.borrow().width() as u32, self.canvas.borrow().height() as u32);
        let bal_dims = self.presenter.dimensions();
        if curr_dims.0 != bal_dims.0 || curr_dims.1 != bal_dims.1 {

            //let ab = *self.canvas.borrow_mut();
            *self.canvas.borrow_mut() = Canvas::new(bal_dims.0 as usize, bal_dims.1 as usize);

            //self.canvas.borrow_mut().deref_mut() = Canvas::new(bal_dims.0 as usize, bal_dims.1 as usize);
        }



        Ok(())
    }

    //produce a BackendTexture that can be altered
    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(
            SoftwareTexture{
                width,
                height,
                commands: vec![],
                texture: Some(Canvas::new(width as usize, height as usize)),
                canvas: Rc::new(RefCell::new(Canvas::new(width as usize, height as usize))),
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
                texture: Some(canvas_buf),
                canvas: self.canvas.clone(),
            }
        ))
    }

    //when a backend texture is copied from one to another, how the pixels are blended together
    fn set_blend_mode(&mut self, _blend: BlendMode) -> GameResult {
        Ok(())
    }

    //tell the renderer where to draw to (texture, whose format is determined by BackendTexture), or with none default back to the canvas
    fn set_render_target(&mut self, _texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        Ok(())
    }

    //fill the window buffer directly with a colored rectangle
    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {

        let color = color.to_rgba();
        let color = ARGBColour::new(color.3, color.0, color.1, color.2);
        
        drawing::rect::fill_rect(self.canvas.borrow_mut().deref_mut(), &color, rect.left, rect.top, rect.right, rect.bottom);

        //let mut image = RgbaImage::new(200, 200);
        //draw_filled_rect_mut(&mut self.buff, ImRect::at(130, 10).of_size(20, 20), color);

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
