use bevy::prelude::*;

use crate::{AppState, CameraMarker};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), spawn_board)
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
    let entity = board_query.single();
    commands.entity(entity).despawn_recursive();
}

fn spawn_board(
    mut camera_query: Query<&mut Transform, With<CameraMarker>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut camera_transform = camera_query.single_mut();
    *camera_transform = Transform::from_xyz(3.5, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);

    let white_material = materials.add(Color::rgb(0.95, 0.95, 1.00).into());

    let base_mesh = meshes.add(shape::Box::from_corners(
        Vec3::new(-3.0, -0.2, -3.0), Vec3::new(3.0, 0.0, 3.0)).into());

    commands.spawn(PbrBundle {
        mesh: base_mesh,
        material: white_material.clone(),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 6.0, 0.0),
        ..default()
    });
}