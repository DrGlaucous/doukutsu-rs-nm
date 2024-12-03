
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::pin::Pin;
use std::ffi::CStr;

use libc::{c_char, c_uint};
//use libretro_rs::retro::JoypadButton;
use std::ffi::c_void;


//use doukutsu_rs::framework::backend::BackendEventLoop;
use doukutsu_rs::framework::backend_libretro::{LibretroEventLoop, LibretroBackend, RenderMode};
use doukutsu_rs::framework::backend::{BackendEventLoop, Backend};
use doukutsu_rs::framework::keyboard::ScanCode;
use doukutsu_rs::framework::gamepad::{Button, Axis};
use doukutsu_rs::framework::context::{self, Context};
use doukutsu_rs::game::Game;
use doukutsu_rs::scene::title_scene;
use doukutsu_rs::game::shared_game_state::SharedGameState;
use doukutsu_rs::sound::backend_libretro::{OutputBufConfig, Runner};

use crate::libretro::{self,
    hw_context::ContextType,
    button_pressed,
    get_save_directory,
    //get_system_directory,
    gl_frame_done,
    joypad_rumble_context,
    key_pressed,
    joystick_analog_state,
    touchpad_analog_state,
    send_audio_samples,
    set_geometry,
    request_shutdown,
    InputDevice,
    JoyPadButton,
    JoypadAnalog,
    JoypadAnalogAxis,
    TouchpadAttribute,
    Key,
    log as rlog,
    log::Level,
};

/// Static system information sent to the frontend on request
pub const SYSTEM_INFO: libretro::SystemInfo = libretro::SystemInfo {
    library_name: cstring!("d-rs"),
    library_version: "0.0.1" as *const _ as *const c_char,
    valid_extensions: cstring!("exe"),
    need_fullpath: true,
    block_extract: false,
};

//Width is set relative to height using aspect ratio
//pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 240; 

pub const GAMEPAD_COUNT: u16 = 2;

////////////////////////BACKEND CALLS

/// Called when a game is loaded and a new context must be built
pub fn load_game(target: PathBuf) -> Option<Box<dyn libretro::Context>> {
    log::info!("Loading {:?}", target); //info!

    //todo: get disk into there
    Core::new(target).ok()
        .map(|c| Box::new(c) as Box<dyn libretro::Context>)
}

pub fn init() {
    //nothing to do here
}

pub fn init_variables() {
    CoreVariables::register();
}

////////////////////////SETTINGS

//helper for the settings macro below
fn parse_numeric(opt: &str) -> Result<u32, <u32 as FromStr>::Err> {
    let num = opt.trim_matches(|c: char| !c.is_numeric());

    num.parse()
}
fn parse_ratio(opt: &str) -> Result<(u32,u32), ()> {

    let num = opt.trim_matches(|c: char| !c.is_numeric()); //trims down to ratio (16:9)
    let num: Vec<_> = num.split(':').collect(); //split into "16" and "9"
    let a: Result<u32, <u32 as FromStr>::Err> = num[0].parse();
    let b: Result<u32, <u32 as FromStr>::Err> = num[1].parse();

    Ok((a.unwrap(), b.unwrap()))
}


fn parse_bool(opt: &str) -> Result<bool, ()> {
    match opt {
        "true" | "enabled" | "on" => Ok(true),
        "false" | "disabled" | "off" => Ok(false),
        _ => Err(()),
    }
}

//macro to build settings for the setting menu
libretro_variables!(
    struct CoreVariables (prefix = "d-rs") {
        internal_upscale_factor: u32, parse_numeric
            => "Internal upscaling factor; \
                1x (freeware)|2x (CS+)|3x|4x",
        screen_ratio: (u32, u32), parse_ratio
            => "Screen Ratio; \
                4:3 (original)|16:9|21:9",
        draw_debug_outlines: bool, parse_bool
            => "Debug Outlines; disabled|enabled",       
        show_fps: bool, parse_bool
            => "Show FPS; disabled|enabled",
        show_debug_window: bool, parse_bool
            => "Show Debug GUI; disabled|enabled", 
        god_mode: bool, parse_bool
            => "GOD Mode (Invincibility); disabled|enabled",
        infinite_booster: bool, parse_bool
            => "Infinite Booster; disabled|enabled",
        noclip: bool, parse_bool
            => "Noclip; disabled|enabled",
        more_rust: bool, parse_bool
            => "More Rust; disabled|enabled", 

    });

