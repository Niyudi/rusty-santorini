use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use bevy::input::mouse::MouseMotion;
use itertools::Itertools;
use crate::AppState;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<Build>()
            .add_event::<Hold>()
            .add_event::<Movement>()
            .add_event::<NextTurn>()
            .add_event::<PlaceWorker>()
            .add_event::<Release>()
            .add_systems(OnEnter(AppState::InGame), spawn_board)
            .add_systems(
                Update, (
                    (camera_input, update_camera).chain(),
                    build,
                    hold,
                    movement,
                    next_turn,
                    place_worker,
                    release,
                    send_events_debug,
                ).run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), despawn_board);
    }
}

// Resources

#[derive(Resource)]
struct BoardMaterials {
    blue_material: Handle<StandardMaterial>,
    white_material: Handle<StandardMaterial>,
    worker1_material: Handle<StandardMaterial>,
    worker2_material: Handle<StandardMaterial>,
}
impl BoardMaterials {
    fn get_turn_material(&self, turn: &Turn) -> Handle<StandardMaterial> {
        match turn {
            Turn::P1 => self.worker1_material.clone(),
            Turn::P2 => self.worker2_material.clone(),
        }
    }
}

#[derive(Resource)]
enum Turn {
    P1,
    P2,
}
impl Turn {
    fn get_worker_marker(&self) -> WorkerMarker {
        match self {
            Turn::P1 => WorkerMarker::P1,
            Turn::P2 => WorkerMarker::P2,
        }
    }
    fn next(&mut self) {
        *self = match self {
            Turn::P1 => Turn::P2,
            Turn::P2 => Turn::P1,
        }
    }
}

// Components

#[derive(Component)]
struct BoardCamera {
    pitch: f32,
    yaw: f32,
}
impl Default for BoardCamera {
    fn default() -> Self {
        Self { pitch: std::f32::consts::FRAC_PI_4, yaw: 0.0 }
    }
}

#[derive(Component)]
struct BaseMarker;

#[derive(Component)]
struct BoardMarker;

#[derive(Component)]
struct BoardPosition(usize, usize, usize);
impl BoardPosition {
    fn above(&self) -> BoardPosition {
        BoardPosition(self.0, self.1, self.2 + 1)
    }
}

#[derive(Component)]
struct Held;

#[derive(Component)]
struct TurnIndicatorMarker;

#[derive(Component, PartialEq)]
enum WorkerMarker {
    P1,
    P2,
}

// Systems

fn build (
    mut commands: Commands,
    mut ev_build: EventReader<Build>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut selected_query: Query<(&BoardPosition, &mut Pickable, &mut PickSelection), With<BoardMarker>>,
    board_materials: Res<BoardMaterials>,
) {
    'events: for _ in ev_build.read() {
        let position = 'pos: {
            for (position, mut pickable, mut selection) in selected_query.iter_mut() {
                if selection.is_selected {
                    *pickable = Pickable::IGNORE;
                    selection.is_selected = false;
                    break 'pos position.above();
                }
            }
            break 'events;
        };

        let (mesh, material) = match position.2 {
            1 => (meshes.add(shape::Box {
                min_x: -0.45,
                max_x: 0.45,
                min_y: 0.0,
                max_y: 1.0,
                min_z: -0.45,
                max_z: 0.45,
            }.into()), board_materials.white_material.clone()),
            2 => (meshes.add(shape::Box {
                min_x: -0.35,
                max_x: 0.35,
                min_y: 0.0,
                max_y: 1.0,
                min_z: -0.35,
                max_z: 0.35,
            }.into()), board_materials.white_material.clone()),
            3 => (meshes.add(shape::Box {
                min_x: -0.25,
                max_x: 0.25,
                min_y: 0.0,
                max_y: 1.0,
                min_z: -0.25,
                max_z: 0.25,
            }.into()), board_materials.white_material.clone()),
            _ => (meshes.add(shape::Box {
                min_x: -0.25,
                max_x: 0.25,
                min_y: 0.0,
                max_y: 0.35,
                min_z: -0.25,
                max_z: 0.25,
            }.into()), board_materials.blue_material.clone()),
        };

        commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_xyz(position.0 as f32 - 2.0, position.2 as f32 - 1.0, position.1 as f32 - 2.0),
                ..default()
            },
            PickableBundle {
                pickable: if position.2 == 4 { Pickable::IGNORE } else { Pickable::default() },
                ..default()
            },
            position,
            BoardMarker,
        ));

        break;
    }
    ev_build.clear();
}

