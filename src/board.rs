use bevy::prelude::*;

use bevy::{
    input::mouse::MouseMotion,
    utils::hashbrown::HashSet,
};
use itertools::Itertools;

use crate::AppState;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame),
                setup
            )
            .add_systems(PreUpdate,
                camera_input.run_if(in_state(AppState::InGame))
            )
            .add_systems(Update,
                (
                    update_camera,
                    update_board,
                ).run_if(in_state(AppState::InGame))
            )
            .add_systems(OnExit(AppState::InGame),
                cleanup
            );
    }
}

// Structs

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum Piece {
    Block,
    Worker {
        player: Player
    },
}

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub enum Player {
    #[default]
    P1,
    P2,
}

// Resources

#[derive(Default, Resource)]
pub struct Board {
    data: [[[Option<Piece> ; 4] ; 4] ; 4],
    turn: Player,
}
impl Board {
    pub fn build(&mut self, row: usize, column: usize, height: usize) {
        if self.data[row][column][height].is_some() {
            panic!("Can't build on ({}, {}, {}) because it's already occupied!", row, column, height);
        }

        self.data[row][column][height] = Some(Piece::Block);
    }
    pub fn movement(&mut self,
        from_row: usize, from_column: usize, from_height: usize,
        to_row: usize, to_column: usize, to_height: usize,
    ) {
        if let Some(Piece::Worker { player: _ }) = self.data[from_row][from_column][from_height] {
            if self.data[to_row][to_column][to_height].is_some() {
                panic!("Can't move to ({}, {}, {}) because it's already occupied!", to_row, to_column, to_height);
            }

            self.data[to_row][to_column][to_height] = self.data[from_row][from_column][from_height];
            self.data[from_row][from_column][from_height] = None;
        } else {
            panic!("Can't move from ({}, {}, {}) because there's no worker there!", from_row, from_column, from_height);
        }
    }
    pub fn next_turn(&mut self) {
        self.turn = match self.turn {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        };
    }
    pub fn place_worker(&mut self, row: usize, column: usize, height: usize, player: Player) {
        if self.data[row][column][height].is_some() {
            panic!("Can't place worker on ({}, {}, {}) because it's already occupied!", row, column, height);
        }

        self.data[row][column][height] = Some(Piece::Worker { player });
    }

    fn get_pieces(&self) -> HashSet<PieceMarker> {
        let mut pieces = HashSet::new();
        for ((row, column), height) in (0..4).cartesian_product(0..4).cartesian_product(0..4) {
            if let Some(piece) = self.data[row][column][height] {
                pieces.insert(PieceMarker {
                    piece,
                    row,
                    column,
                    height,
                });
            }
        }
        pieces
    }
}

#[derive(Resource)]
struct BoardAssets {
    blue_material: Handle<StandardMaterial>,
    level0_height: f32,
    level0_mesh: Handle<Mesh>,
    level1_height: f32,
    level1_mesh: Handle<Mesh>,
    level2_height: f32,
    level2_mesh: Handle<Mesh>,
    level3_height: f32,
    level3_mesh: Handle<Mesh>,
    player1_material: Handle<StandardMaterial>,
    player2_material: Handle<StandardMaterial>,
    white_material: Handle<StandardMaterial>,
    worker_height_offset: f32,
    worker_mesh: Handle<Mesh>,
}
impl BoardAssets {
    fn get_piece(&self, piece_marker: &PieceMarker) -> (Transform, Handle<Mesh>, Handle<StandardMaterial>) {
        let PieceMarker {
            piece,
            row,
            column,
            height,
        } = *piece_marker;

        match piece {
            Piece::Block => match height {
                0 => (
                    Transform::from_xyz(row as f32 - 2.0, self.level0_height, column as f32 - 2.0),
                    self.level0_mesh.clone(),
                    self.white_material.clone(),
                ),
                1 => (
                    Transform::from_xyz(row as f32 - 2.0, self.level1_height, column as f32 - 2.0),
                    self.level1_mesh.clone(),
                    self.white_material.clone(),
                ),
                2 => (
                    Transform::from_xyz(row as f32 - 2.0, self.level2_height, column as f32 - 2.0),
                    self.level2_mesh.clone(),
                    self.white_material.clone(),
                ),
                3 => (
                    Transform::from_xyz(row as f32 - 2.0, self.level3_height, column as f32 - 2.0),
                    self.level3_mesh.clone(),
                    self.blue_material.clone(),
                ),
                _ => panic!("{} is an invalid height!", height),
            },
            Piece::Worker { player } => (
                Transform::from_xyz(
                    row as f32 - 2.0,
                    match height {
                        0 => self.worker_height_offset + self.level0_height,
                        1 => self.worker_height_offset + self.level1_height,
                        2 => self.worker_height_offset + self.level2_height,
                        3 => self.worker_height_offset + self.level3_height,
                        _ => panic!("{} is an invalid height!", height),
                    },
                    column as f32 - 2.0,
                ),
                self.worker_mesh.clone(),
                self.get_player_material(player),
            ),
        }
    }
    fn get_player_material(&self, player: Player) -> Handle<StandardMaterial> {
        match player {
            Player::P1 => self.player1_material.clone(),
            Player::P2 => self.player2_material.clone(),
        }
    }
}

