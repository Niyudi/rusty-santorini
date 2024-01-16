use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use bevy::utils::{HashMap, HashSet};

use crate::{
    AppState,
    board::{BoardMarker, BoardOccupation, BoardPosition, Controllers, Lock, Movement, NextTurn, PlaceWorker, Turn, WorkerMarker},
    controller::Controller,
};

const RAISE: f32 = 0.5;

pub struct HumanControllerPlugin;
impl Plugin for HumanControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<Clicked>()
            .add_state::<HumanControllerState>()
            .add_systems(OnEnter(AppState::InGame), setup_human_controller.run_if(is_controller_used))
            .add_systems(OnExit(AppState::InGame), cleanup_human_controller)
            .add_systems(Update, (
                handle_input.run_if(is_controller_turn),
            ).run_if(in_state(AppState::InGame)))
            .add_systems(OnEnter(HumanControllerState::PlaceWorker),
                (
                    guarantee_pickable,
                    apply_deferred,
                    place_worker_pickable,
                ).chain()
            )
            .add_systems(Update, place_worker.run_if(in_state(HumanControllerState::PlaceWorker)))
            .add_systems(OnExit(HumanControllerState::PlaceWorker), clear_pickable)
            .add_systems(OnEnter(HumanControllerState::PlaceWorkerLock), next_turn_and_lock)
            .add_systems(Update, wait_lock.run_if(in_state(HumanControllerState::PlaceWorkerLock)))
            .add_systems(OnEnter(HumanControllerState::Move),
                (
                    guarantee_pickable,
                    apply_deferred,
                    select_worker_pickable,
                ).chain()
            )
            .add_systems(Update, move_worker.run_if(in_state(HumanControllerState::Move)))
            .add_systems(OnExit(HumanControllerState::Move), clear_pickable)
            .add_systems(Update, wait_lock.run_if(in_state(HumanControllerState::MoveLock)));
    }
}

// States

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum HumanControllerState {
    #[default]
    Idle,
    PlaceWorker,
    PlaceWorkerLock,
    Move,
    MoveLock,
    Build,
}
impl HumanControllerState {
    fn next_state(&self) -> Self {
        match self {
            HumanControllerState::Idle => HumanControllerState::Idle,
            HumanControllerState::PlaceWorker => HumanControllerState::PlaceWorkerLock,
            HumanControllerState::PlaceWorkerLock => HumanControllerState::Move,
            HumanControllerState::Move => HumanControllerState::MoveLock,
            HumanControllerState::MoveLock => HumanControllerState::Build,
            HumanControllerState::Build => todo!(),
        }
    }

}

// Resource

#[derive(Default, Resource)]
struct PlacedWorkers {
    p1: usize,
    p2: usize,
}
impl PlacedWorkers {
    fn add_worker(&mut self, turn: &Turn) {
        match turn {
            Turn::P1 => self.p1 += 1,
            Turn::P2 => self.p2 += 1,
        }
    }
    fn get_workers(&self, turn: &Turn) -> usize {
        match turn {
            Turn::P1 => self.p1,
            Turn::P2 => self.p2,
        }
    }
}

// Components

#[derive(Component)]
struct Selected;

// Systems

fn cleanup_human_controller(
    mut commands: Commands,
) {
    commands.remove_resource::<PlacedWorkers>();
}

fn clear_pickable(
    mut pickables_query: Query<&mut Pickable, Or<(With<BoardMarker>, With<WorkerMarker>)>>,
) {
    for mut pickable in pickables_query.iter_mut() {
        *pickable = Pickable::IGNORE;
    }
}

fn guarantee_pickable(
    mut commands: Commands,
    pickables_query: Query<Entity, (Or<(With<BoardMarker>, With<WorkerMarker>)>, Without<Pickable>)>,
) {
    for entity in pickables_query.iter() {
        commands.entity(entity).insert(PickableBundle {
            pickable: Pickable::IGNORE,
            ..default()
        });
    }
}

fn handle_input(
    mut ev_clicked: EventWriter<Clicked>,
    mut pointer_down: EventReader<Pointer<Down>>,
    pickables_query: Query<(Entity, &BoardPosition), With<Pickable>>,
) {
    let pickables: HashMap<Entity, &BoardPosition> = pickables_query.iter().collect();

    for Pointer {
        pointer_id: _,
        pointer_location: _,
        target,
        event: _,
    } in pointer_down
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        if let Some(board_position) = pickables.get(target) {
            ev_clicked.send(Clicked::Position(**board_position));
        } else {
            ev_clicked.send(Clicked::None);
        }
    }

}

