/// This file contains the libretro definitions ported from `libretro.h`
///
/// For more details see the original well-commented C header file:
/// https://github.com/libretro/RetroArch/blob/master/libretro.h
///
/// I took the liberty to "rustify" the calling convention: I dropped
/// the `retro_` prefix (useless when you have namespaces) and
/// CamelCased the struct names.
///
/// Callback typedefs are altered in the same way and suffixed with
/// `Fn` for clarity.

extern crate libc;

use std::ptr;
use std::ffi::{CStr, CString};
//use std::ffi::{c_void, c_char, c_uint, c_float, c_double, c_size_t as size_t, c_int16_t as int16_t};
use libc::{c_void, c_char, c_uint, c_float, c_double, size_t};
use std::path::PathBuf;
use std::panic;


extern crate log as rlog;

use crate::core;

pub trait Context {
    /// Get the system's audio and video parameters
    fn get_system_av_info(&self) -> SystemAvInfo;
    /// Advance the emulation state by one video frame and render it
    /// to the frontend's framebuffer
    fn render_frame(&mut self);
    /// Called when some configuration variables have been
    /// modified. The core should load the new values and change its
    /// behavior accordingly.
    fn refresh_variables(&mut self);
    /// Reset the game being played
    fn reset(&mut self);
    /// The OpenGL context has been reset, it needs to be rebuilt
    fn gl_context_reset(&mut self);
    /// The OpenGL context is about to be destroyed
    fn gl_context_destroy(&mut self);
    /// Return the maximum size of a save state in bytes
    fn serialize_size(&self) -> usize;
    /// Serialize the savestate in the provided buffer
    fn serialize(&self, _: &mut [u8]) -> Result<(), ()>;
    /// Deserialize the savestate from the provided buffer
    fn unserialize(&mut self, _: &[u8]) -> Result<(), ()>;
    /// Called before render_frame, tells the core how much time has elapsed
    fn elapse_time(&mut self, delta_time: i64);
    /// Called when it's time for the core to upload a series of audio frames
    fn async_audio_callback(&mut self);
    /// Called when the audio thread is paused or resumed
    fn async_audio_state(&mut self, is_enabled: bool);
    /// Called when a controller type is changed
    fn set_controller_port_device(&mut self, controller_port: u32, controller_type: InputDevice);

}

/// Global context instance holding our emulator state. Libretro
/// doesn't support multi-instancing.
///
/// The weird cast is here to make rustc happy with the mutable static
/// pointer. It's a valid cast because `dummy::Context` is 0-sized so
/// the pointer doesn't actually point to anything and is never
/// dereferenced. It cannot be 0 however, since that would be a NULL
/// pointer.
static mut STATIC_CONTEXT: *mut dyn Context = 1 as *mut dummy::Context;

unsafe fn set_context(context: Box<dyn Context>) {
    STATIC_CONTEXT = Box::into_raw(context);
}

unsafe fn drop_context() {
    _ = Box::from_raw(STATIC_CONTEXT);
    STATIC_CONTEXT = &mut dummy::Context;
}

fn context() -> &'static mut dyn Context {
    unsafe {
        &mut *STATIC_CONTEXT
    }
}

#[repr(C)]
pub struct SystemInfo {
   pub library_name: *const c_char,
   pub library_version: *const c_char,
   pub valid_extensions: *const c_char,
   pub need_fullpath: bool,
   pub block_extract: bool,
}

#[repr(C)]
pub struct GameGeometry {
    pub base_width: c_uint, //"unsigned" type in c++
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: c_float,
}

#[repr(C)]
pub struct SystemTiming {
    pub fps: c_double,
    pub sample_rate: c_double,
}

#[repr(C)]
pub struct SystemAvInfo {
    pub geometry: GameGeometry,
    pub timing: SystemTiming,
}

#[repr(C)]
pub struct FrameTimeCallback {
    pub callback: unsafe extern "C" fn(usec: i64),
    pub reference: i64, //default value if the frontend is messing with time
}

pub type EnvironmentFn =
    unsafe extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool;

pub type VideoRefreshFn =
    unsafe extern "C" fn(data: *const c_void,
                         width: c_uint,
                         height: c_uint,
                         pitch: size_t);
pub type AudioSampleFn =
    extern "C" fn(left: i16, right: i16);

pub type AudioSampleBatchFn =
    unsafe extern "C" fn(data: *const i16,
                         frames: size_t) -> size_t;

pub type InputPollFn = extern "C" fn();

pub type InputStateFn =
    extern "C" fn(port: c_uint,
                  device: c_uint,
                  index: c_uint,
                  id:c_uint) -> i16;

#[repr(C)]
pub struct GameInfo {
    path: *const c_char,
    data: *const c_void,
    size: size_t,
    meta: *const c_char,
}