/////////////////////CORE IMPL

struct Core<'a>  {
    //runner backend and other loop pointers are in here
	backend: Box<LibretroBackend>,
	event_loop: Box<LibretroEventLoop>,
    ////data_path: PathBuf,

    state_ref: &'a mut SharedGameState,
    pub game: Pin<Box<Game>>,
    pub context: Pin<Box<Context>>,	
    screen_width: u32,
    screen_height: u32,

    rumble_enabled: bool,
    async_audio_enabled: bool, //true if async audio has been enabled
    delta_time: i64, //time since last frame
    audio_runner: Runner, //object that containst the audio context
}

impl<'a>  Core<'a>  {

    fn new(target: PathBuf) -> Result<Core<'a>, ()>{

        //initialize the hardware backends

        //needed before d-rs logging is started (not sure why this is the only function where this doesn't work...)
        if !rlog::init() {
            return Err(());
        }

        rlog::log(Level::Debug, "Initializing Core.");

        if !libretro::set_pixel_format(libretro::PixelFormat::Xrgb8888) {
            //log::warn!("Can't set pixel format");
            rlog::log(Level::Error, "Can't set pixel format");
            return Err(());
        }

        //try different openGL versions

        let gl_version = if cfg!(target_os = "macos") {(3, 3)} else {(2, 1)}; // mac OS demands at least openGL3 with retroarch frontend

        let mut render_mode = RenderMode::OpenGL(gl_version.0, gl_version.1);
        if !libretro::hw_context::init(ContextType::OpenGlCore, gl_version.0, gl_version.1) {
            render_mode = RenderMode::OpenGLES;
            if !libretro::hw_context::init(ContextType::OpenGlEs2, 2, 1) {
                //log::warn!("Failed to init hardware context");
                rlog::log(Level::Error, "Failed to init hardware context");

                //todo: full software rendering support, but for now, error out.
                return Err(());
            }

        }

        //the value of 50 here is arbitrary (in micros). Bigger numbers mean the mainloop will be called less often.
        if !libretro::register_frame_time_callback(50) {
            //log::warn!("Failed to init delta frame counter");
            rlog::log(Level::Error, "Failed to init delta frame counter");
            return Err(());
        }

        let async_audio_enabled = if !libretro::async_audio_context::register_async_audio_callback() {
            //todo: implement synchronous audio
            
            //log::warn!("Failed to init async audio, falling back to synchronous");
            rlog::log(Level::Warn, "Failed to init async audio, falling back to synchronous");
            false
        } else {true};

        let rumble_enabled = if !libretro::joypad_rumble_context::register_rumble_callback() {
            //log::warn!("Failed to init rumble");
            rlog::log(Level::Error, "Failed to init rumble");
            false
        } else {true};

        //function to use in order to get the current framebuffer (must be handed down to the backend renderer)
        let get_current_framebuffer: fn() -> usize = libretro::hw_context::get_current_framebuffer;
        let get_proc_address: fn(&str) -> *const c_void = libretro::hw_context::get_proc_address;

        //create a hook to grab the audio backend from shared_game_state
        let mut audio_runner: Option<Runner> = None;
        //default config that retroarch runs with
        let sound_config = OutputBufConfig {
            sample_rate: 44_100.0,
            channel_count: 2,
            runner_out: &mut audio_runner,
        };

        //target is assumed to be either the exe OR the directory containing the data folder "./folder/Doukutsu.exe" or "./folder/" (for cs-switch)
        
        let mut resource_dir = target;
        // If it's targeting the actual file, remove the file refrence for just the raw directory.
        if resource_dir.is_file() {
            let _ = resource_dir.pop();
        }
        resource_dir.push("data");

        //set path for the game saves. If we can, start by putting the saves in the global retroarch directory. If not, put it in the portable directory
        let user_dir = if let Some(mut dir) = get_save_directory() {

            //check to make sure there is a d-rs subdirectory for us (not all frontends give us one, but some do.)
            let lib_name = unsafe {CStr::from_ptr(SYSTEM_INFO.library_name).to_bytes().to_vec()};
            let lib_name = String::from_utf8(lib_name).unwrap();
            if !dir.ends_with(&lib_name) {
                dir.push(lib_name);
            }
            dir

        } else {
            //log::warn!("Failed to get save directory. Using portable directory.");
            rlog::log(Level::Warn, "Failed to get save directory. Using portable directory.");
            
            let mut usr_target = resource_dir.clone();
            let _ = usr_target.pop();

            usr_target.push("user");
            usr_target
        };


