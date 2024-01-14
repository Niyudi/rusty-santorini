use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn handle_input(
    mut ev_pointer_down: EventReader<Pointer<Down>>,
) {
    for pointer in ev_pointer_down.read() {
        
    }
}