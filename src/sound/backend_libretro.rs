//this backend will output audio to a raw buffer

use std::any::Any;
use std::io;
use std::io::{BufRead, BufReader, Lines};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

#[cfg(feature = "ogg-playback")]
use lewton::inside_ogg::OggStreamReader;
use num_traits::clamp;
use serde::de::value::I8Deserializer;

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


//default retroarch sample rate:
//44100 samples per second
//i16 sized samples

/// Format that each sample has.
#[cfg_attr(target_os = "emscripten", wasm_bindgen)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SampleFormat {
    /// `i8` with a valid range of 'u8::MIN..=u8::MAX' with `0` being the origin
    I8,

    /// `i16` with a valid range of 'u16::MIN..=u16::MAX' with `0` being the origin
    I16,

    // /// `I24` with a valid range of '-(1 << 23)..(1 << 23)' with `0` being the origin
    // I24,
    /// `i32` with a valid range of 'u32::MIN..=u32::MAX' with `0` being the origin
    I32,

    // /// `I24` with a valid range of '-(1 << 47)..(1 << 47)' with `0` being the origin
    // I48,
    /// `i64` with a valid range of 'u64::MIN..=u64::MAX' with `0` being the origin
    I64,

    /// `u8` with a valid range of 'u8::MIN..=u8::MAX' with `1 << 7 == 128` being the origin
    U8,

    /// `u16` with a valid range of 'u16::MIN..=u16::MAX' with `1 << 15 == 32768` being the origin
    U16,

    // /// `U24` with a valid range of '0..16777216' with `1 << 23 == 8388608` being the origin
    // U24,
    /// `u32` with a valid range of 'u32::MIN..=u32::MAX' with `1 << 31` being the origin
    U32,

    // /// `U48` with a valid range of '0..(1 << 48)' with `1 << 47` being the origin
    // U48,
    /// `u64` with a valid range of 'u64::MIN..=u64::MAX' with `1 << 63` being the origin
    U64,

    /// `f32` with a valid range of `-1.0..1.0` with `0.0` being the origin
    F32,

    /// `f64` with a valid range of -1.0..1.0 with 0.0 being the origin
    F64,
}

pub struct OutputBufConfig<'a> {

    pub sample_rate: f64, //sample format is always i16
    pub channel_count: u16,

    pub runner_out: &'a mut Option<Runner>

}


//just the bits that the runner needs to start, minus the runner pointer.
pub(in crate::sound) struct RunnerConfig {
    pub sample_rate: f64, //sample type is always i16
    pub channel_count: u16,
}



pub struct SoundManagerLibretro {
    soundbank: Option<SoundBank>,
    tx: Sender<PlaybackMessage>,
    prev_song_id: usize,
    current_song_id: usize,
    no_audio: bool,
    load_failed: bool,
    
    //stream: Option<Stream>,
    //
}


impl SoundManagerLibretro {

    pub fn new(ctx: &mut Context, config: &mut OutputBufConfig) -> GameResult<Box<dyn SoundManager>> {

        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();

        let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/organya-wavetable-doukutsu.bin")?)?;
        Ok(Box::new(SoundManagerLibretro::bootstrap(&bnk, tx, rx, config)?))
    }

    fn bootstrap(
        soundbank: &SoundBank,
        tx: Sender<PlaybackMessage>,
        rx: Receiver<PlaybackMessage>,
        config: &mut OutputBufConfig,
    ) -> GameResult<SoundManagerLibretro> {

        let mut sound_manager = SoundManagerLibretro {
            soundbank: Some(soundbank.to_owned()),
            tx,
            prev_song_id: 0,
            current_song_id: 0,
            no_audio: false,
            load_failed: false,
        };

        let runner_config = RunnerConfig {
            sample_rate: config.sample_rate,
            channel_count: config.channel_count,
        };

        let runner = Runner::init(rx, soundbank.to_owned(), runner_config);
        if let Err(runner) = &runner {
            log::error!("Error initializing audio: {}", runner);
        }

        let runner = runner.ok();

        //pass runner object out to the core
        *config.runner_out = runner;

        Ok(sound_manager)

    }

