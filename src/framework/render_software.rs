use std::any::Any;
use std::cell::RefCell;
use std::mem;

use imgui::{DrawData, TextureId, Ui};

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendEventLoop, BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::BlendMode;
use crate::game::Game;



pub struct SoftwareTexture(u16, u16);

impl BackendTexture for SoftwareTexture {
    //return the size of this texture.
    fn dimensions(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    //push a draw command to the draw command stack
    fn add(&mut self, _command: SpriteBatchCommand) {}

    //delete all commands from the draw command stack
    fn clear(&mut self) {}

    
    fn draw(&mut self) -> GameResult<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct SoftwareRenderer(RefCell<imgui::Context>);

//structure:
/*
A canvas holds all direct draw calls,
On present, we push the stuff in the canvas out to the window


*/
impl BackendRenderer for SoftwareRenderer {
    fn renderer_name(&self) -> String {
        "Null".to_owned()
    }

    //puts nothing but a single color on the window
    fn clear(&mut self, _color: Color) {}

    //puts canvas on the window, called on every frame we want drawn
    fn present(&mut self) -> GameResult {
        Ok(())
    }

    //produce a BackendTexture that can be altered
    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    //produce a BackendTexture that cannot be altered? (pre-filled with data)
    fn create_texture(&mut self, width: u16, height: u16, _data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    //when a backend texture is copied from one to another, how the pixels are blended together
    fn set_blend_mode(&mut self, _blend: BlendMode) -> GameResult {
        Ok(())
    }

    //tell the renderer where to draw to (texture, whose format is determined by BackendTexture)
    fn set_render_target(&mut self, _texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        Ok(())
    }

    //fill the window buffer directly with a colored rectangle
    fn draw_rect(&mut self, _rect: Rect<isize>, _color: Color) -> GameResult {
        Ok(())
    }

    //draws a rectangle like draw_rect, but with lines instead
    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    //set the space where drawing stuff will be ignored on the canvas
    fn set_clip_rect(&mut self, _rect: Option<Rect>) -> GameResult {
        Ok(())
    }

    //return pointer to imgui's context
    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.0.as_ptr()) }
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
