use std::io;
use std::sync::{ Arc, RwLock };

use crate::framework::error::GameResult;

use crate::sound::wav::WavFormat;

//use oxdz::Oxdz;
use libxmp_lite::LibXmpPlayer;


//pub(crate) struct TrackerPlaybackEngine<'a> {
pub(crate) struct TrackerPlaybackEngine {

    //player: Option<Arc<RwLock<Box<Oxdz<'a>>>>>, //Option<Box<Oxdz <- original

    player2: Option<Arc<RwLock<Box<LibXmpPlayer>>>>,

    output_format: WavFormat,

    position: u32,

    buffer: Vec<i16>,

    //intentionally blows out music by directly doing a signed-unsigned conversion
    blowout_mode: bool,
}

//pub struct SavedTrackerPlaybackState<'a> {
pub struct SavedTrackerPlaybackState {
    //player: Option<Arc<RwLock<Box<Oxdz<'a>>>>>,
    player2: Option<Arc<RwLock<Box<LibXmpPlayer>>>>,
    position: u32,

}


//impl<'a> TrackerPlaybackEngine<'a> {
impl TrackerPlaybackEngine {

    //pub fn new() -> TrackerPlaybackEngine<'a> {
    pub fn new() -> TrackerPlaybackEngine {
        TrackerPlaybackEngine {
            //player: None,
            player2: None,
            //curr_music: None,
            output_format: WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 },
            position: 0,
            buffer: Vec::with_capacity(4096),
            blowout_mode: false,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: usize) {
        self.output_format.sample_rate = sample_rate as u32;

        // if let Some(player) = self.player.as_mut() {
        //     let _ = player.write().unwrap().rate = sample_rate as u32;
        // }

        //we cannot change libxmp sample rate once we've started a song
    }

    pub fn get_state(&self) -> SavedTrackerPlaybackState { //<'a> {
        SavedTrackerPlaybackState {
            //player: self.player.clone(),
            player2: self.player2.clone(),
            position: self.position.clone(),
        }
    }

    pub fn set_state(&mut self, state: SavedTrackerPlaybackState) { //<'a>) {
        //self.player = state.player;
        self.player2 = state.player2;
        self.position = state.position;
    }

    pub fn set_blowout_mode(&mut self, mode: bool) {
        self.blowout_mode = mode;
    }

    //unused ATM
    // pub fn get_blowout_mode(&mut self) -> bool {
    //     self.blowout_modes
    // }

    pub fn start_song(&mut self, music: Box<Vec<u8>>) {

        // if let Ok(mut ll) = Oxdz::new(&music, self.output_format.sample_rate, "") {
        //     let _ = ll.set_interpolator("linear");
        //     self.player = Some(Arc::new(RwLock::new(Box::new(ll))));
        //     self.rewind();
        // }

        if let Ok(mut ll) = LibXmpPlayer::new(&music, self.output_format.sample_rate as i32) {
            ll.set_looping(true);
            self.player2 = Some(Arc::new(RwLock::new(Box::new(ll))));
        }


    }

    //send music back to start of file
    pub fn rewind(&mut self) {

        // if let Some(player) = &mut self.player {
        //     let _ = player.write().unwrap().set_position(0);
        // }

        if let Some(player) = &mut self.player2 {
            player.write().unwrap().rewind();
        }

    }

    pub fn render_to(&mut self, buf: &mut [u16]) -> usize {


        // if let Some(player) = &mut self.player {
        //     //direct cast, leads to wiiware-type sound
        //     //player.fill_buffer( unsafe { &mut *(buf as *mut [u16] as *mut [i16]) }, 0);

        //     let mut player = player.write().unwrap();

        //     self.buffer.resize(buf.len(), 0);
        //     player.fill_buffer( &mut self.buffer, 0);
        //     //todo: get "position" here (not extremely important though; its never used)
        //     self.buffer.drain(0..buf.len()).map(|n| n as u16 ^ 0x8000).zip(buf.iter_mut()).for_each(|(n, tgt)| *tgt = n);
        //     buf.len()

        // } else {0}


        if let Some(player) = &mut self.player2 {

            let mut player = player.write().unwrap();

            if self.blowout_mode {
            //if false {
                //direct cast, leads to wiiware-type sound
                player.get_buffer( unsafe { &mut *(buf as *mut [u16] as *mut [i16]) });
            } else {
                self.buffer.resize(buf.len(), 0);
                player.get_buffer( &mut self.buffer);

                //signed-unsigned-ness conversion
                self.buffer.drain(0..buf.len()).map(|n| n as u16 ^ 0x8000).zip(buf.iter_mut()).for_each(|(n, tgt)| *tgt = n);
            }

            buf.len()

        } else {0}

        

    }




    pub fn load_from<R: io::Read + io::Seek>(mut f: R) -> GameResult<Vec<u8>> {

        //get file size
        let file_len = f.seek(io::SeekFrom::End(0))?;
        let _ = f.seek(io::SeekFrom::Start(0));

        let mut mod_data = vec![0; file_len as usize];
        let _ = f.read(&mut mod_data);

        Ok(mod_data)

    }

}









