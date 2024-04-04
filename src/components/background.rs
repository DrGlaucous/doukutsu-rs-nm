use crate::common::{Color, Rect};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::{filesystem, graphics};
use crate::game::frame::Frame;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::{BackgroundType, Stage, StageTexturePaths};
use crate::scene::game_scene::LightingMode;
//use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::GameError;
use crate::util::rng::{Xoroshiro32PlusPlus, RNG};

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

    //internal only: do not save to or load from JSON
    #[serde(skip)]
    pub ani_wait: u32,

    //unneded: using frame_start diectly now
    // #[serde(skip)]
    // pub ani_no: u32,

}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LayerConfig {
    
    pub layer_enabled: bool,
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

    //internal only: do not save to or load from JSON
    #[serde(skip)]
    pub layer_x_value: f32, //I think these are the starting positions for each bitmap?
    #[serde(skip)]
    pub layer_y_value: f32,

}

impl LayerConfig {
    pub fn new() -> LayerConfig {
        LayerConfig{
            layer_enabled: true,
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
                ani_wait: 0,
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
            },

            //non-config items
            layer_x_value: 0.0,
            layer_y_value: 0.0,

        }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct BkgConfig {
    #[serde(default = "current_version")] //what is set to if not found in the json
    pub version: u32,
    pub bmp_filename: String,
    pub lighting_mode: u8,
    pub layers : Vec<LayerConfig>,
}

#[inline(always)]
fn current_version() -> u32 {
    1
}


impl BkgConfig {

    pub fn load(ctx: &Context, path: &String) -> GameResult<BkgConfig> {
        //open from ./data/bkg/ folder
        match filesystem::open(ctx, String::from("/bkg/") + path + ".json") {
            Ok(file) => {
                match serde_json::from_reader::<_, BkgConfig>(file) {
                    Ok(bkg_config) => return Ok(bkg_config),
                    Err(err) => {
                        log::warn!("Failed to deserialize bkg file: {}", err);
                        return Err(GameError::from(err));
                    },
                } 
            }
            Err(err) =>{
                log::warn!("Failed to open bkg file: {}", err);
                return Err(GameError::from(err));
            }
        }

        //Ok(BkgConfig::default())
    }

    //a near-clone of the upgrade path in settings.rs, in case more featues need to be added, old config files can be updated to match
    pub fn upgrade(mut self) -> Self {

        let initial_version = self.version;

        //if self.version == 1 {}
        //if self.version == 2 {}


        if self.version != initial_version {
            log::info!("Upgraded bkg file from version {} to {}.", initial_version, self.version);
        }

        self
    }


    //using this to get the template for other BKG files, it serves no other real purpose
    pub fn save(&self, ctx: &Context, path: &String) -> GameResult {
        let file = filesystem::user_create(ctx, "/".to_string() + path + ".json")?;
        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

}

impl Default for BkgConfig{

    fn default() -> BkgConfig {
        BkgConfig{
            version: current_version(),
            bmp_filename: String::new(),
            lighting_mode: 0,
            layers: vec![LayerConfig::new()],
        }
    }

}



pub struct Background {
    pub tick: usize,
    pub prev_tick: usize,

    //new
    pub bk_config: BkgConfig,
    rng: Xoroshiro32PlusPlus,//::new(0),

}

impl Background {
    pub fn new() -> Self {
        Background {
            tick: 0,
            prev_tick: 0,
            bk_config: BkgConfig::default(),
            rng: Xoroshiro32PlusPlus::new(0),

        }
    }