        //user_dir is either "/path_to_libretro/saves/d-rs/" [or] "/path_to_executable/user/"
        //resource_dir is "/path_to_executable/data/"

        let options = doukutsu_rs::game::LaunchOptions {
            server_mode: false,
            editor: false,
            return_types: true,
            external_timer: true,
            resource_dir: Some(resource_dir),
            usr_dir: Some(user_dir),
            audio_config: sound_config,
        };

		let (game, context) = doukutsu_rs::game::init(options).unwrap();

		let mut context = context.unwrap();
		let mut game = game.unwrap();
		let game_ptr = game.as_mut().get_mut();


		let (backend, event_loop) = context.create_backend(game_ptr, get_current_framebuffer, get_proc_address, render_mode).unwrap();

        let state_ref = unsafe {&mut *game.state.get()};

        //set starting resolution:
        let scale_factor = CoreVariables::internal_upscale_factor();
        let ratio = CoreVariables::screen_ratio();
        let initial_height = HEIGHT * scale_factor;
        let initial_width = initial_height * ratio.0 / ratio.1;

        let mut core = Core {
            backend,
            event_loop,
            context,
            state_ref,
            game,
            screen_height: initial_height,
            screen_width: initial_width,
            rumble_enabled,
            async_audio_enabled,
            delta_time: 0,
            audio_runner: audio_runner.unwrap(),

            ////data_path: data.clone().to_path_buf(), 
        };

        //assume gamepads are always connected from retroarch (todo: make this dynamic)
        for idx in 0..GAMEPAD_COUNT {
            use crate::libretro::Context;
            core.set_controller_port_device(idx as u32, InputDevice::JoyPad);
        }

        //refresh config info
        {
            use crate::libretro::Context;
            core.refresh_variables();
        }


        rlog::log(Level::Debug, "Core initialized.");

