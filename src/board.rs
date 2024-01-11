use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use bevy::input::mouse::MouseMotion;
use itertools::Itertools;
use crate::AppState;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlaceWorker>()
            .add_systems(OnEnter(AppState::InGame), spawn_board)
            .add_systems(
                Update, (
                    (camera_input, update_camera).chain(),
                    place_worker,
                    send_place_worker,
                ).run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), despawn_board);
    }
}

// Resources

#[derive(Resource)]
struct BoardMaterials {
    white_material: Handle<StandardMaterial>,
    worker1_material: Handle<StandardMaterial>,
    worker2_material: Handle<StandardMaterial>,
}
impl BoardMaterials {
    fn get_worker_material(&self, worker_marker: &WorkerMarker) -> Handle<StandardMaterial> {
        match worker_marker {
            WorkerMarker::P1 => self.worker1_material.clone(),
            WorkerMarker::P2 => self.worker2_material.clone(),
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
struct BoardMarker;

#[derive(Component)]
struct BoardPosition(usize, usize, usize);
impl BoardPosition {
    fn above(&self) -> BoardPosition {
        BoardPosition(self.0, self.1, self.2 + 1)
    }
}

#[derive(Component)]
enum WorkerMarker {
    P1,
    P2,
}

// Systems

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
    board_query: Query<Entity, With<BoardMarker>>,
    workers_query: Query<Entity, With<WorkerMarker>>,
) {
    for entity in board_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in workers_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    commands.remove_resource::<BoardMaterials>();
    commands.remove_resource::<Turn>();
}

fn place_worker(
    mut commands: Commands,
    mut ev_place_worker: EventReader<PlaceWorker>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut selected_query: Query<(&BoardPosition, &mut Pickable, &mut PickSelection), With<BoardMarker>>,
    mut turn: ResMut<Turn>,
    board_materials: Res<BoardMaterials>,
) {
    for _ in ev_place_worker.read() {
        let position = 'pos: {
            for (position, mut pickable, mut selection) in selected_query.iter_mut() {
                if selection.is_selected && position.2 < 4 {
                    *pickable = Pickable::IGNORE;
                    selection.is_selected = false;
                    break 'pos position.above();
                }
            }
            return;
        };
        let worker_marker = turn.get_worker_marker();

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(shape::Capsule {
                    radius: 0.2,
                    depth: 0.4,
                    ..default()
                }.into()),
                material: board_materials.get_worker_material(&worker_marker),
                transform: Transform::from_xyz(position.0 as f32 - 2.0, position.2 as f32 - 0.6, position.1 as f32 - 2.0),
                ..default()
            },
            PickableBundle::default(),
            position,
            worker_marker,
        ));

        turn.next();
        break;
    }
    ev_place_worker.clear();
}

fn send_place_worker(
    mut ev_place_worker: EventWriter<PlaceWorker>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::S) {
        ev_place_worker.send(PlaceWorker)
    }
}

fn spawn_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((Camera3dBundle::default(), BoardCamera::default(), BoardMarker));

    let white_material = materials.add(Color::rgb_u8(250, 254, 255).into());
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::from_corners(
                Vec3::new(-2.7, -0.2, -2.7), Vec3::new(2.7, -0.05, 2.7)).into()),
            material: white_material.clone(),
            ..default()
        },
        BoardMarker,
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
        white_material,
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

#[derive(Event)]
struct PlaceWorker;