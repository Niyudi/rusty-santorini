mod human;

use bevy::prelude::*;

use human::HumanControllerPlugin;

use crate::AppState;

pub struct ControllersPlugin;
impl Plugin for ControllersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(HumanControllerPlugin)
            .add_systems(Update,
                wait_resources.run_if(in_state(AppState::LoadGame))
            );
    }
}

// Structs

#[derive(PartialEq)]
pub enum Controller {
    Human,
}

// Resources

#[derive(Resource)]
pub struct Controllers {
    pub p1: Controller,
    pub p2: Controller,
}

// Systems

fn wait_resources(
    mut next_state: ResMut<NextState<AppState>>,
    world: &World,
) {
    if world.get_resource::<Controllers>().is_some() {
        next_state.set(AppState::InGame);
    }
}