    fn send(&mut self, message: PlaybackMessage) -> GameResult<()> {
        if self.no_audio {
            return Ok(());
        }

        if self.tx.send(message).is_err() {
            if !self.load_failed {
                log::error!("Error sending message to audio thread. Press Ctrl + F3 to reload sound manager.");
                self.reload()?;
            }
        }

        Ok(())
    }


}


impl SoundManager for SoundManagerLibretro {
    fn reload(&mut self) -> GameResult<()> {

        //todo: implement this!
        log::info!("Reloading sound manager.");

        // let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();
        // let soundbank = self.soundbank.take().unwrap();
        // *self = SoundManagerLibretro::bootstrap(&soundbank, tx, rx)?;

        Ok(())
    }

    //the frontend will handle pausing and resuming
    fn pause(&mut self) {
        // if let Some(stream) = &mut self.stream {
        //     let _ = stream.pause();
        // }
    }

    fn resume(&mut self) {
        // if let Some(stream) = &mut self.stream {
        //     let _ = stream.play();
        // }
    }

    fn play_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }

        self.send(PlaybackMessage::PlaySample(id)).unwrap();
    }

    fn loop_sfx(&self, id: u8) {
        if self.no_audio {
            return;
        }

        self.tx.send(PlaybackMessage::LoopSample(id)).unwrap();
    }

    fn loop_sfx_freq(&mut self, id: u8, freq: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::LoopSampleFreq(id, freq)).unwrap();
    }

    fn stop_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::StopSample(id)).unwrap();
    }

    fn set_org_interpolation(&mut self, interpolation: InterpolationMode) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetOrgInterpolation(interpolation)).unwrap();
    }

    fn set_song_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSongVolume(volume.powf(3.0))).unwrap();
    }

    fn set_sfx_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleVolume(volume.powf(3.0))).unwrap();
    }

    fn set_sfx_samples(&mut self, id: u8, data: Vec<i16>) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleData(id, data)).unwrap();
    }

    fn reload_songs(&mut self, constants: &EngineConstants, settings: &Settings, ctx: &mut Context) -> GameResult {
        let prev_song = self.prev_song_id;
        let current_song = self.current_song_id;

        self.play_song(0, constants, settings, ctx, false)?;
        self.play_song(prev_song, constants, settings, ctx, false)?;
        self.save_state()?;
        self.play_song(current_song, constants, settings, ctx, false)?;

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
        if self.current_song_id == song_id || self.no_audio {
            return Ok(());
        }

        if song_id == 0 {
            log::info!("Stopping BGM");

            self.prev_song_id = self.current_song_id;
            self.current_song_id = 0;

            self.send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation)).unwrap();
            self.send(PlaybackMessage::SaveState).unwrap();

            if fadeout {
                self.send(PlaybackMessage::FadeoutSong).unwrap();
            } else {
                self.send(PlaybackMessage::Stop).unwrap();
            }
        } else if let Some(song_name) = constants.music_table.get(song_id) {
            let mut paths = constants.organya_paths.clone();

            paths.insert(0, "/Soundtracks/".to_owned() + &settings.soundtrack + "/");

            if let Some(soundtrack) =
            constants.soundtracks.iter().find(|s| s.available && s.name == settings.soundtrack)
            {
                paths.insert(0, soundtrack.path.clone());
            }

            let songs_paths = paths.iter().map(|prefix| {
                [
                        #[cfg(feature = "ogg-playback")]
                    (
                        SongFormat::OggMultiPart,
                        vec![format!("{}{}_intro.ogg", prefix, song_name), format!("{}{}_loop.ogg", prefix, song_name)],
                    ),
                        #[cfg(feature = "ogg-playback")]
                    (SongFormat::OggSinglePart, vec![format!("{}{}.ogg", prefix, song_name)]),
                    (SongFormat::Organya, vec![format!("{}{}.org", prefix, song_name)]),
                ]
            });

            for songs in songs_paths {
                for (format, paths) in
                songs.iter().filter(|(_, paths)| paths.iter().all(|path| filesystem::exists(ctx, path)))
                {
                    match format {
                        SongFormat::Organya => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(organya::Song::load_from) {
                                Ok(Ok(org)) => {
                                    log::info!("Playing Organya BGM: {} {}", song_id, path);

                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    let _ = self
                                        .send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation))
                                        .unwrap();
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOrganyaSong(Box::new(org))).unwrap();

                                    return Ok(());
                                }
                                Ok(Err(err)) | Err(err) => {
                                    log::warn!("Failed to load Organya BGM {}: {}", song_id, err);
                                }
                            }
                        }
                        #[cfg(feature = "ogg-playback")]
                        SongFormat::OggSinglePart => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(|f| {
                                OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                            }) {
                                Ok(Ok(song)) => {
                                    log::info!("Playing single part Ogg BGM: {} {}", song_id, path);

                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOggSongSinglePart(Box::new(song))).unwrap();

                                    return Ok(());
                                }
                                Ok(Err(err)) | Err(err) => {
                                    log::warn!("Failed to load single part Ogg BGM {}: {}", song_id, err);
                                }
                            }
                        }
                        #[cfg(feature = "ogg-playback")]
                        SongFormat::OggMultiPart => {
                            // we're sure that there are two elements
                            let path_intro = unsafe { paths.get_unchecked(0) };
                            let path_loop = unsafe { paths.get_unchecked(1) };

                            match (
                                filesystem::open(ctx, path_intro).map(|f| {
                                    OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                                }),
                                filesystem::open(ctx, path_loop).map(|f| {
                                    OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                                }),
                            ) {
                                (Ok(Ok(song_intro)), Ok(Ok(song_loop))) => {
                                    log::info!(
                                        "Playing multi part Ogg BGM: {} {} + {}",
                                        song_id,
                                        path_intro,
                                        path_loop
                                    );

                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOggSongMultiPart(
                                        Box::new(song_intro),
                                        Box::new(song_loop),
                                    ))
                                        .unwrap();

                                    return Ok(());
                                }
                                (Ok(Err(err)), _) | (Err(err), _) | (_, Ok(Err(err))) | (_, Err(err)) => {
                                    log::warn!("Failed to load multi part Ogg BGM {}: {}", song_id, err);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn save_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SaveState).unwrap();
        self.prev_song_id = self.current_song_id;

        Ok(())
    }

    fn restore_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::RestoreState).unwrap();
        self.current_song_id = self.prev_song_id;

        Ok(())
    }

    fn set_speed(&mut self, speed: f32) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        if speed <= 0.0 {
            return Err(InvalidValue("Speed must be bigger than 0.0!".to_owned()));
        }

        self.send(PlaybackMessage::SetSpeed(speed)).unwrap();

        Ok(())
    }

    fn current_song(&self) -> usize {
        self.current_song_id
    }

    
    fn set_sample_params_from_file(&mut self, id: u8, data: Box<dyn io::Read>) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        let mut reader = BufReader::new(data).lines();
        let mut params = PixToneParameters::empty();

        fn next_string<T: FromStr>(reader: &mut Lines<BufReader<Box::<dyn io::Read>>>) -> GameResult<T> {
            while let Some(Ok(str)) = reader.next() {
                let str = str.trim();
                if str.is_empty() || str.starts_with('#') {
                    continue;
                }

                let mut splits = str.split(':');

                let _ = splits.next();
                if let Some(str) = splits.next() {
                    return str.trim().parse::<T>().map_err(|_| {
                        GameError::ParseError("failed to parse the value as specified type.".to_string())
                    });
                } else {
                    break;
                }
            }

            Err(GameError::ParseError("unexpected end.".to_string()))
        }

        for channel in &mut params.channels {
            channel.enabled = next_string::<u8>(&mut reader)? != 0;
            channel.length = next_string::<u32>(&mut reader)?;

            channel.carrier.waveform_type = next_string::<u8>(&mut reader)?;
            channel.carrier.pitch = next_string::<f32>(&mut reader)?;
            channel.carrier.level = next_string::<i32>(&mut reader)?;
            channel.carrier.offset = next_string::<i32>(&mut reader)?;

            channel.frequency.waveform_type = next_string::<u8>(&mut reader)?;
            channel.frequency.pitch = next_string::<f32>(&mut reader)?;
            channel.frequency.level = next_string::<i32>(&mut reader)?;
            channel.frequency.offset = next_string::<i32>(&mut reader)?;

            channel.amplitude.waveform_type = next_string::<u8>(&mut reader)?;
            channel.amplitude.pitch = next_string::<f32>(&mut reader)?;
            channel.amplitude.level = next_string::<i32>(&mut reader)?;
            channel.amplitude.offset = next_string::<i32>(&mut reader)?;

            channel.envelope.initial = next_string::<i32>(&mut reader)?;
            channel.envelope.time_a = next_string::<i32>(&mut reader)?;
            channel.envelope.value_a = next_string::<i32>(&mut reader)?;
            channel.envelope.time_b = next_string::<i32>(&mut reader)?;
            channel.envelope.value_b = next_string::<i32>(&mut reader)?;
            channel.envelope.time_c = next_string::<i32>(&mut reader)?;
            channel.envelope.value_c = next_string::<i32>(&mut reader)?;
        }

        self.set_sample_params(id, params)
    }

    fn set_sample_params(&mut self, id: u8, params: PixToneParameters) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SetSampleParams(id, params)).unwrap();

        Ok(())
    }

    fn load_custom_sound_effects(&mut self, ctx: &mut Context, roots: &Vec<String>) -> GameResult {
        for path in roots.iter().rev() {
            let wavs = filesystem::read_dir(ctx, [path, "sfx/"].join(""))?
                .filter(|f| f.to_string_lossy().to_lowercase().ends_with(".wav"));

            for filename in wavs {
                if let Ok(mut file) = filesystem::open(ctx, &filename) {
                    let wav = wav::WavSample::read_from(&mut file)?;
                    let id = filename
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .parse::<u8>()
                        .unwrap_or(0);
                    if id == 0 {
                        continue;
                    }
                    let step = (wav.format.channels * 2) as usize;
                    let data = wav
                        .data
                        .chunks_exact(2)
                        .into_iter()
                        .step_by(step)
                        .map(|a| i16::from_ne_bytes([a[0], a[1]]))
                        .collect();

                    self.set_sfx_samples(id, data);
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}



//todo: (maybe) put these in "backend"?
pub(in crate::sound) enum PlaybackMessage {
    Stop,
    PlayOrganyaSong(Box<Song>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongSinglePart(Box<OggStreamReader<File>>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongMultiPart(Box<OggStreamReader<File>>, Box<OggStreamReader<File>>),
    PlaySample(u8),
    LoopSample(u8),
    LoopSampleFreq(u8, f32),
    StopSample(u8),
    SetSpeed(f32),
    SetSongVolume(f32),
    SetSampleVolume(f32),
    FadeoutSong,
    SaveState,
    RestoreState,
    SetSampleParams(u8, PixToneParameters),
    SetOrgInterpolation(InterpolationMode),
    SetSampleData(u8, Vec<i16>),
}

#[derive(PartialEq, Eq)]
enum PlaybackState {
    Stopped,
    PlayingOrg,
    #[cfg(feature = "ogg-playback")]
    PlayingOgg,
}

enum PlaybackStateType {
    None,
    Organya(SavedOrganyaPlaybackState),
    #[cfg(feature = "ogg-playback")]
    Ogg(SavedOggPlaybackState),
}

impl Default for PlaybackStateType {
    fn default() -> Self {
        Self::None
    }
}

///////////////////////


pub struct Runner {
    sample_rate: f32,
    channels: usize,
    state: PlaybackState,
    saved_state: PlaybackStateType,
    speed: f32,
    org_engine: Box<OrgPlaybackEngine>,
    #[cfg(feature = "ogg-playback")]
    ogg_engine: Box<OggPlaybackEngine>,
    pixtone: Box<PixTonePlayback>,

    buf_size: usize,
    bgm_buf: Vec<u16>,
    pxt_buf: Vec<u16>,
    bgm_index: usize,
    pxt_index: usize,
    samples: usize,
    bgm_vol: f32,
    bgm_vol_saved: f32,
    sfx_vol: f32,
    bgm_fadeout: bool,
    pub data: Vec<i16>,

    bank: SoundBank,
    rx: Receiver<PlaybackMessage>,
}

impl Runner {

    pub(in crate::sound) fn init(
        rx: Receiver<PlaybackMessage>,
        bank: SoundBank,
        config: RunnerConfig,
    ) -> GameResult<Runner> {

        let sample_rate = config.sample_rate as f32;
        let channels = config.channel_count as usize;
        let mut state = PlaybackState::Stopped;
        let mut saved_state: PlaybackStateType = PlaybackStateType::None;
        let mut speed = 1.0;
        let mut org_engine = Box::new(OrgPlaybackEngine::new());
        #[cfg(feature = "ogg-playback")]
        let mut ogg_engine = Box::new(OggPlaybackEngine::new());
        let mut pixtone = Box::new(PixTonePlayback::new());
        pixtone.create_samples();
    
        log::info!("Audio format: {} {}", sample_rate, channels);
        org_engine.set_sample_rate(sample_rate as usize);
        #[cfg(feature = "ogg-playback")]
        {
            org_engine.loops = usize::MAX;
            ogg_engine.set_sample_rate(sample_rate as usize);
        }
    
        let buf_size = sample_rate as usize * 10 / 1000;
        let mut bgm_buf = vec![0x8080; buf_size * 2];
        let mut pxt_buf = vec![0x8000; buf_size];
        let mut bgm_index = 0;
        let mut pxt_index = 0;
        let mut samples = 0;
        let mut bgm_vol = 1.0_f32;
        let mut bgm_vol_saved = 1.0_f32;
        let mut sfx_vol = 1.0_f32;
        let mut bgm_fadeout = false;
        pixtone.mix(&mut pxt_buf, sample_rate);

        //this is how many samples will be filled each time the run() funcion is called
        let mut data: Vec<i16> = vec![0; (buf_size * channels)];


        Ok(Runner {
            sample_rate: sample_rate,
            channels: channels,
            state: state,
            saved_state: saved_state,
            speed: speed,
            org_engine: org_engine,
            #[cfg(feature = "ogg-playback")]
            ogg_engine: ogg_engine,
            pixtone: pixtone,
            buf_size: buf_size,
            bgm_buf: bgm_buf,
            pxt_buf: pxt_buf,
            bgm_index: bgm_index,
            pxt_index: pxt_index,
            samples: samples,
            bgm_vol: bgm_vol,
            bgm_vol_saved: bgm_vol_saved,
            sfx_vol: sfx_vol,
            bgm_fadeout: bgm_fadeout,
            data: data,
            bank: bank,
            rx: rx,
        })

        
    }

    pub fn run(&mut self){
        loop {
            if self.bgm_fadeout && self.bgm_vol > 0.0 {
                self.bgm_vol -= 0.02;
            }

            if self.bgm_vol < 0.0 {
                self.bgm_vol = 0.0;
            }

            match self.rx.try_recv() {
                Ok(PlaybackMessage::PlayOrganyaSong(song)) => {
                    if self.state == PlaybackState::Stopped {
                        self.saved_state = PlaybackStateType::None;
                    }

                    if self.bgm_fadeout {
                        self.bgm_fadeout = false;
                        self.bgm_vol = self.bgm_vol_saved;
                    }

                    self.org_engine.start_song(*song, &self.bank);

                    for i in &mut self.bgm_buf[0..self.samples] {
                        *i = 0x8000
                    }
                    self.samples = self.org_engine.render_to(&mut self.bgm_buf);
                    self.bgm_index = 0;

                    self.state = PlaybackState::PlayingOrg;
                }
                #[cfg(feature = "ogg-playback")]
                Ok(PlaybackMessage::PlayOggSongSinglePart(data)) => {
                    if self.state == PlaybackState::Stopped {
                        self.saved_state = PlaybackStateType::None;
                    }

                    if self.bgm_fadeout {
                        self.bgm_fadeout = false;
                        self.bgm_vol = self.bgm_vol_saved;
                    }

                    self.ogg_engine.start_single(data);

                    for i in &mut self.bgm_buf[0..self.samples] {
                        *i = 0x8000
                    }
                    self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                    self.bgm_index = 0;

                    self.state = PlaybackState::PlayingOgg;
                }
                #[cfg(feature = "ogg-playback")]
                Ok(PlaybackMessage::PlayOggSongMultiPart(data_intro, data_loop)) => {
                    if self.state == PlaybackState::Stopped {
                        self.saved_state = PlaybackStateType::None;
                    }

                    if self.bgm_fadeout {
                        self.bgm_fadeout = false;
                        self.bgm_vol = self.bgm_vol_saved;
                    }

                    self.ogg_engine.start_multi(data_intro, data_loop);

                    for i in &mut self.bgm_buf[0..self.samples] {
                        *i = 0x8000
                    }
                    self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                    self.bgm_index = 0;

                    self.state = PlaybackState::PlayingOgg;
                }
                Ok(PlaybackMessage::PlaySample(id)) => {
                    self.pixtone.play_sfx(id);
                }

                Ok(PlaybackMessage::LoopSample(id)) => {
                    self.pixtone.loop_sfx(id);
                }
                Ok(PlaybackMessage::LoopSampleFreq(id, freq)) => {
                    self.pixtone.loop_sfx_freq(id, freq);
                }
                Ok(PlaybackMessage::StopSample(id)) => {
                    self.pixtone.stop_sfx(id);
                }
                Ok(PlaybackMessage::Stop) => {
                    if self.state == PlaybackState::Stopped {
                        self.saved_state = PlaybackStateType::None;
                    }

                    self.state = PlaybackState::Stopped;
                }
                Ok(PlaybackMessage::SetSpeed(new_speed)) => {
                    assert!(new_speed > 0.0);
                    self.speed = new_speed;
                    #[cfg(feature = "ogg-playback")]
                    self.ogg_engine.set_sample_rate((self.sample_rate / new_speed) as usize);
                    self.org_engine.set_sample_rate((self.sample_rate / new_speed) as usize);
                }
                Ok(PlaybackMessage::SetSongVolume(new_volume)) => {
                    assert!(self.bgm_vol >= 0.0);
                    if self.bgm_fadeout {
                        self.bgm_vol_saved = new_volume;
                    } else {
                        self.bgm_vol = new_volume;
                    }
                }
                Ok(PlaybackMessage::SetSampleVolume(new_volume)) => {
                    assert!(self.sfx_vol >= 0.0);
                    self.sfx_vol = new_volume;
                }
                Ok(PlaybackMessage::FadeoutSong) => {
                    self.bgm_fadeout = true;
                    self.bgm_vol_saved = self.bgm_vol;
                }
                Ok(PlaybackMessage::SaveState) => {
                    self.saved_state = match self.state {
                        PlaybackState::Stopped => PlaybackStateType::None,
                        PlaybackState::PlayingOrg => PlaybackStateType::Organya(self.org_engine.get_state()),
                        #[cfg(feature = "ogg-playback")]
                        PlaybackState::PlayingOgg => PlaybackStateType::Ogg(self.ogg_engine.get_state()),
                    };
                }
                Ok(PlaybackMessage::RestoreState) => {
                    let saved_state_loc = std::mem::take(&mut self.saved_state);

                    match saved_state_loc {
                        PlaybackStateType::None => {
                            self.state = PlaybackState::Stopped;
                        }
                        PlaybackStateType::Organya(playback_state) => {
                            self.org_engine.set_state(playback_state, &self.bank);

                            if self.state == PlaybackState::Stopped {
                                self.org_engine.rewind();
                            }

                            for i in &mut self.bgm_buf[0..self.samples] {
                                *i = 0x8000
                            }
                            self.samples = self.org_engine.render_to(&mut self.bgm_buf);
                            self.bgm_index = 0;

                            if self.bgm_fadeout {
                                self.bgm_fadeout = false;
                                self.bgm_vol = self.bgm_vol_saved;
                            }

                            self.state = PlaybackState::PlayingOrg;
                        }
                        #[cfg(feature = "ogg-playback")]
                        PlaybackStateType::Ogg(playback_state) => {
                            self.ogg_engine.set_state(playback_state);

                            if self.state == PlaybackState::Stopped {
                                self.ogg_engine.rewind();
                            }

                            for i in &mut self.bgm_buf[0..self.samples] {
                                *i = 0x8000
                            }
                            self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                            self.bgm_index = 0;

                            if self.bgm_fadeout {
                                self.bgm_fadeout = false;
                                self.bgm_vol = self.bgm_vol_saved;
                            }

                            self.state = PlaybackState::PlayingOgg;
                        }
                    }
                }
                Ok(PlaybackMessage::SetSampleParams(id, params)) => {
                    self.pixtone.set_sample_parameters(id, params);
                }
                Ok(PlaybackMessage::SetOrgInterpolation(interpolation)) => {
                    self.org_engine.interpolation = interpolation;
                }
                Ok(PlaybackMessage::SetSampleData(id, data)) => {
                    self.pixtone.set_sample_data(id, data);
                }
                Err(_) => {
                    break;
                }
            }
        }

        for frame in self.data.chunks_mut(self.channels) {
            let (bgm_sample_l, bgm_sample_r): (u16, u16) = {
                if self.state == PlaybackState::Stopped {
                    (0x8000, 0x8000)
                } else if self.bgm_index < self.samples {
                    let samples = (self.bgm_buf[self.bgm_index], self.bgm_buf[self.bgm_index + 1]);
                    self.bgm_index += 2;
                    samples
                } else {
                    for i in &mut self.bgm_buf[0..self.samples] {
                        *i = 0x8000
                    }

                    match self.state {
                        PlaybackState::PlayingOrg => {
                            self.samples = self.org_engine.render_to(&mut self.bgm_buf);
                        }
                        #[cfg(feature = "ogg-playback")]
                        PlaybackState::PlayingOgg => {
                            self.samples = self.ogg_engine.render_to(&mut self.bgm_buf);
                        }
                        _ => unreachable!(),
                    }
                    self.bgm_index = 2;
                    (self.bgm_buf[0], self.bgm_buf[1])
                }
            };

            let pxt_sample: u16 = self.pxt_buf[self.pxt_index];

            if self.pxt_index < (self.pxt_buf.len() - 1) {
                self.pxt_index += 1;
            } else {
                self.pxt_index = 0;
                self.pxt_buf.fill(0x8000);
                self.pixtone.mix(&mut self.pxt_buf, self.sample_rate / self.speed);
            }

            if frame.len() >= 2 {
                let sample_l = clamp(
                    (((bgm_sample_l ^ 0x8000) as i16) as f32 * self.bgm_vol) as isize
                        + (((pxt_sample ^ 0x8000) as i16) as f32 * self.sfx_vol) as isize,
                    -0x7fff,
                    0x7fff,
                ) as i16; //u16
                //    ^ 0x8000;
                let sample_r = clamp(
                    (((bgm_sample_r ^ 0x8000) as i16) as f32 * self.bgm_vol) as isize
                        + (((pxt_sample ^ 0x8000) as i16) as f32 * self.sfx_vol) as isize,
                    -0x7fff,
                    0x7fff,
                ) as i16; //u16
                //    ^ 0x8000;

                frame[0] = sample_l; //T::from_sample(sample_l);
                frame[1] = sample_r; //T::from_sample(sample_r);
            } else {
                let sample = clamp(
                    ((((bgm_sample_l ^ 0x8000) as i16) + ((bgm_sample_r ^ 0x8000) as i16)) as f32 * self.bgm_vol / 2.0)
                        as isize
                        + (((pxt_sample ^ 0x8000) as i16) as f32 * self.sfx_vol) as isize,
                    -0x7fff,
                    0x7fff,
                ) as i16; //u16
                //    ^ 0x8000;

                frame[0] = sample; //T::from_sample(sample);
            }
        }
    }

}

