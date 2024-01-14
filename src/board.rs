use bevy::prelude::*;

use bevy::input::mouse::MouseMotion;
use itertools::Itertools;
use std::{
    fmt::Display, 
    ops::{Index, IndexMut},
};

use crate::{
    AppState,
    controller::{Controller, ControllersPlugin},
};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ControllersPlugin)
            .add_event::<Build>()
            .add_event::<Movement>()
            .add_event::<NextTurn>()
            .add_event::<PlaceWorker>()
            .init_resource::<Turn>()
            .add_systems(OnEnter(AppState::InGame), spawn_board)
            .add_systems(
                Update, (
                    (
                        camera_input,
                        update_camera,
                    ).chain(),
                    build,
                    movement,
                    next_turn,
                    place_worker,
                ).run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), despawn_board);
    }
}

// Resources

#[derive(Resource)]
struct BoardAssets {
    blue_material: Handle<StandardMaterial>,
    level1_height: f32,
    level1_mesh: Handle<Mesh>,
    level2_height: f32,
    level2_mesh: Handle<Mesh>,
    level3_height: f32,
    level3_mesh: Handle<Mesh>,
    level4_height: f32,
    level4_mesh: Handle<Mesh>,
    white_material: Handle<StandardMaterial>,
    worker1_material: Handle<StandardMaterial>,
    worker2_material: Handle<StandardMaterial>,
    worker_height_offset: f32,
    worker_mesh: Handle<Mesh>,
}
impl BoardAssets {
    fn get_block(&self, board_position: &BoardPosition) -> (Transform, Handle<Mesh>, Handle<StandardMaterial>) {
        match board_position.height {
            1 => (
                Transform::from_xyz(board_position.row as f32 - 2.0, self.level1_height, board_position.column as f32 - 2.0),
                self.level1_mesh.clone(), self.white_material.clone()
            ),
            2 => (
                Transform::from_xyz(board_position.row as f32 - 2.0, self.level2_height, board_position.column as f32 - 2.0),
                self.level2_mesh.clone(), self.white_material.clone()
            ),
            3 => (
                Transform::from_xyz(board_position.row as f32 - 2.0, self.level3_height, board_position.column as f32 - 2.0),
                self.level3_mesh.clone(), self.white_material.clone()
            ),
            4 => (
                Transform::from_xyz(board_position.row as f32 - 2.0, self.level4_height, board_position.column as f32 - 2.0),
                self.level4_mesh.clone(), self.blue_material.clone()
            ),
            _ => panic!("{} is an invalid height for a block!", board_position.height),
        }
    }
    fn get_turn_material(&self, turn: &Turn) -> Handle<StandardMaterial> {
        match turn {
            Turn::P1 => self.worker1_material.clone(),
            Turn::P2 => self.worker2_material.clone(),
        }
    }
    fn get_worker(&self, board_position: &BoardPosition, turn: &Turn) -> (Transform, Handle<Mesh>, Handle<StandardMaterial>) {
        (
            Transform::from_translation(self.get_worker_translation(board_position)),
            self.worker_mesh.clone(),
            self.get_turn_material(turn),
        )
    }
    fn get_worker_translation(&self, board_position: &BoardPosition) -> Vec3 {
        Vec3::new(
            board_position.row as f32 - 2.0,
            match board_position.height {
                1 => self.worker_height_offset + self.level1_height,
                2 => self.worker_height_offset + self.level2_height,
                3 => self.worker_height_offset + self.level3_height,
                4 => self.worker_height_offset + self.level4_height,
                _ => panic!("{} is an invalid height for a worker!", board_position.height),
            },
            board_position.column as f32 - 2.0,
        )
    }
}

#[derive(Default, Resource)]
struct BoardOccupation {
    data: [[[bool ; 5] ; 5] ; 5],
}
impl Index<BoardPosition> for BoardOccupation {
    type Output = bool;

    fn index(&self, index: BoardPosition) -> &Self::Output {
        &self.data[index.row][index.column][index.height]
    }
}
impl IndexMut<BoardPosition> for BoardOccupation {
    fn index_mut(&mut self, index: BoardPosition) -> &mut Self::Output {
        &mut self.data[index.row][index.column][index.height]
    }
}

#[derive(Resource)]
pub struct Controllers {
    pub p1: Controller,
    pub p2: Controller,
}

#[derive(Resource)]
pub enum Turn {
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
impl FromWorld for Turn {
    fn from_world(_world: &mut World) -> Self {
        Turn::P1
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
pub struct BoardMarker;

#[derive(Clone, Component, Copy, PartialEq)]
struct BoardPosition {
    pub row: usize,
    pub column: usize,
    pub height: usize,
}
impl BoardPosition {
    fn new(row: usize, column: usize, height: usize) -> Self {
        BoardPosition {
            row,
            column,
            height,
        }
    }
}
impl Display for BoardPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.row, self.column, self.height)
    }
}

#[derive(Component)]
struct TurnIndicatorMarker;

#[derive(Component, PartialEq)]
pub enum WorkerMarker {
    P1,
    P2,
}

// Systems

fn build (
    mut board_occupation: ResMut<BoardOccupation>,
    mut commands: Commands,
    mut ev_build: EventReader<Build>,
    board_assets: Res<BoardAssets>,
) {
    for Build { position } in ev_build.read() {
        let (transform, mesh, material) = board_assets.get_block(position);

        board_occupation[*position] = true;

        commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            *position,
            BoardMarker,
        ));
    }
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
    
    commands.remove_resource::<AmbientLight>();
    commands.remove_resource::<BoardAssets>();
    commands.remove_resource::<BoardOccupation>();
    commands.remove_resource::<Turn>();
}

