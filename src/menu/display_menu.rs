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
}


impl DisplayMenu {

    pub fn new() -> DisplayMenu {
        let main = Menu::new(0, 0, 220, 0);

        DisplayMenu {
            main,
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
                state.loc.t("menus.options_menu.graphics_menu.fixed_ratio").to_owned(),
                state.settings.fixed_ratio,
            ),
        );

        

        //generate a string list of ratios in engine_constants/mod.rs
        //and determine what ratio we've currently got set
        let mut ratio_list = Vec::new();
        let mut ratio_index = 0; //by default, if nothing matches, we use the first index
        for (idx, ratio) in (&state.constants.viewport_ratios.iter()).clone().enumerate() {

            //check if current ratio and desired ratio are "equal"
            if (state.settings.viewport_ratio.0 - ratio.0).abs() < f32::EPSILON
            && (state.settings.viewport_ratio.1 - ratio.1).abs() < f32::EPSILON {
                ratio_index = idx;
            }
            
            ratio_list.push(format!("{}:{}", ratio.0, ratio.1));
        }


        
        self.main.push_entry(
            MainMenuEntry::Ratios,
            MenuEntry::Options(
                state.loc.t("menus.options_menu.graphics_menu.ratio").to_owned(),
                ratio_index,
                ratio_list,
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

        //re-use for each case
        fn update_ratio(
            state: &mut SharedGameState,
            ctx: &mut Context,
            toggle: &mut MenuEntry,
            step: i32,
        ) -> GameResult {
            if let MenuEntry::Options(_name, value, _options) = toggle {

                let ratio_count = state.constants.viewport_ratios.len() as i32;

                //switch between ratio options with wrapping
                let mut new_value= *value as i32 + step;
                if new_value >= ratio_count {
                    new_value -= ratio_count;
                } else if new_value < 0 {
                    new_value += ratio_count
                }
                *value = new_value as usize; //apply to menu

                //set new ratio
                state.settings.viewport_ratio = state.constants.viewport_ratios[*value];

                //save new setting
                let _ = state.settings.save(ctx);

                state.handle_resize(ctx)?;
            }

            Ok(())
        }

        let menu_tick_result = self.main.tick(controller, state);
        match menu_tick_result {
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
            
            MenuSelectionResult::Selected(MainMenuEntry::FixedRatioMode, toggle) => {
                if let MenuEntry::Toggle(_, value) = toggle {
                    state.settings.fixed_ratio = !state.settings.fixed_ratio;
                    let _ = state.settings.save(ctx);
                    *value = state.settings.fixed_ratio;
                    state.handle_resize(ctx)?;
                    
                    let _ = state.settings.save(ctx);
                }
            }            
            
            MenuSelectionResult::Selected(MainMenuEntry::Ratios, toggle)
            | MenuSelectionResult::Right(MainMenuEntry::Ratios, toggle, _) => {
                update_ratio(state, ctx, toggle, 1)?; //step right
            }
            MenuSelectionResult::Left(MainMenuEntry::Ratios, toggle, _) => {
                update_ratio(state, ctx, toggle, -1)?; //step left
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












