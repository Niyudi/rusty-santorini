mod board;
mod menu;

use bevy::prelude::*;

use bevy::window::PresentMode;
use bevy_mod_picking::{
    DefaultPickingPlugins,
    highlight::GlobalHighlight,
    selection::SelectionSettings,
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
        .add_plugins(DefaultPickingPlugins)
        .add_state::<AppState>()
        .add_systems(PostStartup, picking_setup)
        .add_plugins((board::BoardPlugin, menu::MenuPlugin))
        .run();
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppState {
    InGame,
    #[default]
    Menu,
}

// Setup

fn picking_setup(
    mut global_highlight: ResMut<GlobalHighlight<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selection_settings: ResMut<SelectionSettings>,
) {
    let highlight_material = materials.add(Color::rgb(0.25, 0.65, 0.25).into());
    let click_material = materials.add(Color::rgb(0.35, 0.75, 0.35).into());

    *global_highlight = GlobalHighlight {
        hovered: highlight_material.clone(),
        pressed: click_material,
        selected: highlight_material,
    };

    *selection_settings = SelectionSettings {
        click_nothing_deselect_all: true,
        use_multiselect_default_inputs: false,
    };
}