fn camera_input(
    mut camera_query: Query<&mut BoardCamera>,
    mut mouse_evr: EventReader<MouseMotion>,
    mouse: Res<Input<MouseButton>>,
    window_query: Query<&Window>,
) {
    const SENSITIVITY: Vec2 = Vec2::new(60.0, 30.0);

    if !mouse.pressed(MouseButton::Right) {
        return;
    }

    let window = window_query.single();

    let mut rotation = Vec2::ZERO;
    for ev in mouse_evr.read() {
        rotation = ev.delta / Vec2::new(window.width(), window.height()) * SENSITIVITY;
    }

    let mut camera = camera_query.single_mut();
    camera.pitch = (camera.pitch + rotation.y).clamp(0.0, std::f32::consts::FRAC_PI_2);
    camera.yaw = camera.yaw + rotation.x;
    while camera.yaw < 0.0 {
        camera.yaw += std::f32::consts::TAU;
    }
    while camera.yaw > std::f32::consts::TAU {
        camera.yaw -= std::f32::consts::TAU;
    }
}

fn despawn_board(
    mut commands: Commands,
    base_query: Query<Entity, With<BaseMarker>>,
    board_query: Query<Entity, With<BoardMarker>>,
    workers_query: Query<Entity, With<WorkerMarker>>,
) {
    for entity in base_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in board_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in workers_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    commands.remove_resource::<BoardMaterials>();
    commands.remove_resource::<Turn>();
}

fn hold(
    mut commands: Commands,
    mut ev_hold: EventReader<Hold>,
    mut selected_query: Query<(Entity, &WorkerMarker, &mut Transform, &mut Pickable, &mut PickSelection), Without<Held>>,
    held_query: Query<(), With<Held>>,
    turn: Res<Turn>,
) {
    'events: for  _ in ev_hold.read() {
        if !held_query.is_empty() {
            break;
        }

        let (entity, mut transform) = 'transform: {
            for (entity, worker_marker, transform, mut pickable, mut selection) in selected_query.iter_mut() {
                if selection.is_selected && turn.get_worker_marker() == *worker_marker {
                    *pickable = Pickable::IGNORE;
                    selection.is_selected = false;
                    break 'transform (entity, transform);
                }
            }
            break 'events;
        };

        commands.entity(entity).insert(Held);
        transform.translation.y += 0.75;

        break;
    }
    ev_hold.clear();
}

fn movement(
    mut commands: Commands,
    mut ev_movement: EventReader<Movement>,
    mut held_query: Query<(Entity, &mut Transform, &mut BoardPosition, &mut Pickable), (Without<BoardMarker>, With<Held>)>,
    mut selected_query: Query<(&BoardPosition, &mut Pickable, &mut PickSelection), (With<BoardMarker>, Without<Held>)>,
) {
    'events: for _ in ev_movement.read() {
        if let Ok((entity, mut transform, mut position, mut pickable)) = held_query.get_single_mut() {
            *position = 'pos: {
                for (position, mut pickable, mut selection) in selected_query.iter_mut() {
                    if selection.is_selected {
                        *pickable = Pickable::IGNORE;
                        selection.is_selected = false;
                        break 'pos position.above();
                    }
                }
                break 'events;
            };
            
            commands.entity(entity).remove::<Held>();
            transform.translation = Vec3::new(position.0 as f32 - 2.0, position.2 as f32 - 0.6, position.1 as f32 - 2.0);
            *pickable = Pickable::default();
        }
        break;
    }
    ev_movement.clear();
}

fn next_turn(
    mut commands: Commands,
    mut ev_next_turn: EventReader<NextTurn>,
    mut turn: ResMut<Turn>,
    board_materials: Res<BoardMaterials>,
    held_query: Query<(), With<Held>>,
    turn_indicator_query: Query<Entity, With<TurnIndicatorMarker>>,
) {
    for _ in ev_next_turn.read() {
        if !held_query.is_empty() {
            break;
        }

        turn.next();

        let turn_indicator_entity = turn_indicator_query.single();
        commands.entity(turn_indicator_entity).insert(board_materials.get_turn_material(&turn));

        break;
    }
    ev_next_turn.clear();
}

fn place_worker(
    mut commands: Commands,
    mut ev_place_worker: EventReader<PlaceWorker>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut selected_query: Query<(&BoardPosition, &mut Pickable, &mut PickSelection), With<BoardMarker>>,
    turn: Res<Turn>,
    board_materials: Res<BoardMaterials>,
) {
    'events: for _ in ev_place_worker.read() {
        let position = 'pos: {
            for (position, mut pickable, mut selection) in selected_query.iter_mut() {
                if selection.is_selected {
                    *pickable = Pickable::IGNORE;
                    selection.is_selected = false;
                    break 'pos position.above();
                }
            }
            break 'events;
        };
        let worker_marker = turn.get_worker_marker();

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(shape::Capsule {
                    radius: 0.2,
                    depth: 0.4,
                    ..default()
                }.into()),
                material: board_materials.get_turn_material(&turn),
                transform: Transform::from_xyz(position.0 as f32 - 2.0, position.2 as f32 - 0.6, position.1 as f32 - 2.0),
                ..default()
            },
            PickableBundle::default(),
            position,
            worker_marker,
        ));

        break;
    }
    ev_place_worker.clear();
}

