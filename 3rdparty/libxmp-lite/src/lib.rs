use std::ffi::{c_short, c_uint, c_void, CStr};
use std::io;
use std::io::ErrorKind;
use std::mem::{MaybeUninit, size_of};
use std::os::raw::{c_char, c_int, c_long, c_uchar};
use std::ptr::null_mut;
use std::marker::{Sync, Send};

//this was in Alula's template code. What is it?
//use crate::player::Player;

const XMP_NAME_SIZE: usize = 64;
const XMP_MAX_CHANNELS: usize = 64;
const XMP_MAX_MOD_LENGTH: usize = 256;

#[repr(C)]
struct XmpChannel {
    pan: c_int,
    vol: c_int,
    flg: c_int,
}

#[repr(C)]
struct XmpModule {
    name: [c_uchar; XMP_NAME_SIZE],
    r#type: [c_uchar; XMP_NAME_SIZE],
    pat: c_int,
    trk: c_int,
    chn: c_int,
    ins: c_int,
    smp: c_int,
    spd: c_int,
    bpm: c_int,
    len: c_int,
    rst: c_int,
    gvl: c_int,
    xxp: *mut *mut c_void,
    xxt: *mut *mut c_void,
    xxi: *mut *mut c_void,
    xxs: *mut *mut c_void,
    xxc: [XmpChannel; XMP_MAX_CHANNELS],
    xxo: [c_uchar; XMP_MAX_MOD_LENGTH],
}

#[repr(C)]
struct XmpModuleInfo {
    md5: [u8; 16],
    vol_base: c_int,
    r#mod: *mut XmpModule,
    comment: *mut c_char,
    num_sequences: c_int,
    seq_data: *mut *mut c_void,
}

#[repr(C)]
struct XmpEvent {
    note: c_uchar,   /* Note number (0 means no note) */
    ins: c_uchar,    /* Patch number */
    vol: c_uchar,    /* Volume (0 to basevol) */
    fxt: c_uchar,    /* Effect type */
    fxp: c_uchar,    /* Effect parameter */
    f2t: c_uchar,    /* Secondary effect type */
    f2p: c_uchar,    /* Secondary effect parameter */
    _flag: c_uchar,  /* Internal (reserved) flags */
}

#[repr(C)]
struct XmpChannelInfo {
    period: c_uint,      /* Sample period */
    position: c_uint,    /* Sample position */
    pitchbend: c_short,  /* Linear bend from base note*/
    note: c_uchar,       /* Current base note number */
    instrument: c_uchar, /* Current instrument number */
    sample: c_uchar,     /* Current sample number */
    volume: c_uchar,     /* Current volume */
    pan: c_uchar,        /* Current stereo pan */
    reserved: c_uchar,   /* Reserved */
    event: XmpEvent,     /* Current track event */
}

#[repr(C)]
struct XmpFrameInfo {
    pos: c_int,            /* Current position */
    pattern: c_int,        /* Current pattern */
    row: c_int,            /* Current row in pattern */
    num_rows: c_int,       /* Number of rows in current pattern */
    frame: c_int,          /* Current frame */
    speed: c_int,          /* Current replay speed */
    bpm: c_int,            /* Current bpm */
    time: c_int,           /* Current module time in ms */
    total_time: c_int,     /* Estimated replay time in ms*/
    frame_time: c_int,     /* Frame replay time in us */
    buffer: *mut c_void,   /* Pointer to sound buffer */
    buffer_size: c_int,    /* Used buffer size */
    total_size: c_int,     /* Total buffer size */
    volume: c_int,         /* Current master volume */
    loop_count: c_int,     /* Loop counter */
    virt_channels: c_int,  /* Number of virtual channels */
    virt_used: c_int,      /* Used virtual channels */
    sequence: c_int,       /* Current sequence */
    channel_info: [XmpChannelInfo; XMP_MAX_CHANNELS], /* Current channel information */
}

type XmpContext = *mut c_char;

