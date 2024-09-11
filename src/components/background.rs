use std::borrow::BorrowMut;
use std::path::PathBuf;

use crate::common::{Color, Rect};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::{filesystem, graphics};
use crate::game::frame::Frame;
use crate::game::shared_game_state::{SharedGameState, TileSize};
use crate::game::stage::{BackgroundType, Stage, StageTexturePaths};
use crate::scene::game_scene::LightingMode;
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
    pub random_offset_x: bool,
    pub random_offset_y: bool,
    pub lock_to_x_axis: bool,
    pub lock_to_y_axis: bool,
    pub randomize_all_parameters: bool,
    //introduced with json version 2
    #[serde(default = "default_false")]
    pub add_screen_width: bool,
    #[serde(default = "default_false")]
    pub add_screen_height: bool,
    #[serde(default = "default_true")]
    pub relative_to_pillarbox: bool,
    #[serde(default = "default_true")]
    pub relative_to_letterbox: bool,

}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnimationStyle {
    pub frame_count: u32,
    pub frame_start: u32,
    pub animation_speed: u32, //ticks between frame change
    pub follow_speed_x: f32, //for player-following movement
    pub follow_speed_y: f32,
    pub autoscroll_speed_x: f32, //for automatic movement
    pub autoscroll_speed_y: f32,
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
    pub layer_x_value: f32, //I think these are the starting positions for each bitmap when drawn on the screen
    #[serde(skip)]
    pub layer_y_value: f32,

    //calculate additional frame-realtive offsets (like distant scrolling) and place them here so the other offset functions can get at them
    #[serde(skip)]
    pub frame_x_offset: f32,
    #[serde(skip)]
    pub frame_y_offset: f32,

    //coordinates to use for refrencing the screen edge, can be unique for each layer
    #[serde(skip)]
    pub edge_coords: Rect<f32>,

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
                follow_speed_x: 0.0,
                follow_speed_y: 0.0,
                autoscroll_speed_x: 0.0,
                autoscroll_speed_y: 0.0,
                scroll_flags: ScrollFlags{
                    follow_pc_x: false,
                    follow_pc_y: false,
                    autoscroll_x: false,
                    autoscroll_y: false,
                    align_with_water_lvl: false,
                    draw_above_foreground: false,
                    random_offset_x: false,
                    random_offset_y: false,
                    lock_to_x_axis: false,
                    lock_to_y_axis: false,
                    randomize_all_parameters: false,
                    add_screen_width: false,
                    add_screen_height: false,
                    relative_to_pillarbox: true,
                    relative_to_letterbox: true,
                }
            },

            //non-config items
            layer_x_value: 0.0, //current location of the layer on the window
            layer_y_value: 0.0,
            frame_x_offset: 0.0, //extra offsets to apply from the camera
            frame_y_offset: 0.0,
            edge_coords: Rect::new(0.0, 0.0, 0.0, 0.0),

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

    //#[serde(skip)]
    //path: String, //keep the path we loaded with so we can upgrade if needed

}

#[inline(always)]
fn current_version() -> u32 {
    2
}

fn default_false() -> bool {
    false
}
fn default_true() -> bool {
    true
}



impl BkgConfig {

