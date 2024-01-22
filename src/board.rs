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
                    check_win,
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
pub enum Piece {
    Block,
    Board,
    Worker {
        turn: Turn
    },
}

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub enum Turn {
    #[default]
    P1,
    P2,
    WinP1,
    WinP2,
}

// Resources

#[derive(Resource)]
pub struct Board {
    data: [[[Option<Piece> ; 5] ; 5] ; 5],
    turn: Turn,
}
impl Board {
    pub fn build(&mut self, row: usize, column: usize, height: usize) {
        if self.data[row][column][height].is_some() {
            panic!("Can't build on ({}, {}, {}) because it's already occupied!", row, column, height);
        }

        self.data[row][column][height] = Some(Piece::Block);
    }
    pub fn get_piece(&self, row: usize, column: usize, height: usize) -> Option<&Piece> {
        self.data[row][column][height].as_ref()
    }
    pub fn get_pieces(&self) -> HashSet<PieceMarker> {
        let mut pieces = HashSet::new();
        for ((row, column), height) in (0..5).cartesian_product(0..5).cartesian_product(1..5) {
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
    pub fn get_top(&self, row: usize, column: usize) -> Option<usize> {        
        for height in 1..5 {
            match self.data[row][column][height] {
                Some(Piece::Block) => continue,
                None => return Some(height - 1),
                _ => return None,
            }
        }
        None
    }
    pub fn get_turn(&self) -> &Turn {
        &self.turn
    }
    pub fn movement(&mut self,
        from_row: usize, from_column: usize, from_height: usize,
        to_row: usize, to_column: usize, to_height: usize,
    ) {
        if let Some(Piece::Worker { turn: _ }) = self.data[from_row][from_column][from_height] {
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
            Turn::P1 => Turn::P2,
            Turn::P2 => Turn::P1,
            _ => self.turn,
        };
    }
    pub fn place_worker(&mut self, row: usize, column: usize, height: usize, turn: Turn) {
        if self.data[row][column][height].is_some() {
            panic!("Can't place worker on ({}, {}, {}) because it's already occupied!", row, column, height);
        }

        self.data[row][column][height] = Some(Piece::Worker { turn });
    }
    pub fn validate_world_pieces<'a, I>(&self, piece_markers: I) -> bool
        where I: Iterator<Item = &'a PieceMarker>
    {
        let mut pieces = [[[None ; 5] ; 5] ; 5];
        for PieceMarker { piece, row, column, height } in piece_markers {
            pieces[*row][*column][*height] = Some(*piece);
        }

        for ((row, column), height) in (0..5).cartesian_product(0..5).cartesian_product(0..5) {
            if self.data[row][column][height] != pieces[row][column][height] {
                return false;
            }
        }

        true
    }
}
impl Default for Board {
    fn default() -> Self {
        let mut data: [[[Option<Piece>; 5]; 5]; 5] = Default::default();
        for (row, column) in (0..5).cartesian_product(0..5) {
            data[row][column][0] = Some(Piece::Board);
        }

        Self {
            data,
            turn: Turn::default(),
        }
    }
}

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
                    self.white_material.clone(),
                ),
                4 => (
                    Transform::from_xyz(row as f32 - 2.0, self.level4_height, column as f32 - 2.0),
                    self.level4_mesh.clone(),
                    self.blue_material.clone(),
                ),
                _ => panic!("{} is an invalid height!", height),
            },
            Piece::Board => panic!("Can't spawn more board pieces!"),
            Piece::Worker { turn } => (
                Transform::from_xyz(
                    row as f32 - 2.0,
                    match height {
                        1 => self.worker_height_offset + self.level1_height,
                        2 => self.worker_height_offset + self.level2_height,
                        3 => self.worker_height_offset + self.level3_height,
                        4 => self.worker_height_offset + self.level4_height,
                        _ => panic!("{} is an invalid height!", height),
                    },
                    column as f32 - 2.0,
                ),
                self.worker_mesh.clone(),
                self.get_turn_material(turn),
            ),
        }
    }
    fn get_turn_material(&self, turn: Turn) -> Handle<StandardMaterial> {
        match turn {
            Turn::P1 | Turn::WinP1 => self.player1_material.clone(),
            Turn::P2 | Turn::WinP2 => self.player2_material.clone(),
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

#[derive(Clone, Component, Copy, Eq, Hash, PartialEq)]
pub struct PieceMarker {
    pub piece: Piece,
    pub row: usize,
    pub column: usize,
    pub height: usize,
}

#[derive(Component, Default)]
struct TurnIndicatorMarker {
    turn: Turn,
}

#[derive(Component)]
struct WinText;

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

fn check_win(
    mut board: ResMut<Board>,
) {
    const IS_NEIGHBOUR_REACHABLE: fn(usize, usize, usize, &mut ResMut<Board>) -> bool =
    |row, column, height, board| {
        if row > 0 && column > 0 {
            if IS_REACHABLE(row - 1, column - 1, height, board) {
                return true;
            }
        }
        if row > 0 {
            if IS_REACHABLE(row - 1, column, height, board) {
                return true;
            }
        }
        if row > 0 && column < 4 {
            if IS_REACHABLE(row - 1, column + 1, height, board) {
                return true;
            }
        }
        if column > 0 {
            if IS_REACHABLE(row, column - 1, height, board) {
                return true;
            }
        }
        if column < 4 {
            if IS_REACHABLE(row, column + 1, height, board) {
                return true;
            }
        }
        if row < 4 && column > 0 {
            if IS_REACHABLE(row + 1, column - 1, height, board) {
                return true;
            }
        }
        if row < 4 {
            if IS_REACHABLE(row + 1, column, height, board) {
                return true;
            }
        }
        if row < 4 && column < 4 {
            if IS_REACHABLE(row + 1, column + 1, height, board) {
                return true;
            }
        }
        false
    };
    const IS_REACHABLE: fn(usize, usize, usize, &mut ResMut<Board>) -> bool =
    |row, column, height, board| {
        if let Some(top_height) = board.get_top(row, column) {
            top_height <= height
        } else {
            false
        }
    };

    let mut p1_exists = false;
    let mut p1_smothered = true;
    let mut p2_exists = false;
    let mut p2_smothered = true;
    for PieceMarker { piece, row, column, height } in board.get_pieces() {
        if let Piece::Worker { turn } = piece {
            match turn {
                Turn::P1 => {
                    if height == 4 {
                        board.turn = Turn::WinP1;
                        return;
                    }
                    p1_exists = true;
                    if IS_NEIGHBOUR_REACHABLE(row, column, height, &mut board) {
                        p1_smothered = false;
                    }
                }
                Turn::P2 => {
                    if height == 4 {
                        board.turn = Turn::WinP2;
                        return;
                    }
                    p2_exists = true;
                    if IS_NEIGHBOUR_REACHABLE(row, column, height, &mut board) {
                        p2_smothered = false;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    if p1_exists && p1_smothered {
        board.turn = Turn::WinP2;
    } else if p2_exists && p2_smothered {
        board.turn = Turn::WinP1;
    }
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, Or<(With<BoardCamera>, With<BaseMarker>, With<PieceMarker>, With<WinText>)>>,
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
        brightness: 0.6,
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
            PieceMarker {
                piece: Piece::Board,
                row: (i + 2) as usize,
                column: (j + 2) as usize,
                height: 0,
            },
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
    win_text_query: Query<(), With<WinText>>,   
    
) {
    match board.get_turn() {
        Turn::WinP1 => if win_text_query.is_empty() {
            commands.spawn((
                TextBundle {
                    text: Text::from_section("Gold wins!", TextStyle {
                        color: Color::GREEN,
                        font_size: 24.0,
                        ..default()
                    }),
                    style: Style {
                        position_type: PositionType::Absolute,
                        right: Val::Px(5.0),
                        top: Val::Px(5.0),
                        ..default()
                    },
                    ..default()
                },
                WinText,
            ));
        },
        Turn::WinP2 => if win_text_query.is_empty() {
            commands.spawn((
                TextBundle {
                    text: Text::from_section("Silver wins!", TextStyle {
                        color: Color::GREEN,
                        font_size: 24.0,
                        ..default()
                    }),
                    style: Style {
                        position_type: PositionType::Absolute,
                        right: Val::Px(5.0),
                        top: Val::Px(5.0),
                        ..default()
                    },
                    ..default()
                },
                WinText,
            ));
        },
        _ => {}
    }

    let mut board_pieces = board.get_pieces();

    for (entity, piece_marker) in pieces_query.iter() {
        if !board_pieces.remove(piece_marker) {
            if piece_marker.height > 0 {
                commands.entity(entity).despawn();
            }
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
        commands.entity(entity).insert(board_assets.get_turn_material(board.turn));
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
