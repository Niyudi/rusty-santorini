mod human;

use bevy::prelude::*;

use human::HumanControllerPlugin;

pub struct ControllersPlugin;
impl Plugin for ControllersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(HumanControllerPlugin);
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