mod board;
mod menu;

use bevy::prelude::*;

use bevy::window::PresentMode;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins
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
        .add_systems(Startup, setup)
        .add_plugins((board::BoardPlugin, menu::MenuPlugin))
        .run();
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppState {
    InGame,
    #[default]
    Menu,
}

// Camera

#[derive(Component)]
pub struct CameraMarker;

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), CameraMarker));
}