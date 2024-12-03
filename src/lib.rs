//#![cfg_attr(target_os = "horizon", feature(restricted_std))]

#[macro_use]
extern crate log;
extern crate strum;
#[macro_use]
extern crate strum_macros;

mod common;
mod components;
mod data;
#[cfg(feature = "discord-rpc")]
pub mod discord;
#[cfg(feature = "editor")]
mod editor;
mod engine_constants;
mod entity;
pub mod framework; //this is new
pub mod game;
mod graphics;
mod i18n;
mod input;
mod live_debugger;
mod macros;
mod menu;
mod mod_list;
mod mod_requirements;
pub mod scene; // originally private
pub mod sound; // originally private
mod util;