    pub fn load_bkg_custom(
        &mut self,
        ctx: &mut Context,
        textures: &mut StageTexturePaths,
        stage: &mut Stage,
        lighting_mode: &mut LightingMode,
        path: &String,
    ) -> GameResult<()> {

        match BkgConfig::load(ctx, path)
        {
            //return gotten config or do nothing if none found (for me, create template to get started)
            Ok(config) => {
                textures.background = config.bmp_filename.clone();
                self.bk_config = config.upgrade();
                stage.data.background_type = BackgroundType::Custom;
                *lighting_mode = LightingMode::from(self.bk_config.lighting_mode);
            }
            //if doesn't exsist, return the default config (I also used this to create the first BKG templates for later modification)
            Err(_) => {
                let config = BkgConfig::default();
                config.save(ctx, &textures.background)?;
            }
        };

        // //if the config file is valid, load it in
        // if let Ok(config) = BkgConfig::load(ctx, path) {
        //     textures.background = config.filename.clone();
        //     self.bk_config = config.upgrade();
        //     stage.data.background_type = BackgroundType::Custom;
        // }

        Ok(())

    }

    pub fn tick(
        &mut self,
        state: &mut SharedGameState,
    ) -> GameResult<()> {
        self.tick = self.tick.wrapping_add(1);


        for layer in self.bk_config.layers.as_mut_slice() {
            if !layer.layer_enabled {continue;}

            //advance animation frames
            if layer.animation_style.frame_count > 1 {

                layer.animation_style.ani_wait += 1;
                if layer.animation_style.ani_wait >= layer.animation_style.animation_speed {

                    layer.animation_style.frame_start =
                    if layer.animation_style.frame_start < layer.animation_style.frame_count - 1 {layer.animation_style.frame_start + 1} else {0};

                    layer.animation_style.ani_wait = 0;
                }

            }
            //could also possibly do this without needing mutable vars, but cannot specify start frame, and will not halt when bkg is inactive
            //let equivalent_tick = (self.tick as u32 / layer.animation_style.animation_speed) % layer.animation_style.frame_count;


            //advance location offsets
            let scroll_flags = &layer.animation_style.scroll_flags;

            //if-chain for each flag type

            //handle setting bitmap start offsets based on where they are in the window rect
            if scroll_flags.autoscroll_x {
                layer.layer_x_value -= layer.animation_style.scroll_speed_x;

                //if layer's right corner offset by the times it should be draw is less than 0, shift it over by one bitmap width and window width
                if layer.layer_x_value +
                layer.draw_corner_offset_x +
                (((layer.bmp_width + layer.draw_repeat_gap_x) * layer.draw_repeat_x) as f32) < 0.0 {
                    layer.layer_x_value += (layer.bmp_width + layer.draw_repeat_gap_x) as f32 + state.canvas_size.0 as f32;

                    //if y movement is randomized, add a random value +- animation speed to the y movement
                    if scroll_flags.random_scroll_speed_y {
                        layer.layer_y_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
                    }
                }

                //if layer's left corner is beyond the window width
                if layer.layer_x_value + layer.draw_corner_offset_x > (0.0) {

                    //subtract window width
                    layer.layer_x_value -= state.canvas_size.0 as f32;

                    //wait... why is this done potentially 2x? (does it only happen when the x value advances beyond the frame perhaps?)
                    if scroll_flags.random_scroll_speed_y {
                        layer.layer_y_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
                    }

                }
            }

            //same as above but for y
            if scroll_flags.autoscroll_y {
                layer.layer_y_value -= layer.animation_style.scroll_speed_y;

                //if layer's right corner offset by the times it should be draw is less than 0, shift it over by one bitmap width and window width
                if layer.layer_y_value +
                layer.draw_corner_offset_y +
                (((layer.bmp_height + layer.draw_repeat_gap_y) * layer.draw_repeat_y) as f32) < 0.0 {
                    layer.layer_y_value += (layer.bmp_height + layer.draw_repeat_gap_y) as f32 + state.canvas_size.1 as f32;

                    //if y movement is randomized, add a random value +- animation speed to the y movement
                    if scroll_flags.random_scroll_speed_y {
                        layer.layer_y_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
                    }
                }

                //if layer's left corner is beyond the window width
                if layer.layer_y_value + layer.draw_corner_offset_y > 0.0 {

                    //subtract window height
                    layer.layer_y_value -= state.canvas_size.1 as f32;

                    //wait... why is this done potentially 2x? (does it only happen when the x value advances beyond the frame perhaps?)
                    if scroll_flags.random_scroll_speed_y {
                        layer.layer_y_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
                    }

                }
            }


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
        is_front: bool,
    ) -> GameResult {

        //only attempt to draw in the front if we are using a BKG stage that was front layers
        if is_front && stage.data.background_type != BackgroundType::Custom {
            return Ok(());
        }

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
                //start with empty slate
                graphics::clear(ctx, stage.data.background_color);

                //let (bg_total_width, bg_total_height) = (batch.width() as i32, batch.height() as i32);

                for layer in self.bk_config.layers.as_slice() {
                    if !layer.layer_enabled ||
                    (is_front && !layer.animation_style.scroll_flags.draw_above_foreground) //layer is not flagged to draw above the foreground
                    {continue;}

                    let (xoff, yoff) = (layer.bmp_x_offset + layer.bmp_width * layer.animation_style.frame_start, layer.bmp_y_offset + layer.bmp_height * layer.animation_style.frame_start);

                    let layer_rc = Rect::new(
                        xoff as u16,
                        yoff as u16,
                        (xoff + layer.bmp_width) as u16,
                        (yoff + layer.bmp_height) as u16);
                    
                    //everything below this point can probably be moved to the draw function:
                    {

                        let (cam_x, cam_y) = (frame_x % (batch.width() as f32), frame_y % (batch.height() as f32));


                        let scroll_flags = &layer.animation_style.scroll_flags;

                        //not sure if we need these to be descrete
                        let (rep_x, rep_y) = (layer.draw_repeat_x, layer.draw_repeat_y);

                        //start here and draw bitmap, stepping each time by these coords
                        let mut y_off = layer.layer_y_value as f32;

                        if scroll_flags.align_with_water_lvl {
                            y_off += (state.water_level * 0x200) as f32 - cam_y;
                        }

                        //apply map corner offset
                        y_off += layer.draw_corner_offset_y;

                        if scroll_flags.follow_pc_y {
                            y_off -= cam_y as f32 * layer.animation_style.scroll_speed_y;
                        }

                        if scroll_flags.lock_to_x_axis {
                            y_off -= cam_y as f32;
                        }

                        let mut y = 0;
                        while y < rep_y && y_off < (state.canvas_size.1 as f32) * 16.0 {
                            
                            //need this to reset for each layer
                            let mut x_off = layer.layer_x_value as f32;

                            //apply map corner offset
                            x_off += layer.draw_corner_offset_x;

                            if scroll_flags.follow_pc_x {
                                //TODO: get camera position
                                x_off -= cam_x as f32 * layer.animation_style.scroll_speed_x;
                            }

                            if scroll_flags.lock_to_y_axis {
                                //TODO: get camera position
                                x_off -= cam_x as f32;
                            }


                            //while loop (x-axis)
                            let mut x = 0;
                            while x < rep_x && x_off < (state.canvas_size.0 as f32) * 16.0 {

                                //condition taken care of earler in the draw process
                                //if scroll_flags.draw_above_foreground {}

                                batch.add_rect(x_off as f32, y_off as f32, &layer_rc);

                                //draw bitmap here
                                //x: xOff y: yOff
                                x_off += (layer.bmp_width + layer.draw_repeat_gap_x) as f32;
                                x += 1;
                            }

                            y_off += (layer.bmp_height + layer.draw_repeat_gap_y) as f32;
                            
                            y += 1;
                        }
                    }



                }


                // let (bg_width, bg_height) = (batch.width() as i32, batch.height() as i32);
                // let count_x = state.canvas_size.0 as i32 / bg_width + 1;
                // let count_y = state.canvas_size.1 as i32 / bg_height + 1;
                // for y in -1..count_y {
                //     for x in -1..count_x {
                //         batch.add((x * bg_width) as f32, (y * bg_height) as f32);
                //     }
                // }
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
