use bevy::prelude::*;

use bevy::input::mouse::MouseMotion;
use bevy_mod_picking::PickableBundle;
use itertools::Itertools;
use crate::AppState;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), spawn_board)
            .add_systems(Update, (camera_input, update_camera)
                .chain().run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), despawn_board);
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
) {
    for entity in board_query.iter() {
        commands.entity(entity).despawn_recursive();
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