use std::any::Any;
use std::borrow::Borrow;
use std::io;

use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::settings::Settings;
use crate::game::LaunchOptions;
use crate::sound::pixtone::PixToneParameters;


pub enum SongFormat {
    Organya,
    #[cfg(feature = "ogg-playback")]
    OggSinglePart,
    #[cfg(feature = "ogg-playback")]
    OggMultiPart,
}

#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InterpolationMode {
    Nearest,
    Linear,
    Cosine,
    Cubic,
    Polyphase,
}

pub trait SoundManager {
    fn reload(&mut self) -> GameResult<()>;

    fn pause(&mut self);

    fn resume(&mut self);

    fn play_sfx(&mut self, id: u8);

    fn loop_sfx(&self, id: u8);

    fn loop_sfx_freq(&mut self, id: u8, freq: f32);

    fn stop_sfx(&mut self, id: u8);

    fn set_org_interpolation(&mut self, interpolation: InterpolationMode);

    fn set_song_volume(&mut self, volume: f32);

    fn set_sfx_volume(&mut self, volume: f32);

    fn set_sfx_samples(&mut self, id: u8, data: Vec<i16>);

    fn reload_songs(&mut self, constants: &EngineConstants, settings: &Settings, ctx: &mut Context) -> GameResult;

    fn play_song(
        &mut self,
        song_id: usize,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult;

    fn save_state(&mut self) -> GameResult;

    fn restore_state(&mut self) -> GameResult;

    fn set_speed(&mut self, speed: f32) -> GameResult;

    fn current_song(&self) -> usize;

    fn set_sample_params_from_file(&mut self, id: u8, data: Box<dyn io::Read>) -> GameResult;

    fn set_sample_params(&mut self, id: u8, params: PixToneParameters) -> GameResult;

    fn load_custom_sound_effects(&mut self, ctx: &mut Context, roots: &Vec<String>) -> GameResult;

    fn as_any(&self) -> &dyn Any;
}

#[allow(unreachable_code)]
pub fn init_sound_backend(ctx: &mut Context, launch_options: &mut LaunchOptions) -> GameResult<Box<dyn SoundManager>> {

    if ctx.headless {
        return crate::sound::backend_null::SoundManagerNull::new(ctx);
    }

    #[cfg(feature = "backend-libretro")]
    {
        return crate::sound::backend_libretro::SoundManagerLibretro::new(ctx, &mut launch_options.audio_config);
    }

    #[cfg(feature = "audio-cpal")]
    {
        return crate::sound::backend_cpal::SoundManagerCpal::new(ctx);
    }


    return crate::sound::backend_null::SoundManagerNull::new(ctx);


}



