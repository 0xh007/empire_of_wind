use crate::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::egui::epaint::tessellator::Path;
use bevy_xpbd_3d::{math::*, prelude::*};
use big_brain::prelude::*;
use oxidized_navigation::NavMeshAffector;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_npc);
    }
}

fn spawn_npc(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Define the starting point for the NPCs.
    let start_position = Vec3::new(-10.0, 15.0, 4.0);
    let spacing = 1.0; // Spacing between each NPC.

    let move_and_sleep = Steps::build()
        .label("MoveAndSleep")
        .step(MoveToNearest::<SleepArea> {
            speed: 1.5,
            _marker: std::marker::PhantomData,
        })
        .step(Sleep {
            until: 10.0,
            per_second: 15.0,
        });

    let position = start_position + Vec3::new(0.0, 0.0, spacing * 1.0);

    commands.spawn((
        Name::new("NPC"),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.4,
                ..default()
            })),
            material: materials.add(Color::YELLOW.into()),
            transform: Transform::from_translation(position),
            ..default()
        },
        CharacterControllerBundle::new(Collider::capsule(1.0, 0.4), Vector::NEG_Y * 9.81 * 2.0)
            .with_movement(90.0, 0.92, 7.0, (30.0 as Scalar).to_radians()),
        Npc,
        Fatigue {
            is_sleeping: false,
            per_second: 4.0,
            level: 60.0,
        },
        NavigationPath::default(),
        Thinker::build()
            .label("NPC Thinker")
            // Selects the action with the highest score that is above the threshold
            .picker(FirstToScore::new(0.6))
            .when(FatigueScorer, move_and_sleep),
    ));
}