#[repr(C)]
pub struct Variable {
    pub key: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct Message {
    pub msg: *const c_char,
    pub frames: c_uint,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    SetMessage = 6,
    EnvironmentShutdown = 7,
    GetSystemDirectory = 9,
    SetPixelFormat = 10,
    SetHwRender = 14,
    GetVariable = 15,
    SetVariables = 16,
    GetVariableUpdate = 17,
    GetRumbleInterface = 23,
    GetInputDeviceCapabilities = 24, //potentially unused ATM
    GetLogInterface = 27,
    GetSaveDirectory = 31,
    SetSystemAvInfo = 32,
    SetGeometry = 37,
    SetFrameCallback = 21,
    SetAudioCallback = 22,
    GetVFSInterface = (45 | 0x10000),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputDevice {
    None = 0,
    JoyPad = 1,
    Mouse = 2,
    Keyboard = 3,
    LightGun = 4,
    Analog = 5,
    Pointer = 6,
}
impl From<u32> for InputDevice {
    fn from(id: u32) -> InputDevice {
        match id {
            1 => InputDevice::JoyPad,
            2 => InputDevice::Mouse,
            3 => InputDevice::Keyboard,
            4 => InputDevice::LightGun,
            5 => InputDevice::Analog,
            6 => InputDevice::Pointer,
            _ => InputDevice::None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Unknown = 0,
    Backspace = 8,
    Tab = 9,
    Clear = 12,
    Return = 13,
    Pause = 19,
    Escape = 27,
    Space = 32,
    Exclaim = 33,
    QuoteDbl = 34,
    Hash = 35,
    Dollar = 36,
    Ampersand = 38,
    Quote = 39,
    LeftParen = 40,
    RightParen = 41,
    Asterisk = 42,
    Plus = 43,
    Comma = 44,
    Minus = 45,
    Period = 46,
    Slash = 47,
    Num0 = 48,
    Num1 = 49,
    Num2 = 50,
    Num3 = 51,
    Num4 = 52,
    Num5 = 53,
    Num6 = 54,
    Num7 = 55,
    Num8 = 56,
    Num9 = 57,
    Colon = 58,
    Semicolon = 59,
    Less = 60,
    Equals = 61,
    Greater = 62,
    Question = 63,
    At = 64,
    LeftBracket = 91,
    Backslash = 92,
    RightBracket = 93,
    Caret = 94,
    Underscore = 95,
    Backquote = 96,
    A = 97,
    B = 98,
    C = 99,
    D = 100,
    E = 101,
    F = 102,
    G = 103,
    H = 104,
    I = 105,
    J = 106,
    K = 107,
    L = 108,
    M = 109,
    N = 110,
    O = 111,
    P = 112,
    Q = 113,
    R = 114,
    S = 115,
    T = 116,
    U = 117,
    V = 118,
    W = 119,
    X = 120,
    Y = 121,
    Z = 122,
    Delete = 127,
    Kp0 = 256,
    Kp1 = 257,
    Kp2 = 258,
    Kp3 = 259,
    Kp4 = 260,
    Kp5 = 261,
    Kp6 = 262,
    Kp7 = 263,
    Kp8 = 264,
    Kp9 = 265,
    KpPeriod = 266,
    KpDivide = 267,
    KpMultiply = 268,
    KpMinus = 269,
    KpPlus = 270,
    KpEnter = 271,
    KpEquals = 272,
    Up = 273,
    Down = 274,
    Right = 275,
    Left = 276,
    Insert = 277,
    Home = 278,
    End = 279,
    PageUp = 280,
    PageDown = 281,
    F1 = 282,
    F2 = 283,
    F3 = 284,
    F4 = 285,
    F5 = 286,
    F6 = 287,
    F7 = 288,
    F8 = 289,
    F9 = 290,
    F10 = 291,
    F11 = 292,
    F12 = 293,
    F13 = 294,
    F14 = 295,
    F15 = 296,
    NumLock = 300,
    CapsLock = 301,
    ScrolLock = 302,
    RShift = 303,
    LShift = 304,
    RCtrl = 305,
    LCtrl = 306,
    RAlt = 307,
    LAlt = 308,
    RMeta = 309,
    LMeta = 310,
    LSuper = 311,
    RSuper = 312,
    Mode = 313,
    Compose = 314,

    Help = 315,
    Print = 316,
    SysReq = 317,
    Break = 318,
    Menu = 319,
    Power = 320,
    Euro = 321,
    Undo = 322,
}

/// RETRO_DEVICE_ID_JOYPAD_* constants
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum JoyPadButton {
    B = 0,
    Y = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
    A = 8,
    X = 9,
    L = 10,
    R = 11,
    L2 = 12,
    R2 = 13,
    L3 = 14,
    R3 = 15,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum JoypadAnalog {
    AnalogLeft = 0,
    AnalogRight = 1,
    AnalogButton = 2,
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum JoypadAnalogAxis {
    AnalogX = 0,
    AnalogY = 1,
    L2 = 12,
    R2 = 13,
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TouchpadAttribute {
    LocationX = 0,
    LocationY = 1,
    Pressed = 2, //typically keep iterating as long as this is true
    TotalCount = 3,
}



#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Xrgb1555 = 0,
    Xrgb8888 = 1,
    Rgb565 = 2,
}

//module used for using async audio (may/may not successfully be initialized depending on the frontend)
pub mod async_audio_context {
    use super::{call_environment_mut, Environment, context};


    #[repr(C)]
    pub struct AsyncAudioCallback {
        pub data_callback: unsafe extern "C" fn(),
        pub state_callback: unsafe extern "C" fn(enabled: bool),
    }


    /// Called whenever the frontend needs audio.
    #[no_mangle]
    pub unsafe extern "C" fn audio_callback() {
        //context().process_audio or something like that
        context().async_audio_callback();
    }

    /// Called by the frontend to tell the core this if the audio thread is active or not 
    #[no_mangle]
    unsafe extern "C" fn audio_state_callback(state: bool) {
        // True: Audio driver in frontend is active, and callback is
        // expected to be called regularily.
        // False: Audio driver in frontend is paused or inactive.
        // Audio callback will not be called until set_state has been
        // called with true.
        // Initial state is false (inactive).
        context().async_audio_state(state);
    }


    static mut STATIC_ASY_AUDIO_CONTEXT: AsyncAudioCallback = AsyncAudioCallback{
        data_callback: audio_callback,
        state_callback: audio_state_callback,
    };

    //register the delta time function (equivalent of micros() between times retro_run() is called)
    //returns false if the callback is not avalable on the frontend
    pub fn register_async_audio_callback() -> bool {
        unsafe {
            call_environment_mut(Environment::SetAudioCallback,
                                &mut STATIC_ASY_AUDIO_CONTEXT)
        }
    }




}

//module used for hardware rendering, contains elements like those below, but organized into a realted module
pub mod hw_context {
    use std::ffi::CString;
    use libc::{uintptr_t, c_char, c_uint, c_void};
    use super::{call_environment_mut, Environment};

    pub type ResetFn = extern "C" fn();

    pub type GetCurrentFramebufferFn = extern "C" fn() -> uintptr_t;

    pub type GetProcAddressFn = extern "C" fn(sym: *const c_char) -> *const c_void;

    #[repr(C)]
    pub enum ContextType {
        None = 0,
        OpenGl = 1,
        OpenGlEs2 = 2,
        OpenGlCore = 3,
        OpenGlEs3 = 4,
        OpenGlEsVersion = 5,
    }

    #[repr(C)]
    pub struct RenderCallback {
        context_type: ContextType,
        context_reset: ResetFn,
        get_current_framebuffer: GetCurrentFramebufferFn,
        get_proc_address: GetProcAddressFn,
        depth: bool,
        stencil: bool,
        bottom_left_origin: bool,
        version_major: c_uint,
        version_minor: c_uint,
        cache_context: bool,
        context_destroy: ResetFn,
        debug_context: bool,
    }

    pub extern "C" fn reset() {
        super::context().gl_context_reset();
    }

    pub extern "C" fn context_destroy() {
        super::context().gl_context_destroy();
    }

    //these callbacks should be replaced by ones provided by the libretro context
    pub extern "C" fn dummy_get_current_framebuffer() -> uintptr_t {
        panic!("Called missing get_current_framebuffer callback");
    }

    pub extern "C" fn dummy_get_proc_address(_: *const c_char) -> *const c_void {
        panic!("Called missing get_proc_address callback");
    }

    //default values for the STATIC_HW_CONTEXT
    static mut STATIC_HW_CONTEXT: RenderCallback = RenderCallback {
        context_type: ContextType::OpenGlCore, //OpenGlCore,
        context_reset: reset,
        // Filled by frontend
        get_current_framebuffer: dummy_get_current_framebuffer,
        // Filled by frontend
        get_proc_address: dummy_get_proc_address,
        depth: false,
        stencil: false,
        bottom_left_origin: true,
        version_major: 2,
        version_minor: 1,
        cache_context: false,
        context_destroy: context_destroy,
        debug_context: false,
    };

    pub fn init(gl_context_type: ContextType, version_maj: u32, version_min: u32) -> bool {
        unsafe {
            STATIC_HW_CONTEXT.context_type = gl_context_type;
            STATIC_HW_CONTEXT.version_major = version_maj;
            STATIC_HW_CONTEXT.version_minor = version_min;

            call_environment_mut(Environment::SetHwRender,
                &mut STATIC_HW_CONTEXT)
        }
    }

    //these dip into the onboard static struct and expose these process addresses in a safe manner
    pub fn get_proc_address(sym: &str) -> *const c_void {
        // OpenGL symbols should never contain \0 or something's very
        // wrong.
        let sym = CString::new(sym).unwrap();

        unsafe {
            (STATIC_HW_CONTEXT.get_proc_address)(sym.as_ptr() as *const c_char)
        }
    }

    pub fn get_current_framebuffer() -> uintptr_t {
        unsafe {
            (STATIC_HW_CONTEXT.get_current_framebuffer)()
        }
    }
}

//push messages to the frontend
pub mod log {
    use super::{call_environment_mut, Environment};
    use std::ffi::CString;
    use libc::c_char;

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Level {
        Debug = 0,
        Info = 1,
        Warn = 2,
        Error = 3,
    }

    /// I'm lying here for convenience: the function is really a
    /// variadic printf-like but Rust won't let me implement a
    /// variadic `dummy_log`. It doesn't matter anyway, we'll let Rust
    /// do all the formatting and simply pass a single ("%s",
    /// "formatted string").
    pub type PrintfFn = extern "C" fn(Level,
                                      *const c_char,
                                      *const c_char);

    #[repr(C)]
    pub struct Callback {
        log: PrintfFn,
    }

    extern "C" fn dummy_log(_: Level,
                            _: *const c_char,
                            _: *const c_char) {
        panic!("Called missing log callback");
    }

    static mut STATIC_LOG: PrintfFn = dummy_log as PrintfFn;

    pub fn init() -> bool {
        let mut cb = Callback { log: dummy_log };

        unsafe {
            let ok = call_environment_mut(Environment::GetLogInterface,
                                          &mut cb);

            if ok {
                STATIC_LOG = cb.log;
            }

            ok
        }
    }

    /// Send `msg` to the frontend's logger.
    pub fn log(lvl: Level, msg: &str) {
        // Make sure the message ends in a \n, mandated by the
        // libretro API.

        let trailing_newline =
            msg.as_bytes().last().map_or(false, |&c| c == b'\n');

        let format =
            if trailing_newline {
                // Message already contains a \n
                "%s\0"
            } else {
                "%s\n\0"
            };

        let msg = CString::new(msg);

        let cstr =
            match msg.as_ref() {
                Ok(s) => s.as_ptr(),
                // XXX we could replace \0 in the log with something
                // else instead.
                _ => b"<Invalid log message>" as *const _ as *const c_char,
            };

        unsafe {
            STATIC_LOG(lvl, format.as_ptr() as *const _, cstr);
        }
    }


    //simple-as function for getting *something* into the terminal
    pub fn dirty_log(msg: String) {
        log(Level::Info, msg.as_str());
    }


}

//interface for controlling the rumble of the joypad
pub mod joypad_rumble_context {
    use super::{call_environment_mut, Environment};
    use libc::{c_uint, c_ushort};

    #[repr(C)]
    pub enum RumbleMotor {
        RumbleWeak = 0,
        RumbleStrong = 1,
    }
    impl From<u16> for RumbleMotor {
        fn from(val: u16) -> Self {
            match val {
                0 => RumbleMotor::RumbleWeak,
                _ => RumbleMotor::RumbleStrong,
            }
        }
    }

    
    #[repr(C)]
    struct RumbelstateCallback {
        pub rumble_callback: unsafe extern "C" fn(port: c_uint, effect: RumbleMotor, strength: c_ushort) -> bool,
    }

    /// default callback holder
    #[no_mangle]
    pub unsafe extern "C" fn dummy_rumble_callback(_: c_uint, _: RumbleMotor, _: c_ushort) -> bool {
        panic!("Called missing rumble_callback callback");
    }

    static mut STATIC_RUMBLE_CONTEXT: RumbelstateCallback = RumbelstateCallback{
        rumble_callback: dummy_rumble_callback,
    };


    pub fn register_rumble_callback() -> bool {
        unsafe {
            call_environment_mut(Environment::GetRumbleInterface,
                                &mut STATIC_RUMBLE_CONTEXT)
        }
    }

    /// * **port** - The controller port to set the rumble state for.
    /// * **effect** - The rumble motor to set the strength of.
    /// * **strength** - The desired intensity of the rumble motor, ranging from \c 0 to \c 0xffff (inclusive).
    pub fn set_rumble(controller_port: u32, effect: u16, strengh: u16) -> bool {
        unsafe{
            (STATIC_RUMBLE_CONTEXT.rumble_callback)(controller_port, RumbleMotor::from(effect), strengh)
        }
    }



}

//interface for using retroarch's virtual filesystem
pub mod retro_filesystem_context {

    use std::{ffi::CStr, path::PathBuf};
    use super::{call_environment_mut, Environment};
    use libc::{c_uint, c_void, c_char, c_int};


    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum FileAccessMode {
        AccessRead = 1,
        AccessWrite = 2,
        AccessReadWrite = 3,
        AccessUpdateExisting = 4,
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum FileAccessHint {
        HintNone = 0,
        HintFrequentAccess = 1,
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum FileSeekPos {
        SeekStart = 0,
        SeekCurrent = 1,
        SeekEnd = 2,
    }

    //todo: RETRO_VFS_STAT_IS_VALID bitfield

    //note: relative file paths must begin with ./, according to retroarch standards.

    // Opaque pointers
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct FileHandle {
        _unused: [u8; 0],
    }
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct DirHandle {
        _unused: [u8; 0],
    }
    
    //maybe this? (must implement copy and clone)
    //pub type FileHandle = c_void;
    //pub type DirHandle = c_void;

    //*******************
    // Functions Types
    //*******************
    pub type GetPathFn = unsafe extern "C" fn(stream: *mut FileHandle) -> *const c_char;
    pub type OpenFileFn = unsafe extern "C" fn(
        path: *const c_char,
        mode: c_uint,
        hints: c_uint,
    ) -> *mut FileHandle;
    pub type CloseFileFn = unsafe extern "C" fn(stream: *mut FileHandle) -> c_int;
    pub type SizeFileFn = unsafe extern "C" fn(stream: *mut FileHandle) -> i64;
    pub type TellFileFn = unsafe extern "C" fn(stream: *mut FileHandle) -> i64;
    pub type SeekFileFn = unsafe extern "C" fn(
        stream: *mut FileHandle,
        offset: i64,
        seek_position: c_int,
    ) -> i64;
    pub type ReadFileFn = unsafe extern "C" fn(
        stream: *mut FileHandle,
        s: *mut c_void,
        len: u64,
    ) -> i64;
    pub type WriteFileFn = unsafe extern "C" fn(
        stream: *mut FileHandle,
        s: *const c_void,
        len: u64,
    ) -> i64;
    pub type FlushFileFn = unsafe extern "C" fn(stream: *mut FileHandle) -> c_int;
    pub type RemoveFileFn = unsafe extern "C" fn(path: *const c_char) -> c_int;
    pub type RenameFileFn = unsafe extern "C" fn(
        old_path: *const c_char,
        new_path: *const c_char,
    ) -> c_int;
    
    pub type TruncateFileFn = unsafe extern "C" fn(stream: *mut FileHandle, length: i64) -> i64;
    pub type StatFileFn = unsafe extern "C" fn(path: *const c_char, size: *mut i32) -> c_int;
    pub type MakeDirectoryFn = unsafe extern "C" fn(dir: *const c_char) -> c_int;
    pub type OpenDirectoryFn = unsafe extern "C" fn(
        dir: *const c_char,
        include_hidden: bool,
    ) -> *mut DirHandle;
    pub type ReadDirectoryFn = unsafe extern "C" fn(dirstream: *mut DirHandle) -> bool;
    pub type GetNameDirentFn = unsafe extern "C" fn(dirstream: *mut DirHandle) -> *const c_char;
    pub type IsDirectoryDirentFn = unsafe extern "C" fn(dirstream: *mut DirHandle) -> bool;
    pub type CloseDirectoryFn = unsafe extern "C" fn(dirstream: *mut DirHandle) -> c_int;

    //*******************
    // Dummy Functions
    //*******************
    pub unsafe extern "C" fn dummy_path_fn(_stream: *mut FileHandle) -> *const c_char{
        panic!("Called dummy_path_fn");
    }
    pub unsafe extern "C" fn dummy_openfile_fn(
        _path: *const c_char,
        _mode: c_uint,
        _hints: c_uint,
    ) -> *mut FileHandle{
        panic!("Called dummy_openfile_fn");
    }
    pub unsafe extern "C" fn dummy_closefile_fn(_stream: *mut FileHandle) -> c_int{
        panic!("Called dummy_closefile_fn");
    }
    pub unsafe extern "C" fn dummy_sizefile_fn(_stream: *mut FileHandle) -> i64{
        panic!("Called dummy_sizefile_fn");
    }
    pub unsafe extern "C" fn dummy_tellfile_fn(_stream: *mut FileHandle) -> i64{
        panic!("Called dummy_tellfile_fn");
    }
    pub unsafe extern "C" fn dummy_seekfile_fn(
        _stream: *mut FileHandle,
        _offset: i64,
        _seek_position: c_int,
    ) -> i64{
        panic!("Called dummy_seekfile_fn");
    }
    pub unsafe extern "C" fn dummy_readfile_fn(
        _stream: *mut FileHandle,
        _s: *mut c_void,
        _len: u64,
    ) -> i64{
        panic!("Called dummy_readfile_fn");
    }
    pub unsafe extern "C" fn dummy_writefile_fn(
        _stream: *mut FileHandle,
        _s: *const c_void,
        _len: u64,
    ) -> i64{
        panic!("Called dummy_writefile_fn");
    }
    pub unsafe extern "C" fn dummy_flushfile_fn(_stream: *mut FileHandle) -> c_int{
        panic!("Called dummy_flushfile_fn");
    }
    pub unsafe extern "C" fn dummy_removefile_fn(_path: *const c_char) -> c_int{
        panic!("Called dummy_removefile_fn");
    }
    pub unsafe extern "C" fn dummy_renamefile_fn(
        _old_path: *const c_char,
        _new_path: *const c_char,
    ) -> c_int{
        panic!("Called dummy_renamefile_fn");
    }  
    pub unsafe extern "C" fn dummy_truncatefile_fn(_stream: *mut FileHandle, _length: i64) -> i64{
        panic!("Called dummy_truncatefile_fn");
    }
    pub unsafe extern "C" fn dummy_statfile_fn(_path: *const c_char, _size: *mut i32) -> c_int{
        panic!("Called dummy_statfile_fn");
    }
    pub unsafe extern "C" fn dummy_makedir_fn(_dir: *const c_char) -> c_int{
        panic!("Called dummy_makedir_fn");
    }
    pub unsafe extern "C" fn dummy_opendir_fn(
        _dir: *const c_char,
        _include_hidden: bool,
    ) -> *mut DirHandle{
        panic!("Called dummy_opendir_fn");
    }
    pub unsafe extern "C" fn dummy_readdir_fn(_dirstream: *mut DirHandle) -> bool{
        panic!("Called dummy_readdir_fn");
    }
    pub unsafe extern "C" fn dummy_getnamedirent_fn(_dirstream: *mut DirHandle) -> *const c_char{
        panic!("Called dummy_getnamedirent_fn");
    }
    pub unsafe extern "C" fn dummy_isdirectorydirent_fn(_dirstream: *mut DirHandle) -> bool{
        panic!("Called dummy_isdirectorydirent_fn");
    }
    pub unsafe extern "C" fn dummy_closedirectoryfn_fn(_dirstream: *mut DirHandle) -> c_int{
        panic!("Called dummy_closedirectoryfn_fn");
    }



    #[derive(Clone, Copy, PartialEq, Eq)]
    #[repr(C)]
    pub struct VFSInterface {
        vfs_get_path: GetPathFn,
        vfs_open: OpenFileFn,
        vfs_close: CloseFileFn,
        vfs_size: SizeFileFn,
        vfs_tell: TellFileFn,
        vfs_seek: SeekFileFn,
        vfs_read: ReadFileFn,
        vfs_write: WriteFileFn,
        vfs_flush: FlushFileFn,
        vfs_remove: RemoveFileFn,
        vfs_rename: RenameFileFn,
        //v2 interface
        vfs_truncate: TruncateFileFn,
        //v3 interface
        vfs_stat: StatFileFn,
        vfs_mkdir: MakeDirectoryFn,
        vfs_opendir: OpenDirectoryFn,
        vfs_readdir: ReadDirectoryFn,
        vfs_dirent_get_name: GetNameDirentFn,
        vfs_dirent_is_dir: IsDirectoryDirentFn,
        vfs_closedir: CloseDirectoryFn,
    }

    static mut STATIC_VFS_CONTEXT: VFSInterface = VFSInterface {
        vfs_get_path: dummy_path_fn,
        vfs_open: dummy_openfile_fn,
        vfs_close: dummy_closefile_fn,
        vfs_size: dummy_sizefile_fn,
        vfs_tell: dummy_tellfile_fn,
        vfs_seek: dummy_seekfile_fn,
        vfs_read: dummy_readfile_fn,
        vfs_write: dummy_writefile_fn,
        vfs_flush: dummy_flushfile_fn,
        vfs_remove: dummy_removefile_fn,
        vfs_rename: dummy_renamefile_fn,
        //v2 interface
        vfs_truncate: dummy_truncatefile_fn,
        //v3 interface
        vfs_stat: dummy_statfile_fn,
        vfs_mkdir: dummy_makedir_fn,
        vfs_opendir: dummy_opendir_fn,
        vfs_readdir: dummy_readdir_fn,
        vfs_dirent_get_name: dummy_getnamedirent_fn,
        vfs_dirent_is_dir: dummy_isdirectorydirent_fn,
        vfs_closedir: dummy_closedirectoryfn_fn,
    };


    //passed into the command interface to get file operation handles
    #[repr(C)]
    pub struct VFSInterfaceInfo {
        required_version: c_uint,
        interface: *mut VFSInterface,
    }

    pub fn register_vfs_interface(interface_version: u32) -> bool {
        unsafe {

            let mut vfs_getter = VFSInterfaceInfo {
                required_version: interface_version,
                interface: &STATIC_VFS_CONTEXT as *const _ as *mut VFSInterface,
            };

            let result = call_environment_mut(Environment::GetVFSInterface,
                &mut vfs_getter);
            

            STATIC_VFS_CONTEXT = *vfs_getter.interface;

            return result;
            
        }
    }


    //simmilar to std::fs::File, but suckier because I made it.
    pub struct RFile {
        file_ptr: *mut FileHandle,
        mode: FileAccessMode,
    }
    impl RFile {

        pub fn path(&self) -> Result<PathBuf, ()> {
            unsafe {
                let path_ptr = (STATIC_VFS_CONTEXT.vfs_get_path)(self.file_ptr);
                let path_str = CStr::from_ptr(path_ptr).to_str();
                if let Ok(path) = path_str {
                    let pathbuffer = PathBuf::from(path);
                    Ok(pathbuffer)
                }
                else {
                    Err(())
                }

            }
        }

        pub fn open(path: PathBuf, mode: FileAccessMode, hint: FileAccessHint) -> Result<RFile, ()>{
            unsafe {
                //need to do this because vfs expects a null terminated string, but PathBuf does not.
                let path = String::from(path.to_str().unwrap()) + "\0";
                let pth = path.as_bytes();

                let file = (STATIC_VFS_CONTEXT.vfs_open)(pth.as_ptr() as *const c_char, mode as u32, hint as u32);
                if file as *const _ == std::ptr::null() {
                    return Err(())
                }

                Ok(RFile{
                    file_ptr: file,
                    mode: mode,
                })

            }
        }

        pub fn size(&self) -> Result<i64, ()> {
            unsafe{
                //vfs_size does NOT seem to catch null pointers
                let size = (STATIC_VFS_CONTEXT.vfs_size)(self.file_ptr);
                if size == -1 {
                    Err(())
                }
                else {
                    Ok(size)
                }
            }
        }

        pub fn truncate(&self, size: i64) -> Result<(), ()> {
            unsafe {
                let result = (STATIC_VFS_CONTEXT.vfs_truncate)(self.file_ptr, size);
                if result < 0 {
                    Err(())
                } else {
                    Ok(())
                }
            }
        }

        pub fn tell(&self) -> Result<i64, ()> {
            unsafe {
                let head_pos = (STATIC_VFS_CONTEXT.vfs_tell)(self.file_ptr);
                if head_pos < 0 {
                    Err(())
                } else {
                    Ok(head_pos)
                }
            }
        }

        pub fn seek(&self, seek: FileSeekPos, offset: i64) -> Result<i64, ()> {
            unsafe {
                let head_pos = (STATIC_VFS_CONTEXT.vfs_seek)(self.file_ptr, offset, seek as i32);
                if head_pos < 0 {
                    Err(())
                } else {
                    Ok(head_pos)
                }
            }
        }

        pub fn write(&self, data: &[u8]) -> Result<i64, ()> {
            unsafe {
                if (self.mode as u32 &
                    ( FileAccessMode::AccessWrite as u32
                    | FileAccessMode::AccessUpdateExisting as u32
                    )) != 0 {

                    let write_len = data.len() as u64;
                    let bytes_written = (STATIC_VFS_CONTEXT.vfs_write)(self.file_ptr, data.as_ptr() as *const c_void, write_len);
                    if bytes_written < 0 {
                        Err(())
                    } else {
                        Ok(0)
                    }
                } else {
                    Err(())
                }


            }
        }

        pub fn flush(&self) -> Result<(), ()> {
            unsafe {
                let result = (STATIC_VFS_CONTEXT.vfs_flush)(self.file_ptr);
                if result < 0 {
                    Err(())
                } else {
                    Ok(())
                }
            }
        }

        //Put these in seperate struct?
        pub fn delete(path: PathBuf) -> Result<(), ()> {
            unsafe {
                let path = String::from(path.to_str().unwrap()) + "\0";
                let pth = path.as_bytes();

                let result = (STATIC_VFS_CONTEXT.vfs_remove)(pth.as_ptr() as *const c_char);
                if result < 0 {
                    Err(())
                } else {
                    Ok(())
                }
            }
        }

        pub fn rename(path: PathBuf, new_path: PathBuf) -> Result<(), ()> {
            unsafe {
                //todo: use cstring!()
                let path = String::from(path.to_str().unwrap()) + "\0";
                let pth = path.as_bytes();

                let new_path = String::from(new_path.to_str().unwrap()) + "\0";
                let new_pth = new_path.as_bytes();

                let result = (STATIC_VFS_CONTEXT.vfs_rename)(pth.as_ptr() as *const c_char, new_pth.as_ptr() as *const c_char);
                if result < 0 {
                    Err(())
                } else {
                    Ok(())
                }
            }
        }

        pub fn stat(path: PathBuf) -> Result<(i32, i32), ()> {
            unsafe {
                let path = String::from(path.to_str().unwrap()) + "\0";
                let pth = path.as_bytes();

                let mut size: i32 = 0;
                //todo: bitfield result
                let result = (STATIC_VFS_CONTEXT.vfs_stat)(pth.as_ptr() as *const c_char, &mut size);

                Ok((result, size))
            }
        }


    }
    impl Drop for RFile {
        fn drop(&mut self) {
            unsafe {
                (STATIC_VFS_CONTEXT.vfs_close)(self.file_ptr);
            }
        }
    }

    //todo: RDirectory


}


//*******************************************
// Libretro callbacks loaded by the frontend
//*******************************************

//by default, these are populated with panic functions
static mut VIDEO_REFRESH: VideoRefreshFn = dummy::video_refresh;
static mut INPUT_POLL: InputPollFn = dummy::input_poll;
static mut INPUT_STATE: InputStateFn = dummy::input_state;
static mut AUDIO_SAMPLE_BATCH: AudioSampleBatchFn = dummy::audio_sample_batch;
static mut ENVIRONMENT: EnvironmentFn = dummy::environment; //raw environment command interface

//*******************************
// Higher level helper functions
//*******************************

//notify that the game has finished rendering a single frame
pub fn gl_frame_done(width: u32, height: u32) {
    unsafe {
        // When using a hardware renderer we set the data pointer to
        // -1 to notify the frontend that the frame has been rendered
        // in the framebuffer.
        VIDEO_REFRESH(-1isize as *const _,
                      width as c_uint,
                      height as c_uint,
                      0);
    }
}

//push out audio samples to the frontend
pub fn send_audio_samples(samples: &[i16]) {
    if samples.len() & 1 != 0 {
        panic!("Received an odd number of audio samples!");
    }

    let frames = (samples.len() / 2) as size_t;

    let r = unsafe {
        AUDIO_SAMPLE_BATCH(samples.as_ptr(), frames)
    };

    if r != frames {
        panic!("Frontend didn't use all our samples! ({} != {})", r, frames);
    }
}


//register the delta time function (equivalent of micros() between times retro_run() is called)
pub fn register_frame_time_callback(default_time: i64) -> bool {

    let data = FrameTimeCallback{
        callback: frame_time_callback,
        reference: default_time,
    };

    unsafe {
        //let tty = &frame_time_callback;
        call_environment(Environment::SetFrameCallback,
                             &data)
    }
}


//check if a button is pressed
pub fn button_pressed(port: u8, b: JoyPadButton) -> bool {
    unsafe {
        INPUT_STATE(port as c_uint,
                    InputDevice::JoyPad as c_uint,
                    0,
                    b as c_uint) != 0
    }
}

//get analog state of joysticks
pub fn joystick_analog_state(port: u8, bttn: JoypadAnalog, axis: JoypadAnalogAxis) -> i16 {
    unsafe {
        INPUT_STATE(port as c_uint,
                    InputDevice::Analog as c_uint,
                    bttn as c_uint,
                    axis as c_uint)
    }
}


//get state of touchpads (location/pressed state/count)
pub fn touchpad_analog_state(port: u8, touch_id: u32, attribute: TouchpadAttribute) -> i16 {
    unsafe {
        INPUT_STATE(port as c_uint,
                    InputDevice::Pointer as c_uint,
                    touch_id as c_uint,
                    attribute as c_uint)
    }
}


//check if a key is pressed (is also needed if keys are bound to the joypad...?)
pub fn key_pressed(port: u8, k: Key) -> bool {
    unsafe {
        INPUT_STATE(port as c_uint,
                    InputDevice::Keyboard as c_uint,
                    0,
                    k as c_uint) != 0
    }
}

//get a path to the retroarch filesystem (good for BIOS, config data, etc.)
pub fn get_system_directory() -> Option<PathBuf> {
    let mut path: *const c_char = ptr::null();

    let success =
        unsafe {
            //request system directory and set the path pointer to the result
            call_environment_mut(Environment::GetSystemDirectory,
                                 &mut path)
        };

    if success && !path.is_null() {
        let path = unsafe { CStr::from_ptr(path) };

        build_path(path)
    } else {
        None
    }
}
//get a path to the retroarch filesystem (good for save data)
pub fn get_save_directory() -> Option<PathBuf> {
    let mut path: *const c_char = ptr::null();

    let success =
        unsafe {
            //request system directory and set the path pointer to the result
            call_environment_mut(Environment::GetSaveDirectory,
                                 &mut path)
        };

    if success && !path.is_null() {
        let path = unsafe { CStr::from_ptr(path) };

        build_path(path)
    } else {
        None
    }
}

//request that the frontend close the content
pub fn request_shutdown() -> bool {
    unsafe {
        call_environment(Environment::EnvironmentShutdown, &ptr::null::<c_void>())
    }
}

//set pixel mix type
pub fn set_pixel_format(format: PixelFormat) -> bool {
    let f = format as c_uint;

    unsafe {
        call_environment(Environment::SetPixelFormat, &f)
    }
}

//tell the frontend what the window's shape (width/height) is
pub fn set_geometry(geom: &GameGeometry) -> bool {
    unsafe {
        call_environment(Environment::SetGeometry, geom)
    }
}

//set geometry and timing info(FPS and audio sample definition)
/// Can destroy the OpenGL context!
pub unsafe fn set_system_av_info(av_info: &SystemAvInfo) -> bool {
    call_environment(Environment::SetSystemAvInfo, av_info)
}

/// Display `msg` on the screen for `nframes` frames
pub fn set_message(nframes: u32, msg: &str) {
    let msg = CString::new(msg);

    let cstr =
        match msg.as_ref() {
            Ok(s) => s.as_ptr(),
            _ => b"<Invalid log message>" as *const _ as *const c_char,
        };

    let message = Message { msg: cstr, frames: nframes as c_uint };

    unsafe {
        call_environment(Environment::SetMessage, &message);
    }
}

//true if the user changed a setting since last call
pub fn variables_need_update() -> bool {
    let mut needs_update = false;

    let ok =
        unsafe {
            call_environment_mut(Environment::GetVariableUpdate,
                                 &mut needs_update)
        };

    if !ok {
        panic!("Environment::GetVariableUpdate failed");
    }

    needs_update
}

//tells the frontend what options are avalable to be changed in settings
/// `variables` *must* end with a `{ NULL, NULL }` marker
pub unsafe fn register_variables(variables: &[Variable]) -> bool {
    call_environment_slice(Environment::SetVariables, variables)
}

//send command to frontend where the input is a pointer that can be changed
unsafe fn call_environment_mut<T>(which: Environment, var: &mut T) -> bool {
    ENVIRONMENT(which as c_uint, var as *mut _ as *mut c_void)
}
//send a command to the frontend where the input is a constant pointer that cannot be changed
unsafe fn call_environment<T>(which: Environment, var: &T) -> bool {
    ENVIRONMENT(which as c_uint, var as *const _ as *mut c_void)
}
//send a command to the frontend where the input is an array of many things that cannot be changed
unsafe fn call_environment_slice<T>(which: Environment, var: &[T]) -> bool {
    ENVIRONMENT(which as c_uint, var.as_ptr() as *const _ as *mut c_void)
}

/// Cast a mutable pointer into a mutable reference, return None if
/// it's NULL.
fn ptr_as_mut_ref<'a, T>(v: *mut T) -> Option<&'a mut T> {

    if v.is_null() {
        None
    } else {
        Some(unsafe { &mut *v })
    }
}

/// Cast a const pointer into a reference, return None if it's NULL.
fn ptr_as_ref<'a, T>(v: *const T) -> Option<&'a T> {

    if v.is_null() {
        None
    } else {
        Some(unsafe { &*v })
    }
}

//**********************************************
// Libretro entry points called by the frontend
//**********************************************

//these functions are exposed in the core's library to be poked by the frontend

//tell the frontend what version of the API this is using
#[no_mangle]
pub extern "C" fn retro_api_version() -> c_uint {
    // We implement the version 1 of the API
    1
}

//the frontend will populate the command interface pointer with its own function
#[no_mangle]
pub extern "C" fn retro_set_environment(callback: EnvironmentFn) {
    unsafe {
        ENVIRONMENT = callback
    }

    core::init_variables();
}

//ditto, for video
#[no_mangle]
pub extern "C" fn retro_set_video_refresh(callback: VideoRefreshFn) {
    unsafe {
        VIDEO_REFRESH = callback
    }
}

//for now, we do not get individual samples, so this function does nothing
#[no_mangle]
pub extern "C" fn retro_set_audio_sample(_: AudioSampleFn) {
}

//we send out audio samples in batches, so we want the pointer to where batches are loaded in
#[no_mangle]
pub extern "C" fn retro_set_audio_sample_batch(callback: AudioSampleBatchFn) {
    unsafe {
        AUDIO_SAMPLE_BATCH = callback
    }
}

//running INPUT_POLL tells the frontend to check for inputs
#[no_mangle]
pub extern "C" fn retro_set_input_poll(callback: InputPollFn) {
    unsafe {
        INPUT_POLL = callback
    }
}

//get the callback that exposes the latest input states to the backend
#[no_mangle]
pub extern "C" fn retro_set_input_state(callback: InputStateFn) {
    unsafe {
        INPUT_STATE = callback
    }
}

//things that run only one time when the core is first loaded
static mut FIRST_INIT: bool = true;

#[no_mangle]
pub extern "C" fn retro_init() {
    // retro_init can potentially be called several times even if the
    // library hasn't been unloaded (statics are not reset etc...)
    // which makes it rather useless in my opinion. Let's change that.

    unsafe {
        if FIRST_INIT {
            core::init();
            FIRST_INIT = false;
        }
    }
}

//called when the core is unloaded (original author may have intended to reset pointers)
#[no_mangle]
pub extern "C" fn retro_deinit() {
    // XXX Should I reset the callbacks to the dummy implementations
    // here?
}

//informs frontend of library specs? or informs library of frontend specs?
//ans: informs frontend of library specs
#[no_mangle]
pub extern "C" fn retro_get_system_info(info: *mut SystemInfo) {
    let info = ptr_as_mut_ref(info).unwrap(); //cast pointer into something rust-friendly

    // Strings must be static and, of course, 0-terminated
    *info = core::SYSTEM_INFO; //set info to what we have already made
}

//ditto, but about audio and video parameters
#[no_mangle]
pub extern "C" fn retro_get_system_av_info(info: *mut SystemAvInfo) {
    let info = ptr_as_mut_ref(info).unwrap();

    *info = context().get_system_av_info();
}

//tells the frontend what the controller mappings are


//tells the core if a controller has been changed and what type it is
#[no_mangle]
pub extern "C" fn retro_set_controller_port_device(port: c_uint,
                                                   device: c_uint) {

    rlog::info!("port device: {} {}", port, device); //debug!
    context().set_controller_port_device(port, InputDevice::from(device));
}

//tell backend it needs to reset
#[no_mangle]
pub extern "C" fn retro_reset() {
    context().reset();
}

//called every ingame "tick"
#[no_mangle]
pub unsafe extern "C" fn retro_run() {
    //get latest input states
    INPUT_POLL();

    //grab backend runner (for d-rs, this is simply the backend, we need to impl context for it)
    let context = context();

    //check for panics from the backend and catch them in r
    let r = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        //check if settings have been updated and change internal settings to match
        if variables_need_update() {
            context.refresh_variables();
        }

        //draw video frame
        context.render_frame();
    }));

    // Catch panics to cleanly destroy everything. Since this method
    // is called from a C FFI the panic won't be able to go up anyway.
    if r.is_err() {
        drop_context();
        // This will probably cause an abort
        panic!("retro_run panicked");
    }
}

//tell rust how big the save state will be
#[no_mangle]
pub extern "C" fn retro_serialize_size() -> size_t {
    context().serialize_size()
}

//save state
#[no_mangle]
pub extern "C" fn retro_serialize(data: *mut c_void,
                                  size: size_t) -> bool {
    //cast pointer
    let data = unsafe {
        ::std::slice::from_raw_parts_mut(data as *mut u8, size)
    };

    // Set the buffer to 0 in case parts of it remain unused, it'll
    // avoid putting garbage in the save file and might help with
    // compression
    for b in data.iter_mut() {
        *b = 0;
    }

    //tell backend to handle save-stating
    context().serialize(data).is_ok()
}

//load a save state back
#[no_mangle]
pub extern "C" fn retro_unserialize(data: *const c_void,
                                    size: size_t) -> bool {
    let data = unsafe {
        ::std::slice::from_raw_parts(data as *const u8, size)
    };

    context().unserialize(data).is_ok()
}

//handle cheats (not implemented here)
#[no_mangle]
pub extern "C" fn retro_cheat_reset() {
}

#[no_mangle]
pub fn retro_cheat_set(_index: c_uint,
                       _enabled: bool,
                       _code: *const c_char) {
}


#[no_mangle]
pub extern "C" fn retro_load_game(info: *const GameInfo) -> bool {
    let info = ptr_as_ref(info).unwrap();

    if info.path.is_null() {
        rlog::warn!("No path in GameInfo!");
        //warn!("No path in GameInfo!");
        return false;
    }

    let path = unsafe { CStr::from_ptr(info.path) };

    let path =
        match build_path(path) {
            Some(p) => p,
            None => return false,
        };

    match core::load_game(path) {
        Some(c) => {
            unsafe {
                set_context(c);
            }
            true
        }
        None => {
            rlog::warn!("Couldn't load game!");
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn retro_load_game_special(_type: c_uint,
                                          _info: *const GameInfo,
                                          _num_info: size_t) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn retro_unload_game()  {
    drop_context();
}

#[no_mangle]
pub extern "C" fn retro_get_region() -> c_uint {
    0
}

#[no_mangle]
pub extern "C" fn retro_get_memory_data(_id: c_uint) -> *mut c_void {
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn retro_get_memory_size(_id: c_uint) -> size_t {
    0
}


//must be initialized with register_frame_time_callback before it is active
#[no_mangle]
pub unsafe extern "C" fn frame_time_callback(delta_time: i64) {
    context().elapse_time(delta_time);
}

//*************************************************************
// Dummy Module (for when there is not Context implemented)
//*************************************************************

pub mod dummy {
    //! Placeholder implementation for the libretro callback in order
    //! to catch calls to those function in the function pointer has
    //! not yet been loaded.

    use libc::{c_void, c_uint, size_t};

    use super::InputDevice;

    pub unsafe extern "C" fn video_refresh(_: *const c_void,
                                       _: c_uint,
                                       _: c_uint,
                                       _: size_t) {
        panic!("Called missing video_refresh callback");
    }

    pub extern "C" fn input_poll() {
        panic!("Called missing input_poll callback");
    }

    pub unsafe extern "C" fn audio_sample_batch(_: *const i16,
                                                _: size_t) -> size_t {
        panic!("Called missing audio_sample_batch callback");
    }

    pub extern "C" fn input_state(_: c_uint,
                                  _: c_uint,
                                  _: c_uint,
                                  _: c_uint) -> i16 {
        panic!("Called missing input_state callback");
    }

    pub unsafe extern "C" fn environment(_: c_uint, _: *mut c_void) -> bool {
        panic!("Called missing environment callback");
    }

    pub struct Context;

    impl super::Context for Context {
        fn render_frame(&mut self) {
            panic!("Called render_frame with no context!");
        }

        fn get_system_av_info(&self) -> super::SystemAvInfo {
            panic!("Called get_system_av_info with no context!");
        }

        fn refresh_variables(&mut self) {
            panic!("Called refresh_variables with no context!");
        }

        fn reset(&mut self) {
            panic!("Called reset with no context!");
        }

        fn gl_context_reset(&mut self) {
            panic!("Called context_reset with no context!");
        }

        fn gl_context_destroy(&mut self) {
            panic!("Called context_destroy with no context!");
        }

        fn serialize_size(&self) -> usize {
            panic!("Called serialize_size with no context!");
        }

        fn serialize(&self, _: &mut [u8]) -> Result<(), ()> {
            panic!("Called serialize with no context!");
        }

        fn unserialize(&mut self, _: &[u8]) -> Result<(), ()> {
            panic!("Called unserialize with no context!");
        }

        fn elapse_time(&mut self, _: i64) {
            panic!("Called elapse_time with no context!");
        }

        fn async_audio_callback(&mut self) {
            panic!("Called async_audio_callback with no context!");
        }

        fn async_audio_state(&mut self, _: bool) {
            //this is reduced to warn level because the core has been unloaded by the time this is called.
            log::warn!("Called async_audio_state with no context!");
        }

        fn set_controller_port_device(&mut self, _: u32, _: InputDevice) {
            panic!("Called set_controller_port_device with no context!");
        }

    }
}

/// Build a PathBuf from a C-string provided by the frontend. If the
/// C-string doesn't contain a valid Path encoding return
/// "None". `c_str` *must* be a valid pointer to a C-string.
#[cfg(unix)]
fn build_path(cstr: &CStr) -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    // On unix I assume that the path is an arbitrary null-terminated
    // byte string
    Some(PathBuf::from(OsStr::from_bytes(cstr.to_bytes())))
}

/// Build a PathBuf from a C-string provided by the frontend. If the
/// C-string doesn't contain a valid Path encoding return
/// "None". `c_str` *must* be a valid pointer to a C-string.
#[cfg(not(unix))]
fn build_path(cstr: &CStr) -> Option<PathBuf> {
    // On Windows and other non-unices I assume that the path is utf-8
    // encoded
    match cstr.to_str() {
        Ok(s) => Some(PathBuf::from(s)),
        Err(_e) => {
            rlog::error!("The frontend gave us an invalid path: {}",
                   cstr.to_string_lossy()); //originally error! macro, is this a valid substitute?
            None
        }
    }
}

//sends a command to the environment to get some variable (settings?)
pub unsafe fn get_variable<T, E>(var: &str,
                                 var_cstr: *const c_char,
                                 parser: fn (&str) -> Result<T, E>) -> T
{
    let mut v = Variable {
        key: var_cstr as *const _,
        value: ptr::null(),
    };

    let ok =
        call_environment_mut(Environment::GetVariable, &mut v);

    if !ok || v.value.is_null() {
        panic!("Couldn't get variable {}", var);
    }

    let value = CStr::from_ptr(v.value).to_str().unwrap();

    match parser(value) {
        Ok(v) => v,
        Err(_) => panic!("Couldn't parse variable {}", var),
    }
}

#[allow(unused_macros)]
macro_rules! cstring {
    ($x:expr) => {
        concat!($x, '\0') as *const _ as *const c_char
    };
}

/// Create a structure `$st` which will be used to register and access
/// libretro variables:
///
/// ```rust
/// libretro_variables!(
///     struct MyVariables (prefix = "mycore") {
///         some_option: i32, FromStr::from_str => "Do something; 1|2|3",
///         enable_stuff: bool, parse_bool => "Enable stuff; enabled|disabled",
///     });
///
/// fn parse_bool(opt: &str) -> Result<bool, ()> {
///    match opt {
///        "true" | "enabled" | "on" => Ok(true),
///        "false" | "disabled" | "off" => Ok(false),
///        _ => Err(()),
///    }
/// }
///
/// ```
///
/// The variable names given to the frontend will be prefixed with
/// `$prefix` as mandated by libretro.
///
/// $parser must be a function that takes an &str and returns a
/// Result<T, _> where T is the option type.
///
/// The variables can then be registered with the frontend (prefrably
/// in the `init_variables` callback with:
///
/// ```rust
/// MyVariables::register();
/// ```
///
/// Individual variables can be accessed using getter functions:
///
/// ```rust
/// let value = MyVariables::some_option();
/// ```
#[macro_export]
macro_rules! libretro_variables {
    (struct $st:ident (prefix = $prefix:expr) {
        $($name:ident : $ty:ty , $parser:expr => $str:expr),+$(,)*
    }) => (
        struct $st;

        impl $st {
            fn register() {

                let variables = [
                    $($crate::libretro::Variable {
                        key: cstring!(concat!($prefix, '_', stringify!($name))),
                        value: cstring!($str),
                    }),+,
                    // End of table marker
                    $crate::libretro::Variable {
                        key: ::std::ptr::null() as *const c_char,
                        value: ::std::ptr::null() as *const c_char,
                    }
                    ];

                let ok = unsafe {
                    $crate::libretro::register_variables(&variables)
                };

                if !ok {
                    log::warn!("Failed to register variables"); //warn
                }
            }

            $(fn $name() -> $ty {
                let cstr = cstring!(concat!($prefix, '_', stringify!($name)));

                unsafe {
                    $crate::libretro::get_variable(stringify!($name),
                                                   cstr,
                                                   $parser)
                }
            })+
        });
}

#[macro_export]
macro_rules! libretro_message {
    ($nframes:expr, $($arg:tt)+) =>
        ($crate::libretro::set_message($nframes, &format!($($arg)+)))
}
