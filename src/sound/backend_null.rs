//this backend will output audio to a raw buffer

use std::any::Any;
use std::io;

#[cfg(feature = "ogg-playback")]
use lewton::inside_ogg::OggStreamReader;
use num_traits::clamp;

use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::error::GameError::{AudioError, InvalidValue};
use crate::framework::filesystem;
use crate::framework::filesystem::File;
use crate::game::settings::Settings;
#[cfg(feature = "ogg-playback")]
use crate::sound::ogg_playback::{OggPlaybackEngine, SavedOggPlaybackState};
use crate::sound::org_playback::{OrgPlaybackEngine, SavedOrganyaPlaybackState};
use crate::sound::organya;
use crate::sound::organya::Song;
use crate::sound::pixtone::{PixToneParameters, PixTonePlayback};
use crate::sound::{wave_bank, wave_bank::SoundBank, wav};
use crate::sound::backend::*;




pub struct SoundManagerNull {
    current_song_id: usize,
}


impl SoundManagerNull {

    pub fn new(ctx: &mut Context) -> GameResult<Box<dyn SoundManager>> {

        // if ctx.headless {
        //     log::info!("Running in headless mode, skipping initialization.");
        //     return Ok(Box::new(SoundManagerLibretro {
        //         current_song_id: 0
        //     }));
        // }
        //let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/organya-wavetable-doukutsu.bin")?)?;

        Ok(Box::new(SoundManagerNull{current_song_id: 0}))
    }

}


impl SoundManager for SoundManagerNull {
    fn reload(&mut self) -> GameResult<()>
    {
        log::info!("Skipping sound manager reload because audio is not enabled.");
        Ok(())
    }

    fn pause(&mut self) {}

    fn resume(&mut self) {}

    fn play_sfx(&mut self, id: u8) {}

    fn loop_sfx(&self, id: u8) {}

    fn loop_sfx_freq(&mut self, id: u8, freq: f32) {}

    fn stop_sfx(&mut self, id: u8) {}

    fn set_org_interpolation(&mut self, interpolation: InterpolationMode) {}

    fn set_song_volume(&mut self, volume: f32) {}

    fn set_sfx_volume(&mut self, volume: f32) {}

    fn set_sfx_samples(&mut self, id: u8, data: Vec<i16>) {}

    fn reload_songs(&mut self, constants: &EngineConstants, settings: &Settings, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn play_song(
        &mut self,
        song_id: usize,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult {
        self.current_song_id = song_id;
        Ok(())
    }

    fn save_state(&mut self) -> GameResult {
        Ok(())
    }

    fn restore_state(&mut self) -> GameResult {
        Ok(())
    }

    fn set_speed(&mut self, speed: f32) -> GameResult {
        Ok(())
    }

    fn current_song(&self) -> usize {
        self.current_song_id
    }

    fn set_sample_params_from_file(&mut self, id: u8, data: Box<dyn io::Read>) -> GameResult {
        Ok(())
    }

    fn set_sample_params(&mut self, id: u8, params: PixToneParameters) -> GameResult {
        Ok(())
    }

    fn load_custom_sound_effects(&mut self, ctx: &mut Context, roots: &Vec<String>) -> GameResult {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

