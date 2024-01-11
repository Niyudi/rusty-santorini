mod board;
mod menu;

use bevy::prelude::*;

use bevy::window::PresentMode;
use bevy_mod_picking::{
    backends::raycast,
    highlight::{GlobalHighlight, HighlightPlugin, HighlightPluginSettings, PickHighlight},
    input,
    picking_core,
};

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
        .add_plugins((
            picking_core::CorePlugin,
            picking_core::InteractionPlugin,
            input::InputPlugin,
            raycast::RaycastBackend,
        ))
        .init_resource::<HighlightPluginSettings>()
        .register_type::<PickHighlight>()
        .register_type::<HighlightPluginSettings>()
        .add_plugins(HighlightPlugin::<StandardMaterial> {
            highlighting_default: |mut assets| GlobalHighlight {
                hovered: assets.add(Color::rgb(0.35, 0.75, 0.35).into()),
                pressed: assets.add(Color::rgb(0.35, 0.75, 0.35).into()),
            },
        })
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