        Ok(core)
        
    }

    // mainly for testing, having this active as well as the gamepad results in some conflicts because some buttons are applied 2x
    #[allow(unused)]
    fn poll_keys(&mut self) {
        
        for (ret_key, drs_key) in KEY_MAP {
            key_pressed(0, ret_key);
            self.event_loop.update_keys(&mut self.context, drs_key, key_pressed(0, ret_key));
        }
    }

    fn poll_gamepad(&mut self) {
    
        // for idx in 0..GAMEPAD_COUNT {
        //     let output = joystick_analog_state(idx as u8, JoypadAnalog::AnalogButton, JoypadAnalogAxis::L2);
        //     if output != 0 {
        //         rlog::log(Level::Info, format!("ID: {} Analog value: {}", idx, output).as_str());
        //         let bt_state = button_pressed(idx as u8, JoyPadButton::L2);
        //         rlog::log(Level::Info, format!("Button pressed: {}", bt_state).as_str());
        //     }
        // }

    
        for idx in 0..GAMEPAD_COUNT {

            //handle axis
            for ((ret_stick, ret_axis, ret_but),drs_axis) in AXIS_MAP {
                let mut analog_value = joystick_analog_state(idx as u8, ret_stick, ret_axis);
                //required for touch or keybound L2/R2 buttons, since they don't register an analog value that way, but d-rs still needs one
                if ret_stick == JoypadAnalog::AnalogButton
                && analog_value == 0
                && button_pressed(idx as u8, ret_but) {
                    analog_value = i16::MAX;
                }
                self.event_loop.update_gamepad_axis(&mut self.context, idx, drs_axis, analog_value);
                

            }

            //handle buttons
            for (ret_but, drs_but) in BUTTON_MAP {

                //test (fast conditional breakpoint)
                // let bt_state = button_pressed(idx as u8, ret_but);
                // if bt_state {        
                //     rlog::log(Level::Info, format!("Button pressed: {}:{}", idx, ret_but as u32).as_str());
                // }

                self.event_loop.update_gamepad_key(&mut self.context, idx, drs_but, button_pressed(idx as u8, ret_but));
    
            }


        }
    }

    #[allow(unused)]
    fn poll_touch(&mut self) {

        let mut iterator = 0;
        while touchpad_analog_state(0, iterator, TouchpadAttribute::Pressed) != 0 {

            let x = touchpad_analog_state(0, iterator, TouchpadAttribute::LocationX);
            let y = touchpad_analog_state(0, iterator, TouchpadAttribute::LocationY);

            self.event_loop.update_touchpad(x, y, iterator as u16);

            iterator += 1;
        }
        self.event_loop.finalize_touchpad(&mut self.state_ref);

    }

    fn run_audio(&mut self) {

        self.audio_runner.run();
        send_audio_samples(&self.audio_runner.data);
    }

    //returns retroarch-formatted AV-info from internal core variables
    fn core_av_info(&self) -> libretro::SystemAvInfo {


        // output is 640x480
        let max_width = (self.screen_width * 1) as c_uint;
        let max_height = (self.screen_height * 1) as c_uint;

        libretro::SystemAvInfo {
            geometry: libretro::GameGeometry {
                // The base resolution will be overriden using
                // ENVIRONMENT_SET_GEOMETRY before rendering a frame so
                // this base value is not really important
                base_width: max_width,
                base_height: max_height,
                max_width: HEIGHT * 3 * 21 / 9, // widest aspect ratio at largest scale, any smaller and we'd get edge clipping at this larger scale.
                max_height: HEIGHT * 3, // note: we could also forego this with backend reinitialization, but that's slow and process-heavy
                aspect_ratio: (max_width as f32)/(max_height as f32),
            },
            timing: libretro::SystemTiming {
                fps: 50 as f64,
                sample_rate: 44_100. //samples per second
            }
        }


    }


    fn set_resolution(&mut self) {

        let initial_width = self.screen_width;
        let initial_height = self.screen_height;

        let scale_factor = CoreVariables::internal_upscale_factor();
        let ratio = CoreVariables::screen_ratio();

        let height = HEIGHT * scale_factor;
        let width = height * ratio.0 / ratio.1;

        self.screen_height = height;
        self.screen_width = width;

        if height != initial_height || width != initial_width {
            let new_av_info = self.core_av_info();
            set_geometry(&new_av_info.geometry);
            let _ = self.event_loop.handle_resize(&mut self.state_ref, &mut self.context, width, height);
        }

    }

}


impl<'a>  libretro::Context  for Core<'a>  {

    fn render_frame(&mut self) {


        self.poll_gamepad();
        
        //(almost) implemented, but not completely as it isn't needed and I'm tired
        //self.poll_touch();
        //self.poll_keys();

        self.event_loop.update(self.state_ref, self.game.as_mut().get_mut(), &mut self.context, self.delta_time as u64);
        gl_frame_done(self.screen_width, self.screen_height);

        //terminate with the ingame menu
        if self.state_ref.shutdown {
            request_shutdown();
        }

        //run audio synchronously? (todo: make audio batch sample size dynamic for this to work)
        // if !self.async_audio_enabled {
        //     self.run_audio();
        // }


    }

    //tell frontend what audio and video parameters to use
    fn get_system_av_info(&self) -> libretro::SystemAvInfo {
        self.core_av_info()
    }

    //settings have been changed, update them inside the game
    fn refresh_variables(&mut self){
        //let result = parse_ratio("16:9 (widescreen)");

        //let internal_upscale_factor = CoreVariables::internal_upscale_factor();
        //let screen_ratio = CoreVariables::screen_ratio();
        self.set_resolution();

        self.state_ref.settings.god_mode = CoreVariables::god_mode();
        self.state_ref.settings.fps_counter = CoreVariables::show_fps();
        self.state_ref.settings.infinite_booster = CoreVariables::infinite_booster();
        self.state_ref.settings.debug_outlines = CoreVariables::draw_debug_outlines();
        self.state_ref.settings.noclip = CoreVariables::noclip();
        self.state_ref.debugger = CoreVariables::show_debug_window();
        self.state_ref.more_rust = CoreVariables::more_rust();




    }

    //soft-reset (gl is not re-initialized, send game back to top menu)
    fn reset(&mut self) {
        self.state_ref.next_scene = Some(Box::new(title_scene::TitleScene::new()));
    }

