use crate::common::{Color, Rect};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::{filesystem, graphics};
use crate::game::frame::Frame;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::{BackgroundType, Stage, StageTexturePaths};


//this could (and probably should) be a bitfield, but I don't know how I'd serialize/deserialize that (inexperience shows)
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScrollFlags {
    pub follow_pc_x: bool,
    pub follow_pc_y: bool,
    pub autoscroll_x: bool,
    pub autoscroll_y: bool,
    pub align_with_water_lvl: bool,
    pub draw_above_foreground: bool,
    pub random_scroll_speed_x: bool,
    pub random_scroll_speed_y: bool,
    pub lock_to_x_axis: bool,
    pub lock_to_y_axis: bool,
    pub randomize_all_parameters: bool,

}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnimationStyle {
    pub frame_count: u32,
    pub frame_start: u32,
    pub animation_speed: u32, //ticks between frame change
    pub scroll_speed_x: f32,
    pub scroll_speed_y: f32,
    pub scroll_flags: ScrollFlags,

}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LayerConfig {
    
    pub bmp_x_offset: u32,
    pub bmp_y_offset: u32,
    pub bmp_width: u32,
    pub bmp_height: u32,

    pub draw_repeat_x: u32,
    pub draw_repeat_y: u32,
    pub draw_repeat_gap_x: u32,
    pub draw_repeat_gap_y: u32,
    pub draw_corner_offset_x: f32,
    pub draw_corner_offset_y: f32,

    pub animation_style: AnimationStyle,
}