fn place_worker(
    mut board_query: Query<(&BoardPosition, &mut Pickable), With<BoardMarker>>,
    mut ev_clicked: EventReader<Clicked>,
    mut ev_next_turn: EventWriter<NextTurn>,
    mut ev_place_worker: EventWriter<PlaceWorker>,
    mut lock: ResMut<Lock>,
    mut next_state: ResMut<NextState<HumanControllerState>>,
    mut placed_workers: ResMut<PlacedWorkers>,
    turn: Res<Turn>,
) {
    if placed_workers.get_workers(&turn) == 2 {
        match *turn {
            Turn::P1 => {
                lock.lock();
                ev_next_turn.send(NextTurn);
            }
            Turn::P2 => {
                next_state.set(HumanControllerState::PlaceWorkerLock);
                return;
            }
        }
    }

    let mut board_pickables: HashMap<BoardPosition, Mut<Pickable>> = board_query.iter_mut().map(|(x, y)| (*x, y)).collect();

    for clicked in ev_clicked.read() {
        if lock.is_locked() {
            continue;
        }
        
        if let Clicked::Position(clicked_position) = clicked {
            ev_place_worker.send(PlaceWorker { position: clicked_position.above() });
            lock.lock();
            placed_workers.add_worker(&turn);
            let pickable = board_pickables.get_mut(clicked_position).unwrap();
            **pickable = Pickable::IGNORE;
        }
    }
}

fn place_worker_pickable(
    mut board_query: Query<(&BoardPosition, &mut Pickable), With<BoardMarker>>,
    workers_query: Query<&BoardPosition, With<WorkerMarker>>,
) {
    let occupied_positions: HashSet<&BoardPosition> = workers_query.iter().collect();

    for (board_position, mut pickable) in board_query.iter_mut() {
        if !occupied_positions.contains(board_position) {
            *pickable = Pickable::default();
        }
    }
}

fn move_worker(
    mut board_query: Query<(&BoardPosition, &mut Pickable), (With<BoardMarker>, Without<Selected>, Without<WorkerMarker>)>,
    mut commands: Commands,
    mut ev_clicked: EventReader<Clicked>,
    mut ev_movement: EventWriter<Movement>,
    mut lock: ResMut<Lock>,
    mut next_state: ResMut<NextState<HumanControllerState>>,
    mut selected_query: Query<(Entity, &BoardPosition, &mut Transform, &mut Pickable), (Without<BoardMarker>, With<Selected>, With<WorkerMarker>)>,
    mut workers_query: Query<(Entity, &BoardPosition, &mut Transform, &mut Pickable), (Without<BoardMarker>, Without<Selected>, With<WorkerMarker>)>,
    board_occupation: Res<BoardOccupation>,
) {
    for clicked in ev_clicked.read() {
        match clicked {
            Clicked::Position(clicked_position) => {
                if let Ok((entity, board_position, _, _)) = selected_query.get_single_mut() {
                    lock.lock();
                    ev_movement.send(Movement {
                        from: *board_position,
                        to: clicked_position.above(),
                    });
                    commands.entity(entity).remove::<Selected>();
                    next_state.set(HumanControllerState::MoveLock);
                } else {
                    for (entity, board_position, mut transform, mut pickable) in workers_query.iter_mut() {
                        if board_position == clicked_position {
                            transform.translation.y += RAISE;
                            commands.entity(entity).insert(Selected);
                            
                            *pickable = Pickable::IGNORE;
                            
                            let mut board_pickables: HashMap<BoardPosition, Mut<Pickable>> = board_query.iter_mut().map(|(x, y)| (*x, y)).collect();

                            for (row, column) in board_position.neighbours() {
                                if let Some(height) = board_occupation.top(row, column) {
                                    let pickable_board = board_pickables.get_mut(&BoardPosition::new(row, column, height - 1)).unwrap();
                                    **pickable_board = Pickable::default();
                                }
                            }
                        }
                    }
                }
            }
            Clicked::None => {
                if let Ok((entity, _, mut transform, mut pickable)) = selected_query.get_single_mut() {
                    transform.translation.y -= RAISE;
                    commands.entity(entity).remove::<Selected>();
                    *pickable = Pickable::default();

                    for (_, mut pickable) in board_query.iter_mut() {
                        *pickable = Pickable::IGNORE;
                    }
                }
            }
        } 
    }
}

fn next_turn_and_lock(
    mut ev_next_turn: EventWriter<NextTurn>,
    mut lock: ResMut<Lock>,
) {
    if !lock.is_locked() {
        lock.lock();
        ev_next_turn.send(NextTurn);
    }
}


fn select_worker_pickable(
    mut worker_query: Query<(&WorkerMarker, &mut Pickable)>,
    turn: Res<Turn>,
) {
    let turn_worker_marker = turn.get_worker_marker();
    for (worker_marker, mut pickable) in worker_query.iter_mut() {
        if turn_worker_marker == *worker_marker {
            *pickable = Pickable::default(); 
        }
    }
}

fn setup_human_controller(
    mut commands: Commands,
    mut next_state: ResMut<NextState<HumanControllerState>>,
) {
    commands.insert_resource(PlacedWorkers::default());
    next_state.set(HumanControllerState::PlaceWorker);
}

fn wait_lock(
    mut next_state: ResMut<NextState<HumanControllerState>>,
    lock: Res<Lock>,
    state: Res<State<HumanControllerState>>,
) {
    if !lock.is_locked() {
        next_state.set(state.next_state())
    }
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
            };
        } 
    }
    false
}

fn is_controller_used(
    world: &World,
) -> bool {
    if let Some(Controllers {
        p1,
        p2,
    }) = world.get_resource::<Controllers>() {
        if *p1 == Controller::Human || *p2 == Controller::Human {
            return true;
        }
    }
    false
}

// Events

#[derive(Event)]
enum Clicked {
    Position(BoardPosition),
    None,
}