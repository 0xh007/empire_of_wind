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
            mesh: meshes.add(Capsule3d {
                radius: 0.4,
                ..default()
            }),
            material: materials.add(Color::YELLOW),
            transform: Transform::from_xyz(-14.0, 13.5, -0.14),
            ..default()
        },
        CharacterControllerBundle::new(Collider::capsule(1.0, 0.4), Vector::NEG_Y * 9.81 * 2.0)
            .with_movement(90.0, 0.92, 7.0, (30.0 as Scalar).to_radians()),
    ));
}
