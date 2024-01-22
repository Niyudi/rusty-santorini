use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use bevy::utils::hashbrown::HashMap;
use itertools::Itertools;
use std::ops::Deref;

use super::{Controller, Controllers};
use crate::{
    AppState,
    board::{Board, Piece, PieceMarker, Turn},
    menus::Paused,
};

pub struct HumanControllerPlugin;
impl Plugin for HumanControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<Clicked>()
            .add_systems(OnEnter(AppState::InGame),
                spawn_controllers
            )
            .add_systems(PreUpdate,(
                (
                    guarantee_pickable,
                    apply_deferred,
                    handle_input,
                ).chain()
            ).run_if(in_state(AppState::InGame).and_then(is_controller_used)))
            .add_systems(Update, (
                pause_pickable,
                run_controllers,
            ).run_if(in_state(AppState::InGame).and_then(is_controller_used)))
            .add_systems(OnExit(AppState::InGame), cleanup);
    }
}

const BLOCK: Pickable = Pickable {
    should_block_lower: true,
    should_emit_events: false,
};

// Structs

enum HumanControllerState {
    PrepPlaceWorker,
    PlaceWorker1,
    PlaceWorker2,
    PrepMovement,
    Movement1,
    Movement2 {
        selected_row: usize,
        selected_column: usize,
        selected_height: usize,
    },
    PrepBuild {
        selected_row: usize,
        selected_column: usize,
    },
    Build,
}

// Components

#[derive(Component)]
struct HumanController {
    turn: Turn,
    state: HumanControllerState,
}

#[derive(Component)]
struct PauseBlockerMarker;

// Systems

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, Or<(With<HumanController>, With<PauseBlockerMarker>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<Controllers>();
}

fn guarantee_pickable(
    mut commands: Commands,
    pickables_query: Query<Entity, (With<PieceMarker>, Without<Pickable>)>,
) {
    for entity in pickables_query.iter() {
        commands.entity(entity).insert(PickableBundle {
            pickable: BLOCK,
            ..default()
        });
    }
}

fn handle_input(
    mut ev_clicked: EventWriter<Clicked>,
    mut pointer_down: EventReader<Pointer<Down>>,
    pieces_query: Query<(Entity, &PieceMarker)>,
) {
    let pieces: HashMap<Entity, &PieceMarker> = pieces_query.iter().collect();

    for Pointer {
        pointer_id: _,
        pointer_location: _,
        target,
        event: _,
    } in pointer_down
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        if let Some(PieceMarker {
            piece: _,
            row,
            column,
            height,
        }) = pieces.get(target) {
            ev_clicked.send(Clicked {
                row: *row,
                column: *column,
                height: *height,
            });
        }
    }
}

fn pause_pickable(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    pause_blocker_query: Query<Entity, With<PauseBlockerMarker>>,
    paused: Res<Paused>,
) {
    if paused.value {
        if pause_blocker_query.get_single().is_err() {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::UVSphere { radius: 9.9, ..default() }.into()),
                    material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0).into()),
                    ..default()
                },
                PauseBlockerMarker,
            ));
        }
    } else {
        if let Ok(entity) = pause_blocker_query.get_single() {
            commands.entity(entity).despawn();
        }
    }
}

