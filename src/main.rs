mod board;
mod menu;

use bevy::prelude::*;

use bevy::render::{
    RenderPlugin,
    settings::{Backends, RenderCreation, WgpuSettings},
};
use bevy::window::PresentMode;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins
            // .set(RenderPlugin {
            //     render_creation: RenderCreation::Automatic(WgpuSettings {
            //         backends: Some(Backends::VULKAN),
            //         ..default()
            //     })
            // })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync,
                    prevent_default_event_handling: true,
                    resizable: false,
                    resolution: (1270., 720.).into(),
                    title: "Santorini".to_string(),
                    ..default()
                }),
                ..default()
            })
        )
        .add_state::<AppState>()
        .add_plugins((board::BoardPlugin, menu::MenuPlugin))
        .run();
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppState {
    InGame,
    #[default]
    Menu,
}