    pub fn load(ctx: &Context, path: &String) -> GameResult<BkgConfig> {
        //open from ./data/bkg/ folder
        match filesystem::open(ctx, String::from("/bkg/") + path + ".json") {
            Ok(file) => {
                match serde_json::from_reader::<_, BkgConfig>(file) {
                    Ok(bkg_config) => return Ok(bkg_config.upgrade(path)),
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
    pub fn upgrade(mut self, path: &String) -> Self {

        let initial_version = self.version;

        if self.version == 1 {
            self.version = 2;
        }

        if self.version != initial_version {
            log::info!("Upgraded bkg file \"{}\" from version {} to {}.", path, initial_version, self.version);

            if let Err(r) = self.write_out(path) {
                log::error!("Failed to save updated bkg file: {}", r);
            }
        }

        self
    }

    //using this to get the template for other BKG files, it serves no other real purpose
    pub fn save(&self, ctx: &Context, path: &String) -> GameResult {
        let file = filesystem::user_create(ctx, "/".to_string() + path + ".json")?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub fn update_parameter(&mut self, layer_no: usize, parameter: usize, value: usize) -> GameResult {
        
        //ensure that there is a valid layer to alter
        if self.layers.len() > 0 {
            //handle OOB layers
            let layer_no = if layer_no >= self.layers.len() {self.layers.len() - 1} else {layer_no};


            let layer_ref = self.layers[layer_no].borrow_mut();
            let value = value as u32;
            match parameter
            {
                0 => layer_ref.layer_enabled = value != 0,
                1 => layer_ref.bmp_x_offset = value,
                2 => layer_ref.bmp_y_offset = value,
                3 => layer_ref.bmp_width = value,
                4 => layer_ref.bmp_height = value,
                5 => layer_ref.draw_repeat_x = value,
                6 => layer_ref.draw_repeat_y = value,
                7 => layer_ref.draw_repeat_gap_x = value,
                8 => layer_ref.draw_repeat_gap_y = value,
                9 => layer_ref.draw_corner_offset_x = value as f32,
                10 => layer_ref.draw_corner_offset_y = value as f32,

                //animation_style
                11 => layer_ref.animation_style.frame_count = value,
                12 => layer_ref.animation_style.frame_start = value,
                13 => layer_ref.animation_style.animation_speed = value,
                14 => layer_ref.animation_style.follow_speed_x = value as f32,
                15 => layer_ref.animation_style.follow_speed_y = value as f32,
                16 => layer_ref.animation_style.autoscroll_speed_x = value as f32,
                17 => layer_ref.animation_style.autoscroll_speed_y = value as f32,

                //scroll flags (set from bitfield)
                18 => {
                    layer_ref.animation_style.scroll_flags.follow_pc_x = 0 < (value & 1 << 0);
                    layer_ref.animation_style.scroll_flags.follow_pc_y = 0 < (value & 1 << 1);
                    layer_ref.animation_style.scroll_flags.autoscroll_x = 0 < (value & 1 << 2);
                    layer_ref.animation_style.scroll_flags.autoscroll_y = 0 < (value & 1 << 3);
                    layer_ref.animation_style.scroll_flags.align_with_water_lvl = 0 < (value & 1 << 4);
                    layer_ref.animation_style.scroll_flags.draw_above_foreground = 0 < (value & 1 << 5);
                    layer_ref.animation_style.scroll_flags.random_offset_x = 0 < (value & 1 << 6);
                    layer_ref.animation_style.scroll_flags.random_offset_y = 0 < (value & 1 << 7);
                    layer_ref.animation_style.scroll_flags.lock_to_x_axis = 0 < (value & 1 << 8);
                    layer_ref.animation_style.scroll_flags.lock_to_y_axis = 0 < (value & 1 << 9);
                    layer_ref.animation_style.scroll_flags.randomize_all_parameters = 0 < (value & 1 << 10);
                    layer_ref.animation_style.scroll_flags.add_screen_width = 0 < (value & 1 << 11);
                    layer_ref.animation_style.scroll_flags.add_screen_height = 0 < (value & 1 << 12);
                    layer_ref.animation_style.scroll_flags.relative_to_pillarbox= 0 < (value & 1 << 13);
                    layer_ref.animation_style.scroll_flags.relative_to_letterbox = 0 < (value & 1 << 14);
                }
                //invalid parameter: do nothing
                _ => {}
            }

        }
        
        Ok(())
    }

    //write self to ./data/bkg/path.json (filesystem only allows writing to the user directory)
    fn write_out(&self, path: &String) -> GameResult {
        

        //path format: DiscoVision
        //needed format: bkg/DiscoVision.json (use PathBuf to do this)

        let mut path_buf = PathBuf::from("bkg");
        path_buf.push(path.clone() + ".json");


        let bkg_file = filesystem::get_writable_file(path_buf)?;

        //write out
        serde_json::to_writer_pretty(bkg_file, self)?;

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

    //cache original map values
    pub cache_background_path: String,
    pub cache_background_type: BackgroundType,
    pub cache_background_lighting: LightingMode,

}

impl Background {
    pub fn new() -> Self {
        Background {
            tick: 0,
            prev_tick: 0,
            bk_config: BkgConfig::default(),
            rng: Xoroshiro32PlusPlus::new(4873), //rando-starting number

            cache_background_path: String::new(),
            cache_background_type: BackgroundType::Black,
            cache_background_lighting: LightingMode::None,

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

        //cache old data
        self.cache_background_path = textures.background.clone();
        self.cache_background_type = stage.data.background_type.clone();
        self.cache_background_lighting = *lighting_mode;

        //if the config file is valid, load it in
        if let Ok(config) = BkgConfig::load(ctx, path) {
            textures.background = config.bmp_filename.clone(); //we need to check the validity of the filename here to stop the program from crashing, but this not essential for function
            self.bk_config = config;
            stage.data.background_type = BackgroundType::Custom;
            *lighting_mode = LightingMode::from(self.bk_config.lighting_mode);
        }

        //init random parameters if configured
        for layer in self.bk_config.layers.as_mut_slice() {
            if layer.animation_style.scroll_flags.randomize_all_parameters {
                layer.draw_corner_offset_x = self.rng.range(0..(layer.draw_corner_offset_x as i32)) as f32;
                layer.draw_corner_offset_y = self.rng.range(0..(layer.draw_corner_offset_y as i32)) as f32;
                layer.animation_style.animation_speed = self.rng.range((layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32 * 2)) as u32;
                layer.animation_style.ani_wait = self.rng.range(0..layer.animation_style.animation_speed as i32) as u32;
                layer.animation_style.frame_start = self.rng.range(0..layer.animation_style.frame_count as i32) as u32;
            }
        }

        Ok(())

    }

    pub fn tick(
        &mut self,
        state: &mut SharedGameState,
        stage: &Stage,
        frame: &Frame,
    ) -> GameResult<()> {
        self.tick = self.tick.wrapping_add(1);



        //we need the map size so we can account for the letterboxing/pillarboxing
        let tile_size = match state.tile_size {
            TileSize::Tile16x16 => 16,
            TileSize::Tile8x8 => 8,
        };
        //size of the loaded stage in pixels
        let map_pxl_width = stage.map.width * tile_size;
        let map_pxl_height = stage.map.height * tile_size;
        //actual size of a single letterbox (left or right)/(top or bottom)
        let pilrbox_width = if state.canvas_size.0 > map_pxl_width as f32 {(state.canvas_size.0 - map_pxl_width as f32) / 2.0} else {0.0};
        let ltrbox_height = if state.canvas_size.1 > map_pxl_height as f32 {(state.canvas_size.1 - map_pxl_height as f32) / 2.0} else {0.0};

        //the new offsets that should be used 
        let pb_canvas_width = if state.canvas_size.0 > map_pxl_width as f32 {pilrbox_width + map_pxl_width as f32} else {state.canvas_size.0};
        let lb_canvas_height = if state.canvas_size.1 > map_pxl_width as f32 {ltrbox_height + map_pxl_height as f32} else {state.canvas_size.1};

        //map edges if letterboxes are taken into account
        let boxed_lim = Rect::new(pilrbox_width, ltrbox_height, pb_canvas_width, lb_canvas_height);
        //map edges relative to actual window size
        let windowed_lim = Rect::new(0.0, 0.0, state.canvas_size.0, state.canvas_size.1);




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
            

            ////advance location offsets

            //for ease of refrence
            let scroll_flags = &layer.animation_style.scroll_flags;

            //reset frame offsets so background location resets with no-flag conditions
            let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);
            let scale = state.scale;

            //handle edge relativity
            if scroll_flags.relative_to_pillarbox {
                (layer.edge_coords.left, layer.edge_coords.right) = (boxed_lim.left, boxed_lim.right);
            } else {
                (layer.edge_coords.left, layer.edge_coords.right) = (windowed_lim.left, windowed_lim.right);
            }

            if scroll_flags.relative_to_letterbox {
                (layer.edge_coords.top, layer.edge_coords.bottom) = (boxed_lim.top, boxed_lim.bottom);
            } else {
                (layer.edge_coords.top, layer.edge_coords.bottom) = (windowed_lim.top, windowed_lim.bottom);
            }


            layer.frame_x_offset = layer.edge_coords.left;
            layer.frame_y_offset = layer.edge_coords.top;

            if scroll_flags.follow_pc_x {
                layer.frame_x_offset -= (frame_x as f32 * layer.animation_style.follow_speed_x * scale).floor() / scale;
            }
            if scroll_flags.lock_to_y_axis {
                layer.frame_x_offset -= frame_x as f32;
            }
            if scroll_flags.add_screen_width {
                layer.frame_x_offset += layer.edge_coords.width();
            }

            if scroll_flags.align_with_water_lvl {
                layer.frame_y_offset += (state.water_level / 0x200) as f32 - frame_y;
            }
            if scroll_flags.follow_pc_y {
                layer.frame_y_offset -= (frame_y as f32 * layer.animation_style.follow_speed_y * scale).floor() / scale;
            }
            if scroll_flags.lock_to_x_axis {
                layer.frame_y_offset -= frame_y as f32;
            }
            if scroll_flags.add_screen_height {
                layer.frame_y_offset += layer.edge_coords.height()
            }



            //if-chain for each flag type:

            //animate autoscrolling (looping is handled in the conditions below)
            if scroll_flags.autoscroll_x {layer.layer_x_value -= layer.animation_style.autoscroll_speed_x;}
            if scroll_flags.autoscroll_y {layer.layer_y_value -= layer.animation_style.autoscroll_speed_y;}

            let full_width = (layer.bmp_width + layer.draw_repeat_gap_x) as f32;
            let full_height = (layer.bmp_height + layer.draw_repeat_gap_y) as f32;

            //looping for infinite-width tilesets: (problematic method)
            // if layer.draw_repeat_x == 0 {
            //     //offset just behind left wall and shift in
            //     if layer.layer_x_value  + layer.frame_x_offset + layer.draw_corner_offset_x > 0.0 {
            //         //see how many multiples of the full width we need to shift over (simple final-initial calculation), mainly to counteract large draw_corner_offset values
            //         let offset_dist = (0.0 - layer.draw_corner_offset_x) - (layer.layer_x_value  + layer.frame_x_offset);
            //         let time_count = (offset_dist / full_width).floor();
            //         layer.layer_x_value += time_count * full_width;
            //     }
            //     else if layer.layer_x_value + layer.frame_x_offset + layer.draw_corner_offset_x < 0.0 - full_width {
            //         let offset_dist = (0.0 - layer.draw_corner_offset_x - full_width) - (layer.layer_x_value + layer.frame_x_offset);
            //         let time_count = (offset_dist / full_width).floor();
            //         layer.layer_x_value += full_width * time_count;
            //     }
            // }

            //looping for infinite-width tilesets: (note: it takes several cycles to get this within range if corner offsets are massve: that's what the code above tried to solve, but it introduces other problems I don't want to deal with)
            if layer.draw_repeat_x == 0 {
                //offset just behind left wall, and shift in
                if layer.layer_x_value + layer.frame_x_offset + layer.draw_corner_offset_x > layer.edge_coords.left { //0.0 {                    
                    layer.layer_x_value -= full_width;
                }
                else if layer.layer_x_value + layer.frame_x_offset + layer.draw_corner_offset_x < layer.edge_coords.left - full_width {
                    layer.layer_x_value += full_width;
                }
            }
            //if the bitmap is set to repeat and the bitmap count is finite, handle looping it
            else if scroll_flags.autoscroll_x {
                //layer.layer_x_value -= layer.animation_style.scroll_speed_x;

                //if layer's right corner offset by the times it should be draw is less than 0, shift it over by one bitmap width and window width
                if layer.layer_x_value + layer.draw_corner_offset_x +
                (full_width * layer.draw_repeat_x as f32) +
                layer.frame_x_offset < layer.edge_coords.left {

                    //move whole layerset to the right side of the viewspace
                    layer.layer_x_value += (full_width * layer.draw_repeat_x as f32) + layer.edge_coords.width();

                    //if y movement is randomized, add a random value +- animation speed to the y position
                    if scroll_flags.random_offset_y {
                        layer.layer_y_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
                    }
                }

                //if layer's left corner is beyond the window width
                else if layer.layer_x_value + layer.draw_corner_offset_x +
                layer.frame_x_offset > layer.edge_coords.right {

                    //move whole layer set to the left side of the viewspace
                    layer.layer_x_value -= (full_width * layer.draw_repeat_x as f32) + layer.edge_coords.width();

                    if scroll_flags.random_offset_y {
                        layer.layer_y_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
                    }

                }
            }


            //same as above but for y
            if layer.draw_repeat_y == 0 {
                //offset just behind left wall, and shift in
                if layer.layer_y_value + layer.frame_y_offset + layer.draw_corner_offset_y >  layer.edge_coords.top { //0.0 {
                    layer.layer_y_value -= full_height;
                }
                else if layer.layer_y_value + layer.frame_y_offset + layer.draw_corner_offset_y < layer.edge_coords.top - full_height {
                    layer.layer_y_value += full_height;
                }
            }
            else if scroll_flags.autoscroll_y {
                //layer.layer_y_value -= layer.animation_style.scroll_speed_y;

                //if layer's top corner offset by the times it should be draw is less than 0, shift it down by one bitmap height and window height
                if layer.layer_y_value + layer.draw_corner_offset_y +
                (full_height * layer.draw_repeat_y as f32) +
                layer.frame_y_offset < layer.edge_coords.top {

                    //move whole layerset to the bottom of the viewspace
                    layer.layer_y_value += (full_height * layer.draw_repeat_y as f32) + layer.edge_coords.height();

                    //if y movement is randomized, add a random value +- animation speed to the x position
                    if scroll_flags.random_offset_x {
                        layer.layer_x_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
                    }
                }

                //if layer's bottom corner is beyond the window height
                else if layer.layer_y_value + layer.draw_corner_offset_y +
                layer.frame_y_offset > layer.edge_coords.bottom {

                    //move whole layer set to the bottom of the viewspace
                    layer.layer_y_value -= (full_height * layer.draw_repeat_y as f32) + layer.edge_coords.height();

                    if scroll_flags.random_offset_x {
                        layer.layer_x_value += self.rng.range(-(layer.animation_style.animation_speed as i32)..(layer.animation_style.animation_speed as i32)) as f32;
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
                if !is_front {graphics::clear(ctx, stage.data.background_color);}

                for layer in self.bk_config.layers.as_slice() {
                    if !layer.layer_enabled ||
                    (is_front && !layer.animation_style.scroll_flags.draw_above_foreground) || //layer is not flagged to draw above the foreground
                    (!is_front && layer.animation_style.scroll_flags.draw_above_foreground) ||
                    (layer.bmp_height == 0 || layer.bmp_width == 0) //skip 0-width rects
                    {continue;}


                    let (xoff, yoff) = (layer.bmp_x_offset + layer.bmp_width * layer.animation_style.frame_start, layer.bmp_y_offset);
                    let layer_rc = Rect::new(
                        xoff as u16,
                        yoff as u16,
                        (xoff + layer.bmp_width) as u16,
                        (yoff + layer.bmp_height) as u16);
                    


                    //not sure if we need these to be descrete
                    let (rep_x, rep_y) = (layer.draw_repeat_x, layer.draw_repeat_y);

                    //start here and draw bitmap, stepping each time by these coords
                    let mut y_off = layer.layer_y_value as f32;

                    //apply misc. camera/water offsets
                    y_off += layer.frame_y_offset;

                    //apply map corner offset
                    y_off += layer.draw_corner_offset_y;

                    let mut y = 0;
                    while (y < rep_y || rep_y == 0) && y_off < layer.edge_coords.bottom { //(state.canvas_size.1 as f32) * 16.0 {
                        
                        //need this to reset for each layer
                        let mut x_off = layer.layer_x_value as f32;

                        //apply map corner offset
                        x_off += layer.draw_corner_offset_x;

                        //apply camera offset
                        x_off += layer.frame_x_offset;


                        //while loop (x-axis)
                        let mut x = 0;
                        while (x < rep_x || rep_x == 0) && x_off < layer.edge_coords.right { //(state.canvas_size.0 as f32) * 16.0 {

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
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
