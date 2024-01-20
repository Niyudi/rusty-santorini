mod main_menu;
mod pause_menu;

use bevy::prelude::*;

use main_menu::MainMenuPlugin;
use pause_menu::PauseMenuPlugin;

pub use pause_menu::Paused;

pub struct MenusPlugin;
impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((MainMenuPlugin, PauseMenuPlugin));
    }
}