    //gl context was destroyed, now rebuild it (called when game is initialized).
    fn gl_context_reset(&mut self){
        let _ = self.event_loop.rebuild_renderer(self.state_ref, &mut self.context, self.screen_width, self.screen_height);
    }

    //called when frontend window resolution is changed,
    //the gl context is about to be destroyed, remove anything from the back while you can
    fn gl_context_destroy(&mut self){
         let _ = self.event_loop.destroy_renderer(&mut self.state_ref, &mut self.context);
    }

    //how long since the last frame was called
    fn elapse_time(&mut self, delta_time: i64) {
        self.delta_time = delta_time; //in microseconds us
    }

    fn async_audio_callback(&mut self) {
        self.run_audio();
    }
    //not really needed at the moment...
    fn async_audio_state(&mut self, _is_enabled: bool) {
        
    }

    //used to change or set controller mappings
    fn set_controller_port_device(&mut self, port: u32, controller_type: InputDevice) {

        match controller_type {
            InputDevice::JoyPad => {

                //assign the joypad to the backend
                self.event_loop.add_gamepad(self.state_ref, &mut self.context, port, 
                    if self.rumble_enabled {Some(joypad_rumble_context::set_rumble)} else {None}
                );

                //set up user-readable joypad mappings (might be optional since these IDs can also be set ingame, making the ones here invalid.)



            }
            InputDevice::Keyboard => {
                //remove gamepad and "add" keyboard (?)
            }
            _ => {}
        }

    }


    //todo: remove unused functions from Context
    fn serialize_size(&self) -> usize {
        0
    }
    fn serialize(&self, mut _buf: &mut [u8]) -> Result<(), ()> {
        Ok(())
    }
    fn unserialize(&mut self, mut _buf: &[u8]) -> Result<(), ()> {
        Ok(())
    }

}


/////////////////////UTILS

//need this static array to iterate over the enum:
const KEY_MAP: [(Key, ScanCode); 101] = [
    (Key::A, ScanCode::A),
    (Key::B, ScanCode::B),
    (Key::C, ScanCode::C),
    (Key::D, ScanCode::D),
    (Key::E, ScanCode::E),
    (Key::F, ScanCode::F),
    (Key::G, ScanCode::G),
    (Key::H, ScanCode::H),
    (Key::I, ScanCode::I),
    (Key::J, ScanCode::J),
    (Key::K, ScanCode::K),
    (Key::L, ScanCode::L),
    (Key::M, ScanCode::M),
    (Key::N, ScanCode::N),
    (Key::O, ScanCode::O),
    (Key::P, ScanCode::P),
    (Key::Q, ScanCode::Q),
    (Key::R, ScanCode::R),
    (Key::S, ScanCode::S),
    (Key::T, ScanCode::T),
    (Key::U, ScanCode::U),
    (Key::V, ScanCode::V),
    (Key::W, ScanCode::W),
    (Key::X, ScanCode::X),
    (Key::Y, ScanCode::Y),
    (Key::Z, ScanCode::Z),
    (Key::Num1, ScanCode::Key1),
    (Key::Num2, ScanCode::Key2),
    (Key::Num3, ScanCode::Key3),
    (Key::Num4, ScanCode::Key4),
    (Key::Num5, ScanCode::Key5),
    (Key::Num6, ScanCode::Key6),
    (Key::Num7, ScanCode::Key7),
    (Key::Num8, ScanCode::Key8),
    (Key::Num9, ScanCode::Key9),
    (Key::Num0, ScanCode::Key0),
    (Key::Return, ScanCode::Return),
    (Key::Escape, ScanCode::Escape),
    (Key::Backspace, ScanCode::Backspace),
    (Key::Tab, ScanCode::Tab),
    (Key::Space, ScanCode::Space),
    (Key::Minus, ScanCode::Minus),
    (Key::Equals, ScanCode::Equals),
    (Key::LeftBracket, ScanCode::LBracket),
    (Key::RightBracket, ScanCode::RBracket),
    (Key::Backslash, ScanCode::Backslash),
    (Key::Semicolon, ScanCode::Semicolon),
    (Key::Comma, ScanCode::Comma),
    (Key::Period, ScanCode::Period),
    (Key::Slash, ScanCode::Slash),
    (Key::CapsLock, ScanCode::Capslock),
    (Key::F1, ScanCode::F1),
    (Key::F2, ScanCode::F2),
    (Key::F3, ScanCode::F3),
    (Key::F4, ScanCode::F4),
    (Key::F5, ScanCode::F5),
    (Key::F6, ScanCode::F6),
    (Key::F7, ScanCode::F7),
    (Key::F8, ScanCode::F8),
    (Key::F9, ScanCode::F9),
    (Key::F10, ScanCode::F10),
    (Key::F11, ScanCode::F11),
    (Key::F12, ScanCode::F12),
    (Key::Pause, ScanCode::Pause),
    (Key::Insert, ScanCode::Insert),
    (Key::Home, ScanCode::Home),
    (Key::PageUp, ScanCode::PageUp),
    (Key::Delete, ScanCode::Delete),
    (Key::End, ScanCode::End),
    (Key::PageDown, ScanCode::PageDown),
    (Key::Right, ScanCode::Right),
    (Key::Left, ScanCode::Left),
    (Key::Down, ScanCode::Down),
    (Key::Up, ScanCode::Up),
    (Key::KpDivide, ScanCode::NumpadDivide),
    (Key::KpMultiply, ScanCode::NumpadMultiply),
    (Key::KpMinus, ScanCode::NumpadSubtract),
    (Key::KpPlus, ScanCode::NumpadAdd),
    (Key::KpEnter, ScanCode::NumpadEnter),
    (Key::Kp1, ScanCode::Numpad1),
    (Key::Kp2, ScanCode::Numpad2),
    (Key::Kp3, ScanCode::Numpad3),
    (Key::Kp4, ScanCode::Numpad4),
    (Key::Kp5, ScanCode::Numpad5),
    (Key::Kp6, ScanCode::Numpad6),
    (Key::Kp7, ScanCode::Numpad7),
    (Key::Kp8, ScanCode::Numpad8),
    (Key::Kp9, ScanCode::Numpad9),
    (Key::Kp0, ScanCode::Numpad0),
    (Key::Power, ScanCode::Power),
    (Key::KpEquals, ScanCode::NumpadEquals),
    (Key::F13, ScanCode::F13),
    (Key::F14, ScanCode::F14),
    (Key::F15, ScanCode::F15),
    (Key::SysReq, ScanCode::Sysrq),
    (Key::LCtrl, ScanCode::LControl),
    (Key::LShift, ScanCode::LShift),
    (Key::LAlt, ScanCode::LAlt),
    (Key::RCtrl, ScanCode::RControl),
    (Key::RShift, ScanCode::RShift),
    (Key::RAlt, ScanCode::RAlt),
];

