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

#[derive(PartialEq)]
pub enum Controller {
    Human,
}