// Components

#[derive(Component)]
struct BaseMarker;

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
pub struct BoardMarker;

#[derive(Component, Eq, Hash, PartialEq)]
pub struct PieceMarker {
    piece: Piece,
    row: usize,
    column: usize,
    height: usize,
}

#[derive(Component, Default)]
struct TurnIndicatorMarker {
    turn: Player,
}

// Systems

fn camera_input(
    mut camera_query: Query<&mut BoardCamera>,
    mut mouse_evr: EventReader<MouseMotion>,
    mouse: Res<Input<MouseButton>>,
    window_query: Query<&Window>,
) {
    const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.001;
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
    camera.pitch = (camera.pitch + rotation.y).clamp(0.0, MAX_PITCH);
    camera.yaw = camera.yaw + rotation.x;
    while camera.yaw < 0.0 {
        camera.yaw += std::f32::consts::TAU;
    }
    while camera.yaw > std::f32::consts::TAU {
        camera.yaw -= std::f32::consts::TAU;
    }
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, Or<(With<BaseMarker>, With<BoardMarker>, With<PieceMarker>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<AmbientLight>();
    commands.remove_resource::<Board>();
    commands.remove_resource::<BoardAssets>();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Recurring assets
    let board_assets = BoardAssets {
        blue_material: materials.add(Color::BLUE.into()),
        level0_height: 0.0,
        level0_mesh: meshes.add(shape::Box {
            min_x: -0.475,
            max_x: 0.475,
            min_y: 0.0,
            max_y: 1.0,
            min_z: -0.475,
            max_z: 0.475,
        }.into()),
        level1_height: 1.0,
        level1_mesh: meshes.add(shape::Box {
            min_x: -0.425,
            max_x: 0.425,
            min_y: 0.0,
            max_y: 0.8,
            min_z: -0.425,
            max_z: 0.425,
        }.into()),
        level2_height: 1.8,
        level2_mesh: meshes.add(shape::Box {
            min_x: -0.4,
            max_x: 0.4,
            min_y: 0.0,
            max_y: 0.6,
            min_z: -0.4,
            max_z: 0.4,
        }.into()),
        level3_height: 2.4,
        level3_mesh: meshes.add(shape::Box {
            min_x: -0.4,
            max_x: 0.4,
            min_y: 0.0,
            max_y: 0.25,
            min_z: -0.4,
            max_z: 0.4,
        }.into()),
        white_material: materials.add(Color::rgb_u8(250, 254, 255).into()),
        player1_material: materials.add(StandardMaterial {
            base_color: Color::GOLD,
            metallic: 1.0,
            reflectance: 0.8,
            perceptual_roughness: 0.4,
            ..default()
        }),
        player2_material: materials.add(StandardMaterial {
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
            material: board_assets.player1_material.clone(),
            ..default()
        },
        TurnIndicatorMarker::default(),
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
            BoardMarker,
        ));
    }

    // Inserts resources
    commands.insert_resource(Board::default());
    commands.insert_resource(board_assets);
}

fn update_board(
    mut commands: Commands,
    mut turn_indicator_query: Query<(Entity, &mut TurnIndicatorMarker)>,
    board: Res<Board>,
    board_assets: Res<BoardAssets>,
    pieces_query: Query<(Entity, &PieceMarker)>,
) {
    let mut board_pieces = board.get_pieces();

    for (entity, piece_marker) in pieces_query.iter() {
        if !board_pieces.remove(piece_marker) {
            commands.entity(entity).despawn();
        }
    }

    for piece_marker in board_pieces.into_iter() {
        let (transform, mesh, material) = board_assets.get_piece(&piece_marker);
        commands.spawn((
            PbrBundle {
                transform,
                mesh,
                material,
                ..default()
            },
            piece_marker,
        ));
    }

    let (entity, mut turn_indicator_marker) = turn_indicator_query.single_mut();
    if turn_indicator_marker.turn != board.turn {
        turn_indicator_marker.turn = board.turn;
        commands.entity(entity).insert(board_assets.get_player_material(board.turn));
    }
}

fn update_camera(
    mut camera_query: Query<(&mut Transform, &BoardCamera)>,
) {
    const DISTANCE: f32 = 10.0;

    let (mut tranform, camera) = camera_query.single_mut();

    let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
    let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
    *tranform = Transform::from_xyz(
        DISTANCE * yaw_cos * pitch_cos,
        DISTANCE * pitch_sin,
        DISTANCE * yaw_sin * pitch_cos,
    ).looking_at(Vec3::ZERO, Vec3::Y);
}