fn run_controllers(
    mut board: ResMut<Board>,
    mut controllers: Query<&mut HumanController>,
    mut ev_clicked: EventReader<Clicked>,
    mut pieces_query: Query<(&PieceMarker, &mut Pickable, &mut Transform)>,
) {
    const SELECT: fn(usize, usize, &mut ResMut<Board>, &mut HashMap<(usize, usize, usize), (Mut<Pickable>, Mut<Transform>)>) =
    |row, column, board, world_pieces| {
        if let Some(top_height) = board.get_top(row, column) {
            let (mut pickable, _) = world_pieces.remove(&(row, column, top_height)).unwrap();
            *pickable = Pickable::default();
        }
    };
    const SELECT_IF_REACHABLE: fn(usize, usize, usize, &mut ResMut<Board>, &mut HashMap<(usize, usize, usize), (Mut<Pickable>, Mut<Transform>)>) =
    |row, column, height, board, world_pieces| {
        if let Some(top_height) = board.get_top(row, column) {
            if top_height <= height {
                let (mut pickable, _) = world_pieces.remove(&(row, column, top_height)).unwrap();
                *pickable = Pickable::default();
            }
        }
    };
    const SELECT_NEIGHBOURS: fn(usize, usize, &mut ResMut<Board>, &mut HashMap<(usize, usize, usize), (Mut<Pickable>, Mut<Transform>)>) =
    |row, column, board, world_pieces| {
        if row > 0 && column > 0 {
            SELECT(row - 1, column - 1, board, world_pieces);
        }
        if row > 0 {
            SELECT(row - 1, column, board, world_pieces);
        }
        if row > 0 && column < 4 {
            SELECT(row - 1, column + 1, board, world_pieces);
        }
        if column > 0 {
            SELECT(row, column - 1, board, world_pieces);
        }
        if column < 4 {
            SELECT(row, column + 1, board, world_pieces);
        }
        if row < 4 && column > 0 {
            SELECT(row + 1, column - 1, board, world_pieces);
        }
        if row < 4 {
            SELECT(row + 1, column, board, world_pieces);
        }
        if row < 4 && column < 4 {
            SELECT(row + 1, column + 1, board, world_pieces);
        }
    };
    const SELECT_NEIGHBOURS_IF_REACHABLE: fn(usize, usize, usize, &mut ResMut<Board>, &mut HashMap<(usize, usize, usize), (Mut<Pickable>, Mut<Transform>)>) =
    |row, column, height, board, world_pieces| {
        if row > 0 && column > 0 {
            SELECT_IF_REACHABLE(row - 1, column - 1, height, board, world_pieces);
        }
        if row > 0 {
            SELECT_IF_REACHABLE(row - 1, column, height, board, world_pieces);
        }
        if row > 0 && column < 4 {
            SELECT_IF_REACHABLE(row - 1, column + 1, height, board, world_pieces);
        }
        if column > 0 {
            SELECT_IF_REACHABLE(row, column - 1, height, board, world_pieces);
        }
        if column < 4 {
            SELECT_IF_REACHABLE(row, column + 1, height, board, world_pieces);
        }
        if row < 4 && column > 0 {
            SELECT_IF_REACHABLE(row + 1, column - 1, height, board, world_pieces);
        }
        if row < 4 {
            SELECT_IF_REACHABLE(row + 1, column, height, board, world_pieces);
        }
        if row < 4 && column < 4 {
            SELECT_IF_REACHABLE(row + 1, column + 1, height, board, world_pieces);
        }
    };

    const RAISE: f32 = 0.5;

    if !board.validate_world_pieces(pieces_query.iter().map(|(x, _, _)| x)) {
        return;
    }

    let mut world_pieces: HashMap<(usize, usize, usize), (Mut<Pickable>, Mut<Transform>)> =
        pieces_query
        .iter_mut()
        .map(|(x, y, z)| ((x.row, x.column, x.height), (y, z)))
        .collect();

    for mut controller in controllers.iter_mut() {
        if controller.turn != *board.get_turn() {
            continue;
        }

        match controller.state {
            HumanControllerState::PrepPlaceWorker => {
                for (row, column) in (0..5).cartesian_product(0..5) {
                    if board.get_top(row, column).is_some_and(|x| x == 0) {
                        let (mut pickable, _) = world_pieces.remove(&(row, column, 0)).unwrap();
                        *pickable = Pickable::default();
                    }
                }

                controller.state = HumanControllerState::PlaceWorker1;
            }
            HumanControllerState::PlaceWorker1 => {
                for Clicked { row, column, height } in ev_clicked.read() {
                    board.place_worker(*row, *column, height + 1, controller.turn);

                    let (mut pickable, _) = world_pieces.remove(&(*row, *column, *height)).unwrap();
                    *pickable = BLOCK;

                    controller.state = HumanControllerState::PlaceWorker2;

                    break;
                }
                ev_clicked.clear();
            }
            HumanControllerState::PlaceWorker2 => {
                for Clicked { row, column, height } in ev_clicked.read() {
                    board.place_worker(*row, *column, height + 1, controller.turn);

                    for (mut pickable, _) in world_pieces.into_values() {
                        *pickable = BLOCK;
                    }

                    controller.state = HumanControllerState::PrepMovement;
                    board.next_turn();

                    break;
                }
                ev_clicked.clear();
            }
            HumanControllerState::PrepMovement => {
                let mut workers = Vec::new();
                for PieceMarker { piece, row, column, height } in board.get_pieces() {
                    if let Piece::Worker { turn } = piece {
                        if turn == *board.get_turn() {
                            workers.push((row, column, height));
                        }
                    }
                }

                for pos in workers {
                    let (mut pickable, _) = world_pieces.remove(&pos).unwrap();
                    *pickable = Pickable::default();
                }

                controller.state = HumanControllerState::Movement1;
            }
            HumanControllerState::Movement1 => {
                for Clicked { row, column, height } in ev_clicked.read() {
                    let (mut pickable, mut transform) = world_pieces.remove(&(*row, *column, *height)).unwrap();
                    *pickable = BLOCK;
                    transform.translation.y += RAISE;

                    SELECT_NEIGHBOURS_IF_REACHABLE(*row, *column, *height, &mut board, &mut world_pieces);

                    controller.state = HumanControllerState::Movement2 {
                        selected_row: *row,
                        selected_column: *column,
                        selected_height: *height,
                    };

                    break;
                }
                ev_clicked.clear();
            }
            HumanControllerState::Movement2 { selected_row, selected_column, selected_height } => {
                for Clicked { row, column, height } in ev_clicked.read() {
                    for (ref mut pickable, _) in world_pieces.values_mut() {
                        **pickable = BLOCK;
                    }

                    match board.get_piece(*row, *column, *height) {
                        Some(Piece::Worker { turn: _ }) => {
                            let (mut pickable, mut transform) = world_pieces.remove(&(selected_row, selected_column, selected_height)).unwrap();
                            *pickable = Pickable::default();
                            transform.translation.y -= RAISE;

                            let (mut pickable, mut transform) = world_pieces.remove(&(*row, *column, *height)).unwrap();
                            *pickable = BLOCK;
                            transform.translation.y += RAISE;

                            SELECT_NEIGHBOURS_IF_REACHABLE(*row, *column, *height, &mut board, &mut world_pieces);

                            controller.state = HumanControllerState::Movement2 {
                                selected_row: *row,
                                selected_column: *column,
                                selected_height: *height,
                            };
                        }
                        _ => {
                            board.movement(
                                selected_row, selected_column, selected_height,
                                *row, *column, height + 1);

                            controller.state = HumanControllerState::PrepBuild {
                                selected_row: *row,
                                selected_column: *column,
                            };
                        }
                    }

                    break;
                }
                ev_clicked.clear();
            }
            HumanControllerState::PrepBuild { selected_row, selected_column } => {
                SELECT_NEIGHBOURS(selected_row, selected_column, &mut board, &mut world_pieces);

                controller.state = HumanControllerState::Build;
            }
            HumanControllerState::Build => {
                for Clicked { row, column, height } in ev_clicked.read() {
                    board.build(*row, *column, height + 1);

                    for (mut pickable, _) in world_pieces.into_values() {
                        *pickable = BLOCK;
                    }

                    controller.state = HumanControllerState::PrepMovement;
                    board.next_turn();

                    break;
                }
                ev_clicked.clear();
            }
        }

        break;
    }
}

fn spawn_controllers(
    mut commands: Commands,
    controllers: Res<Controllers>,
) {
    if controllers.p1 == Controller::Human {
        commands.spawn(HumanController {
            turn: Turn::P1,
            state: HumanControllerState::PrepPlaceWorker,
        });
    }
    if controllers.p2 == Controller::Human {
        commands.spawn(HumanController {
            turn: Turn::P2,
            state: HumanControllerState::PrepPlaceWorker,
        });
    }
}

// Run conditions

fn is_controller_used(
    controllers: Res<Controllers>,
) -> bool {
    let Controllers {
        p1,
        p2,
    } = controllers.deref();

    if *p1 == Controller::Human || *p2 == Controller::Human {
        true
    } else {
        false
    }
}

// Events

#[derive(Event)]
struct Clicked {
    row: usize,
    column: usize,
    height: usize,
}