// external library refrences
extern "C" {
    fn xmp_create_context() -> XmpContext;
    fn xmp_free_context(ctx: XmpContext);
    fn xmp_load_module_from_memory(ctx: XmpContext, data: *const c_void, length: c_long) -> c_int;
    fn xmp_release_module(ctx: XmpContext);
    fn xmp_test_module_from_memory(buffer: *mut c_void, length: c_long, info: *mut c_void) -> c_int;
    fn xmp_start_player(ctx: XmpContext, sample_rate: c_int, flags: c_int) -> c_int;
    fn xmp_play_buffer(ctx: XmpContext, buffer: *mut c_void, size: c_int, r#loop: c_int) -> c_int;
    fn xmp_get_module_info(ctx: XmpContext, mod_info: *mut XmpModuleInfo);
    fn xmp_get_frame_info(ctx: XmpContext, frame_info: *mut XmpFrameInfo);
    fn xmp_set_position(ctx: XmpContext, position: c_int) -> c_int;
    fn xmp_set_row(ctx: XmpContext, row: c_int) -> c_int;
    fn xmp_restart_module(ctx: XmpContext);
}

pub struct LibXmpPlayer {
    context: XmpContext,
    looping: bool,
}
unsafe impl Send for LibXmpPlayer {}
unsafe impl Sync for LibXmpPlayer {}

/// convert return values into rust-friendly errors
fn xmp_assert(code: c_int) -> io::Result<()> {
    match code {
        0 => Ok(()),
        -2 => Err(io::Error::new(ErrorKind::InvalidData, "Internal error")),
        -3 => Err(io::Error::new(ErrorKind::InvalidData, "Unsupported module format")),
        -4 => Err(io::Error::new(ErrorKind::InvalidData, "Error loading file")),
        -5 => Err(io::Error::new(ErrorKind::InvalidData, "Error depacking file")),
        -6 => Err(io::Error::new(ErrorKind::InvalidData, "System error")),
        -7 => Err(io::Error::new(ErrorKind::InvalidData, "Invalid parameter")),
        -8 => Err(io::Error::new(ErrorKind::InvalidData, "Invalid player state")),
        _ => Err(io::Error::new(ErrorKind::InvalidData, "Unknown error code")),
    }
}

impl LibXmpPlayer {

    /// create a new `libxmp` context in addition to loading and starting a tracker file
    pub fn new(data: &[u8], sample_rate: i32) -> io::Result<LibXmpPlayer> {
        unsafe {
            let context = xmp_create_context();
            if context.is_null() {
                return Err(io::Error::new(ErrorKind::Other, "xmp_create_context() returned null."));
            }

            //typical sample rate: 48000
            xmp_assert(xmp_load_module_from_memory(context, data.as_ptr() as _, data.len() as _))?;
            xmp_assert(xmp_start_player(context, sample_rate, 0))?;

            Ok(LibXmpPlayer {
                context,
                looping: false,
            })
        }
    }
    
    /// create a new `libxmp` context without doing anything else
    pub fn new_context() -> io::Result<LibXmpPlayer> {
        unsafe {
            let context = xmp_create_context();
            if context.is_null() {
                return Err(io::Error::new(ErrorKind::Other, "xmp_create_context() returned null."));
            }

            //moved to descrete loading functions
            //xmp_assert(xmp_load_module_from_memory(context, data.as_ptr() as _, data.len() as _))?;
            //xmp_assert(xmp_start_player(context, 48000, 0))?;

            Ok(LibXmpPlayer {
                context,
                looping: false,
            })
        }
    }

    /// load a new module into the backend (note: `unload()` does not need to be called beforehand)
    pub fn load(&mut self, data: &[u8]) -> io::Result<()> {
        unsafe {
            xmp_assert(xmp_load_module_from_memory(self.context, data.as_ptr() as _, data.len() as _))?;
        }
        Ok(())
    }

    /// close the current module and free internal resources
    pub fn unload(&mut self) {
        unsafe {
            xmp_release_module(self.context);
        }
    }

    /// begin playback of the loaded module (must call "load" on a module first), assumes audio output is stereo
    pub fn start(&mut self, sample_rate: i32) -> io::Result<()> {
        unsafe {
            xmp_assert(xmp_start_player(self.context, sample_rate, 0))?;
        }
        Ok(())
    }

    /// check if the data is a valid module, returns "true" if it is
    pub fn probe(data: &[u8]) -> bool {
        unsafe {
            xmp_test_module_from_memory(data.as_ptr() as _, data.len() as _, null_mut()) == 0
        }
    }

    /// get the name of the currently loaded module
    pub fn get_name(&self) -> Option<String> {
        unsafe {
            let mut mod_info = MaybeUninit::<XmpModuleInfo>::zeroed().assume_init();
            xmp_get_module_info(self.context, &mut mod_info as *mut XmpModuleInfo);

            let module = if let Some(mm) = mod_info.r#mod.as_ref() {
                mm
            } else {
                return None;
            };
            //let module = &*mod_info.r#mod;


            // we trust libxmp to null terminate the string.
            let name = CStr::from_ptr(module.name.as_ptr() as _);
            let name_str = name.to_string_lossy().to_string();

            // assert!(name_str.len() < XMP_NAME_SIZE);

            Some(name_str)
        }
    }

    /// unknown function purpose
    pub fn get_duration_millis(&self) -> i64 {
        0
    }

    /// report current position in playback (todo: implement)
    pub fn get_position(&self) -> i64 {
        unsafe {
            let mut frame_info = MaybeUninit::<XmpFrameInfo>::zeroed().assume_init();
            xmp_get_frame_info(self.context, &mut frame_info as *mut XmpFrameInfo);

            frame_info.pos as i64
        }
    }


    /// get "looping" attribute
    pub fn is_looping(&self) -> bool {
        self.looping
    }

    /// set whether or not the currently loaded module should loop
    pub fn set_looping(&mut self, state: bool) {
        self.looping = state;
    }

    /// go to position (todo: implement)
    pub fn seek_position(&mut self, pos: i64) -> io::Result<()> {
        unsafe {
            xmp_assert(xmp_set_position(self.context, pos as i32))?;
        }
        Ok(())
    }

    pub fn seek_row(&mut self, pos: i64) -> io::Result<()> {
        unsafe {
            xmp_assert(xmp_set_row(self.context, pos as i32))?;
        }
        Ok(())
    }

    pub fn rewind(&mut self) {
        unsafe {
            xmp_restart_module(self.context);
        }
    }

    /// returns `false` if we hit the end, `true` if operation was successful, fills `samples` with audio
    pub fn get_buffer(&mut self, samples: &mut [i16]) -> bool {
        unsafe {
            xmp_play_buffer(self.context,
                            samples.as_ptr() as _,
                            (samples.len() * size_of::<i16>()) as _,
                            if self.looping { 0 } else { 1 }) == 0
        }
    }


}

impl Drop for LibXmpPlayer {
    fn drop(&mut self) {
        unsafe {
            xmp_free_context(self.context);
        }
    }
}

