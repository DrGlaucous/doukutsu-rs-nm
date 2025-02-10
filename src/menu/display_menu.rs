use itertools::Itertools;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::shared_game_state::{SharedGameState, WindowMode};
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};


/////////////////////////////////////


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MainMenuEntry {
    FullscreenMode, //fullscreen/not
    FixedRatioMode, //use fixed ratios or fill the screen
    Ratios, //what ratios to use
    Back,
}

impl Default for MainMenuEntry {
    fn default() -> Self {
        MainMenuEntry::FullscreenMode
    }
}

pub struct DisplayMenu {
    main: Menu<MainMenuEntry>,
    ratio_count: i32,
}


impl DisplayMenu {

    pub fn new() -> DisplayMenu {
        let main = Menu::new(0, 0, 220, 0);

        DisplayMenu {
            main,
            ratio_count: 0,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {


        #[cfg(not(any(target_os = "android", target_os = "horizon", feature = "backend-libretro")))]
        self.main.push_entry(
            MainMenuEntry::FullscreenMode,
            MenuEntry::Options(
                state.loc.t("menus.options_menu.graphics_menu.window_mode.entry").to_owned(),
                state.settings.window_mode as usize,
                vec![
                    state.loc.t("menus.options_menu.graphics_menu.window_mode.windowed").to_owned(),
                    state.loc.t("menus.options_menu.graphics_menu.window_mode.fullscreen").to_owned(),
                ],
            ),
        );

        self.main.push_entry(
            MainMenuEntry::FixedRatioMode,
            MenuEntry::Toggle(
                format!("Fixed ratio:"),
                state.settings.fixed_ratio,
            ),
        );

        self.ratio_count = 2;
        self.main.push_entry(
            MainMenuEntry::Ratios,
            MenuEntry::Options(
                format!("Ingame Ratio:"),
                state.settings.window_mode as usize,
                vec![
                    format!("4:3"),
                    format!("16:9"),
                ],
            ),
        );


        self.main.push_entry(MainMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));


        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.main.update_width(state);
        self.main.update_height(state);
        self.main.x = ((state.canvas_size.0 - self.main.width as f32) / 2.0).floor() as isize;
        self.main.y = 30 + ((state.canvas_size.1 - self.main.height as f32) / 2.0).floor() as isize;

    }

    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {
        self.update_sizes(state);

        match self.main.tick(controller, state) {
            MenuSelectionResult::Selected(MainMenuEntry::FullscreenMode, toggle)
            | MenuSelectionResult::Right(MainMenuEntry::FullscreenMode, toggle, _)
            | MenuSelectionResult::Left(MainMenuEntry::FullscreenMode, toggle, _) => {
                if let MenuEntry::Options(_, value, _) = toggle {
                    let (new_mode, new_value) = match *value {
                        0 => (WindowMode::Fullscreen, 1),
                        1 => (WindowMode::Windowed, 0),
                        _ => unreachable!(),
                    };

                    *value = new_value;
                    state.settings.window_mode = new_mode;

                    let _ = state.settings.save(ctx);
                }
            }
            MenuSelectionResult::Selected(MainMenuEntry::Ratios, toggle)
            | MenuSelectionResult::Right(MainMenuEntry::Ratios, toggle, _)
            | MenuSelectionResult::Left(MainMenuEntry::Ratios, toggle, _) => {
                if let MenuEntry::Options(stringer, value, _) = toggle {
                    
                    //let stringer = format!("4:3");

                    let current_ratio = state.settings.viewport_ratio;

                    //parse string setting to get new ratio size (this allows new ratios to be set in the localization files without recompiling)
                    let new_ratio = if let Some((x,y)) = stringer.split(":").map(|a| {a.parse::<f32>().unwrap_or(-1.0)}).collect_tuple() {
                        if x > 0.0 && y > 0.0 {
                            (x,y)
                        } else {
                            current_ratio
                        }
                    } else {
                        current_ratio
                    };
                    state.settings.viewport_ratio = new_ratio;
                    
                    //switch between ratio options
                    let mut new_value= *value as i32 + 1;
                    if new_value >= self.ratio_count {
                        new_value = 0;
                    }
                    *value = new_value as usize;

                    let _ = state.settings.save(ctx);
                }
            }

            MenuSelectionResult::Selected(MainMenuEntry::FixedRatioMode, toggle) => {
                if let MenuEntry::Toggle(_, value) = toggle {
                    state.settings.fixed_ratio = !state.settings.fixed_ratio;
                    let _ = state.settings.save(ctx);

                    *value = state.settings.fixed_ratio;

                }
            }

            MenuSelectionResult::Selected(MainMenuEntry::Back, _) | MenuSelectionResult::Canceled => exit_action(),
            _ => (),
        }

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.main.draw(state, ctx)?;

        Ok(())
    }
}