impl LayerConfig {
    pub fn new() -> LayerConfig {
        LayerConfig{
            bmp_x_offset: 0,
            bmp_y_offset: 0,
            bmp_width: 0,
            bmp_height: 0,
            draw_repeat_x: 0,
            draw_repeat_y: 0,
            draw_repeat_gap_x: 0,
            draw_repeat_gap_y: 0,
            draw_corner_offset_x: 0.0,
            draw_corner_offset_y: 0.0,
            animation_style: AnimationStyle{
                frame_count: 0,
                frame_start: 0,
                animation_speed: 0,
                scroll_speed_x: 0.0,
                scroll_speed_y: 0.0,
                scroll_flags: ScrollFlags{
                    follow_pc_x: false,
                    follow_pc_y: false,
                    autoscroll_x: false,
                    autoscroll_y: false,
                    align_with_water_lvl: false,
                    draw_above_foreground: false,
                    random_scroll_speed_x: false,
                    random_scroll_speed_y: false,
                    lock_to_x_axis: false,
                    lock_to_y_axis: false,
                    randomize_all_parameters: false,
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BkgConfig {
    #[serde(default = "current_version")]
    pub version: u32,
    pub layers : Vec<LayerConfig>,
}

#[inline(always)]
fn current_version() -> u32 {
    1
}


impl BkgConfig {
    pub fn load(ctx: &Context, path: &String) -> GameResult<BkgConfig> {
        //open from ./data/bkg/ folder
        match filesystem::open(ctx, String::from("bkg/") + path) {
            Ok(file) => {
                match serde_json::from_reader::<_, BkgConfig>(file) {
                    Ok(bkg_config) => return Ok(bkg_config),
                    Err(err) => log::warn!("Failed to deserialize bkg file: {}", err),
                } 
            }
            Err(err) =>{
                log::warn!("Failed to open bkg file: {}", err);
            }
        }

        Ok(BkgConfig::default())
    }

    pub fn default() -> BkgConfig {
        BkgConfig{
            version: current_version(),
            layers: vec![LayerConfig::new()],
        }
    }
}




pub struct Background {
    pub tick: usize,
    pub prev_tick: usize,

    //new
    //used to see if we need to load a different config file for a different BK
    pub bk_name: String,
    //set to true if settings are dynamically modified with TSC, so if the map is re-loaded, we reload the default config from the json
    pub bk_config_modified: bool,
    //the config itself
    pub bk_config: BkgConfig,

}

impl Background {
    pub fn new() -> Self {
        Background {
            tick: 0,
            prev_tick: 0,

            bk_name: String::from(""),
            bk_config_modified: false,
            bk_config: BkgConfig::default(),

        }
    }

    pub fn tick(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        textures: &StageTexturePaths,

    ) -> GameResult<()> {
        self.tick = self.tick.wrapping_add(1);

        //check for bk change
        if textures.background != self.bk_name
        {
            self.bk_config = BkgConfig::load(ctx, &textures.background)?;
            self.bk_name = textures.background.clone();
        }

        Ok(())
    }

    pub fn draw_tick(&mut self) -> GameResult<()> {
        self.prev_tick = self.tick;

        Ok(())
    }

    pub fn draw(
        &self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        frame: &Frame,
        textures: &StageTexturePaths,
        stage: &Stage,
    ) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &textures.background)?;
        let scale = state.scale;
        let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

        match stage.data.background_type {
            BackgroundType::TiledStatic => {
                graphics::clear(ctx, stage.data.background_color);

                let (bg_width, bg_height) = (batch.width() as i32, batch.height() as i32);
                let count_x = state.canvas_size.0 as i32 / bg_width + 1;
                let count_y = state.canvas_size.1 as i32 / bg_height + 1;

                for y in -1..count_y {
                    for x in -1..count_x {
                        batch.add((x * bg_width) as f32, (y * bg_height) as f32);
                    }
                }
            }
            BackgroundType::TiledParallax | BackgroundType::Tiled | BackgroundType::Waterway => {
                graphics::clear(ctx, stage.data.background_color);

                let (off_x, off_y) = if stage.data.background_type == BackgroundType::Tiled {
                    (frame_x % (batch.width() as f32), frame_y % (batch.height() as f32))
                } else {
                    (
                        ((frame_x / 2.0 * scale).floor() / scale) % (batch.width() as f32),
                        ((frame_y / 2.0 * scale).floor() / scale) % (batch.height() as f32),
                    )
                };

                let (bg_width, bg_height) = (batch.width() as i32, batch.height() as i32);
                let count_x = state.canvas_size.0 as i32 / bg_width + 2;
                let count_y = state.canvas_size.1 as i32 / bg_height + 2;

                for y in -1..count_y {
                    for x in -1..count_x {
                        batch.add((x * bg_width) as f32 - off_x, (y * bg_height) as f32 - off_y);
                    }
                }
            }
            BackgroundType::Water => {
                graphics::clear(ctx, stage.data.background_color);
            }
            BackgroundType::Black => {
                graphics::clear(ctx, stage.data.background_color);
            }
            BackgroundType::Scrolling => {
                graphics::clear(ctx, stage.data.background_color);

                let (bg_width, bg_height) = (batch.width() as i32, batch.height() as i32);
                let offset_x = self.tick as f32 % (bg_width as f32 / 3.0);
                let interp_x = (offset_x * (1.0 - state.frame_time as f32)
                    + (offset_x + 1.0) * state.frame_time as f32)
                    * 3.0
                    * scale;

                let count_x = state.canvas_size.0 as i32 / bg_width + 6;
                let count_y = state.canvas_size.1 as i32 / bg_height + 1;

                for y in -1..count_y {
                    for x in -1..count_x {
                        batch.add((x * bg_width) as f32 - interp_x, (y * bg_height) as f32);
                    }
                }
            }
            BackgroundType::OutsideWind | BackgroundType::Outside | BackgroundType::OutsideUnknown => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 0));

                let offset_x = (self.tick % 640) as i32;
                let offset_y = ((state.canvas_size.1 - 240.0) / 2.0).floor();

                // Sun/Moon with 100px buffers on either side
                let (start, width, center) = if state.constants.is_switch {
                    (0, 427, ((state.canvas_size.0 - 427.0) / 2.0).floor())
                } else {
                    (144, 320, ((state.canvas_size.0 - 320.0) / 2.0).floor())
                };

                for x in (0..(center as i32)).step_by(100) {
                    batch.add_rect(x as f32, offset_y, &Rect::new_size(start, 0, 100, 88));
                }

                batch.add_rect(center, offset_y, &Rect::new_size(0, 0, width, 88));

                for x in (center as i32 + width as i32..(state.canvas_size.0 as i32)).step_by(100) {
                    batch.add_rect(x as f32, offset_y, &Rect::new_size(start, 0, 100, 88));
                }

                // top / bottom edges
                if offset_y > 0.0 {
                    let scale = offset_y;

                    for x in (0..(state.canvas_size.0 as i32)).step_by(100) {
                        batch.add_rect_scaled(x as f32, 0.0, 1.0, scale, &Rect::new_size(128, 0, 100, 1));
                    }

                    batch.add_rect_scaled(
                        (state.canvas_size.0 - 320.0) / 2.0,
                        0.0,
                        1.0,
                        scale,
                        &Rect::new_size(0, 0, 320, 1),
                    );

                    for x in ((-offset_x * 4)..(state.canvas_size.0 as i32)).step_by(320) {
                        batch.add_rect_scaled(
                            x as f32,
                            offset_y + 240.0,
                            1.0,
                            scale + 4.0,
                            &Rect::new_size(0, 239, 320, 1),
                        );
                    }
                }

                for x in ((-offset_x / 2)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 88.0, &Rect::new_size(0, 88, 320, 35));
                }

                for x in ((-offset_x % 320)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 123.0, &Rect::new_size(0, 123, 320, 23));
                }

                for x in ((-offset_x * 2)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 146.0, &Rect::new_size(0, 146, 320, 30));
                }

                for x in ((-offset_x * 4)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 176.0, &Rect::new_size(0, 176, 320, 64));
                }
            }

            BackgroundType::Custom => {
                graphics::clear(ctx, stage.data.background_color);

                let (bg_width, bg_height) = (batch.width() as i32, batch.height() as i32);
                let count_x = state.canvas_size.0 as i32 / bg_width + 1;
                let count_y = state.canvas_size.1 as i32 / bg_height + 1;

                for y in -1..count_y {
                    for x in -1..count_x {
                        batch.add((x * bg_width) as f32, (y * bg_height) as f32);
                    }
                }
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
