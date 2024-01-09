use bevy::prelude::*;

use bevy::{
    input::mouse::MouseMotion,
    window::PrimaryWindow,
};
use itertools::Itertools;
use crate::{AppState, CameraMarker};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), spawn_board)
            .add_systems(Update, orbit_mouse.run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), despawn_board);
    }
}

// Components

#[derive(Component)]
struct BoardMarker;

// Systems

fn despawn_board(
    mut commands: Commands,
    board_query: Query<Entity, With<BoardMarker>>,
) {
    for entity in board_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn orbit_mouse(
    mut camera_query: Query<&mut Transform, With<CameraMarker>>,
    mut mouse_evr: EventReader<MouseMotion>,
    mouse: Res<Input<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if !mouse.pressed(MouseButton::Right) {
        return;
    }

    let mut rotation = 0.0;
    for ev in mouse_evr.read() {
        rotation = ev.delta.x * 10.0;
    }

    let mut camera_transform = camera_query.single_mut();

    if rotation > 0.0 {
        let window = window_query.single();
        let delta = rotation / window.width() * std::f32::consts::PI;

        let yaw = Quat::from_rotation_y(-delta);
        camera_transform.rotation = yaw * camera_transform.rotation;
    }

    let rot_matrix = Mat3::from_quat(camera_transform.rotation);
    camera_transform.translation = rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, 6.0));
    camera_transform.translation.y = 4.5;
}

fn spawn_board(
    mut camera_query: Query<&mut Transform, With<CameraMarker>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut camera_transform = camera_query.single_mut();
    *camera_transform = Transform::from_xyz(0.0, 4.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y);

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
            BoardMarker,
        ));
    }

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 8000.0,
                range: 100.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 6.0, 0.0),
            ..default()
        },
        BoardMarker,
    ));
}