fn movement(
    mut board_occupation: ResMut<BoardOccupation>,
    mut ev_movement: EventReader<Movement>,
    mut worker_query: Query<(&BoardPosition, &mut Transform), With<WorkerMarker>>,
    board_assets: Res<BoardAssets>,
) {
    for Movement { from, to } in ev_movement.read() {
        if board_occupation[*to] {
            panic!("Move event failed: {} is already occupied!", to);
        }

        let mut transform = 'transform: {
            for (position, transform) in worker_query.iter_mut() {
                if position == from {
                    break 'transform transform;
                }
            }
            panic!("Move event failed: there's no worker in {}!", from);
        };

        board_occupation[*from] = false;
        board_occupation[*to] = true;

        transform.translation = board_assets.get_worker_translation(from);
    }
}

fn next_turn(
    mut commands: Commands,
    mut ev_next_turn: EventReader<NextTurn>,
    mut turn: ResMut<Turn>,
    board_assets: Res<BoardAssets>,
    turn_indicator_query: Query<Entity, With<TurnIndicatorMarker>>,
) {
    for _ in ev_next_turn.read() {
        turn.next();
        commands.entity(turn_indicator_query.single()).insert(board_assets.get_turn_material(&turn));
    }
}

fn place_worker(
    mut commands: Commands,
    mut ev_place_worker: EventReader<PlaceWorker>,
    board_assets: Res<BoardAssets>,
    board_occupation: Res<BoardOccupation>,
    turn: Res<Turn>,
) {
    for PlaceWorker { position } in ev_place_worker.read() {
        if board_occupation[*position] {
            panic!("Place worker event failed: {} is already occupied!", position);
        }

        let (transform, mesh, material) = board_assets.get_worker(position, &turn);

        commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            *position,
            turn.get_worker_marker(),
        ));
    }
}

fn spawn_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Recurring assets
    let board_assets = BoardAssets {
        blue_material: materials.add(Color::BLUE.into()),
        level1_height: 0.0,
        level1_mesh: meshes.add(shape::Box {
            min_x: -0.475,
            max_x: 0.475,
            min_y: 0.0,
            max_y: 1.0,
            min_z: -0.475,
            max_z: 0.475,
        }.into()),
        level2_height: 1.0,
        level2_mesh: meshes.add(shape::Box {
            min_x: -0.425,
            max_x: 0.425,
            min_y: 0.0,
            max_y: 0.8,
            min_z: -0.425,
            max_z: 0.425,
        }.into()),
        level3_height: 1.8,
        level3_mesh: meshes.add(shape::Box {
            min_x: -0.4,
            max_x: 0.4,
            min_y: 0.0,
            max_y: 0.6,
            min_z: -0.4,
            max_z: 0.4,
        }.into()),
        level4_height: 2.4,
        level4_mesh: meshes.add(shape::Box {
            min_x: -0.4,
            max_x: 0.4,
            min_y: 0.0,
            max_y: 0.25,
            min_z: -0.4,
            max_z: 0.4,
        }.into()),
        white_material: materials.add(Color::rgb_u8(250, 254, 255).into()),
        worker1_material: materials.add(StandardMaterial {
            base_color: Color::GOLD,
            metallic: 1.0,
            reflectance: 0.8,
            perceptual_roughness: 0.4,
            ..default()
        }),
        worker2_material: materials.add(StandardMaterial {
            base_color: Color::SILVER,
            metallic: 1.0,
            reflectance: 0.8,
            perceptual_roughness: 0.4,
            ..default()
        }),
        worker_height_offset: 0.4,
        worker_mesh: meshes.add(shape::Capsule {
            radius: 0.2,
            depth: 0.4,
            ..default()
        }.into()),
    };

    // Camera
    commands.spawn((Camera3dBundle::default(), BoardCamera::default(), BaseMarker));

    // Lights
    commands.insert_resource(AmbientLight {
        color: Color::rgb(1.0, 0.8, 0.7),
        brightness: 0.4,
    });
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
        BaseMarker,
    ));
    
    // Base
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::from_corners(
                Vec3::new(-2.7, -0.2, -2.7), Vec3::new(2.7, -0.05, 2.7)).into()),
            material: board_assets.white_material.clone(),
            ..default()
        },
        BaseMarker,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::from_corners(
                Vec3::new(-2.9, -0.21, -2.9), Vec3::new(2.9, -0.49, 2.9)).into()),
            material: board_assets.worker1_material.clone(),
            ..default()
        },
        TurnIndicatorMarker,
        BaseMarker,
    ));

    // Level 0 board
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
            BoardPosition::new((i + 2) as usize, (j + 2) as usize, 0),
            BoardMarker,
        ));
    }

    // Inserts resources
    commands.insert_resource(board_assets);
    commands.insert_resource(BoardOccupation::default());
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
struct Build {
    position: BoardPosition,
}

#[derive(Event)]
struct Movement {
    from: BoardPosition,
    to: BoardPosition,
}

#[derive(Event)]
struct NextTurn;

#[derive(Event)]
struct PlaceWorker {
    position: BoardPosition,
}
