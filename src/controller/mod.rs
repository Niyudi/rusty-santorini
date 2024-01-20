mod human;

use bevy::prelude::*;

use human::HumanControllerPlugin;

use crate::AppState;

pub struct ControllersPlugin;
impl Plugin for ControllersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<ControllersState>()
            .add_plugins(HumanControllerPlugin)
            .add_systems(Update,
                start_controllers.run_if(in_state(AppState::InGame).and_then(run_once()))
            );
    }
}

// States

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum ControllersState {
    #[default]
    Idle,
    Running,
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

fn start_controllers(
    mut next_state: ResMut<NextState<ControllersState>>,
) {
    next_state.set(ControllersState::Running);
}