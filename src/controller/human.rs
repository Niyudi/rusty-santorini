use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{
    AppState,
    board::{BoardMarker, Controllers, Turn, WorkerMarker},
    controller::Controller,
};

pub struct HumanControllerPlugin;
impl Plugin for HumanControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update,(
                guarantee_pickable,
                (
                    handle_input,
                ).run_if(is_controller_turn)

            ).run_if(in_state(AppState::InGame)));
    }
}

// Systems

fn guarantee_pickable(
    mut commands: Commands,
    board_query: Query<Entity, (With<BoardMarker>, Without<Pickable>)>,
    worker_query: Query<Entity, (With<WorkerMarker>, Without<Pickable>)>,
) {
    for entity in board_query.iter() {
        commands.entity(entity).insert(PickableBundle {
            pickable: Pickable::IGNORE,
            ..default()
        });
    }
    for entity in worker_query.iter() {
        commands.entity(entity).insert(PickableBundle {
            pickable: Pickable::IGNORE,
            ..default()
        });
    }
    
}

fn handle_input(
) {

}

// Run conditions

fn is_controller_turn(
    world: &World,
) -> bool {
    if let Some(turn) = world.get_resource::<Turn>() {
        if let Some(controllers) = world.get_resource::<Controllers>() {
            return match *turn {
                Turn::P1 => controllers.p1 == Controller::Human,
                Turn::P2 => controllers.p2 == Controller::Human,
            }
        } 
    }
    false
}