const BUTTON_MAP: [(JoyPadButton, Button); 14] = [
    (JoyPadButton::A, Button::East),
    (JoyPadButton::B, Button::South),
    (JoyPadButton::X, Button::North),
    (JoyPadButton::Y, Button::West),
    (JoyPadButton::Up, Button::DPadUp),
    (JoyPadButton::Down, Button::DPadDown),
    (JoyPadButton::Left, Button::DPadLeft),
    (JoyPadButton::Right, Button::DPadRight),
    (JoyPadButton::L, Button::LeftShoulder),
    (JoyPadButton::L3, Button::LeftStick),
    (JoyPadButton::R, Button::RightShoulder),
    (JoyPadButton::R3, Button::RightStick),
    (JoyPadButton::Select, Button::Back),
    (JoyPadButton::Start, Button::Start),
];

const AXIS_MAP: [((JoypadAnalog, JoypadAnalogAxis, JoyPadButton), Axis); 6] = [
    ((JoypadAnalog::AnalogLeft, JoypadAnalogAxis::AnalogX, JoyPadButton::A), Axis::LeftX),
    ((JoypadAnalog::AnalogLeft, JoypadAnalogAxis::AnalogY, JoyPadButton::A), Axis::LeftY),
    ((JoypadAnalog::AnalogRight, JoypadAnalogAxis::AnalogX, JoyPadButton::A), Axis::RightX),
    ((JoypadAnalog::AnalogRight, JoypadAnalogAxis::AnalogY, JoyPadButton::A), Axis::RightY),
    ((JoypadAnalog::AnalogButton, JoypadAnalogAxis::L2, JoyPadButton::L2), Axis::TriggerLeft),
    ((JoypadAnalog::AnalogButton, JoypadAnalogAxis::R2, JoyPadButton::R2), Axis::TriggerRight),
];
