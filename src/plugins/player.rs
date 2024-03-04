use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*};

use crate::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Spawning player");
    commands.spawn((
        Name::new("Player"),
        Player,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.4,
                ..default()
            })),
            material: materials.add(Color::YELLOW.into()),
            transform: Transform::from_xyz(-14.0, 13.5, -0.14),
            ..default()
        },
        CharacterControllerBundle::new(Collider::capsule(1.0, 0.4)).with_movement(
            30.0,
            0.92,
            7.0,
            (30.0 as Scalar).to_radians(),
        ),
        Friction::new(0.6)
            .with_dynamic_coefficient(0.5)
            .with_static_coefficient(0.6)
            .with_combine_rule(CoefficientCombine::Average),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(2.0),
    ));
}
