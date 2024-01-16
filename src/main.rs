mod board;
mod controller;
mod menu;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use bevy::window::PresentMode;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins
            .set(
                WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoVsync,
                        prevent_default_event_handling: true,
                        resizable: false,
                        resolution: (1270., 720.).into(),
                        title: "Santorini".to_string(),
                        ..default()
                    }),
                    ..default()
                }
            )
        )
        .add_plugins(DefaultPickingPlugins)
        .add_state::<AppState>()
        .add_systems(PostStartup, picking_setup)
        .add_plugins((board::BoardPlugin, menu::MenuPlugin))
        .run();
}

// States

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

// Setup

fn picking_setup(
    mut global_highlight: ResMut<GlobalHighlight<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    *global_highlight = GlobalHighlight {
        hovered: materials.add(Color::rgb(0.25, 0.65, 0.25).into()),
        pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
    };
}