fn release(
    mut commands: Commands,
    mut ev_release: EventReader<Release>,
    mut held_query: Query<(Entity, &mut Transform, &mut Pickable), With<Held>>,
) {
    for _ in ev_release.read() {
        if let Ok((entity, mut transform, mut pickable)) = held_query.get_single_mut() {
            commands.entity(entity).remove::<Held>();
            transform.translation.y -= 0.75;
            *pickable = Pickable::default();
        }
        break;
    }
    ev_release.clear();
}

fn send_events_debug(
    mut ev_build: EventWriter<Build>,
    mut ev_hold: EventWriter<Hold>,
    mut ev_movement: EventWriter<Movement>,
    mut ev_next_turn: EventWriter<NextTurn>,
    mut ev_place_worker: EventWriter<PlaceWorker>,
    mut ev_release: EventWriter<Release>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::S) {
        ev_place_worker.send(PlaceWorker);
    } else if keys.just_pressed(KeyCode::B) {
        ev_build.send(Build);
    } else if keys.just_pressed(KeyCode::N) {
        ev_next_turn.send(NextTurn);
    } else if keys.just_pressed(KeyCode::H) {
        ev_hold.send(Hold);
    } else if keys.just_pressed(KeyCode::M) {
        ev_movement.send(Movement);
    } else if keys.just_pressed(KeyCode::R) {
        ev_release.send(Release);
    }
}

fn spawn_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((Camera3dBundle::default(), BoardCamera::default(), BoardMarker));

    let white_material = materials.add(Color::rgb_u8(250, 254, 255).into());
    let worker1_material = materials.add(StandardMaterial {
        base_color: Color::GOLD,
        metallic: 1.0,
        reflectance: 0.8,
        perceptual_roughness: 0.4,
        ..default()
    });
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::from_corners(
                Vec3::new(-2.7, -0.2, -2.7), Vec3::new(2.7, -0.05, 2.7)).into()),
            material: white_material.clone(),
            ..default()
        },
        BaseMarker,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::from_corners(
                Vec3::new(-2.9, -0.21, -2.9), Vec3::new(2.9, -0.49, 2.9)).into()),
            material: worker1_material.clone(),
            ..default()
        },
        TurnIndicatorMarker,
        BaseMarker,
    ));

    let light_square_material = 
        materials.add(Color::rgb_u8(117, 205, 255).into());
    let dark_square_material = 
        materials.add(Color::rgb_u8(65, 92, 224).into());
    let square_mesh =
        meshes.add(shape::Box::from_corners(Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 0.05, 0.5)).into());
    for (i, j) in (-2..=2).cartesian_product(-2..=2) {
        commands.spawn((
            PbrBundle {
                mesh: square_mesh.clone(),
                material: if (i + j) % 2 == 0 { light_square_material.clone() } else { dark_square_material.clone() },
                transform: Transform::from_xyz(i as f32, -0.05, j as f32),
                ..default()
            },
            PickableBundle::default(),
            BoardPosition((i + 2) as usize, (j + 2) as usize, 0),
            BoardMarker,
        ));
    }

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 10000.0,
                range: 100.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(6.0, 6.0, 0.0),
            ..default()
        },
        BoardMarker,
    ));

    commands.insert_resource(BoardMaterials {
        blue_material: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..default()
        }),
        white_material,
        worker1_material,
        worker2_material: materials.add(StandardMaterial {
            base_color: Color::SILVER,
            metallic: 1.0,
            reflectance: 0.8,
            perceptual_roughness: 0.4,
            ..default()
        }),
    });
    commands.insert_resource(Turn::P1);
}

fn update_camera(
    mut camera_query: Query<(&mut Transform, &BoardCamera)>,
) {
    const DISTANCE: f32 = 10.0;

    let (mut tranform, camera) = camera_query.single_mut();

    let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
    *tranform = Transform::from_xyz(
            DISTANCE * camera.yaw.cos() * pitch_cos,
            DISTANCE * pitch_sin,
            DISTANCE * camera.yaw.sin() * pitch_cos,
        ).looking_at(Vec3::ZERO, Vec3::Y);
}

// Events

#[derive(Event)]
struct Build;

#[derive(Event)]
struct Hold;

#[derive(Event)]
struct Movement;

#[derive(Event)]
struct NextTurn;

#[derive(Event)]
struct PlaceWorker;

#[derive(Event)]
struct Release;