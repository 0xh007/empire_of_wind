use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*, SubstepSchedule, SubstepSet};

use crate::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>()
            .add_systems(Update, update_grounded)
            .add_systems(Update, apply_deferred)
            // .add_systems(Update, apply_gravity)
            .add_systems(Update, movement)
            .add_systems(Update, apply_movement_damping);
        // .add_systems(
        //     SubstepSchedule,
        //     kinematic_controller_collisions.in_set(SubstepSet::SolveUserConstraints),
        // );
    }
}

/// Updates the ['Grounded'] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal that isn't too
        // steep
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                rotation.rotate(-hit.normal2).angle_between(Vector::Y).abs() <= angle.angle()
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Responds to ['MovementAction'] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<
        (
            &MovementAcceleration,
            &JumpImpulse,
            &mut LinearVelocity,
            Has<Grounded>,
        ),
        With<Player>,
    >,
) {
    // Precision is adjusted so that the example works with both the 'f32' and 'f64' features.
    let delta_time = time.delta_seconds();

    for event in movement_event_reader.read() {
        for (movement_acceleration, jump_impulse, mut linear_velocity, is_grounded) in
            &mut controllers
        {
            match event {
                MovementAction::Move(direction) => {
                    linear_velocity.x +=
                        direction.x * movement_acceleration.acceleration() * delta_time;
                    linear_velocity.z -=
                        direction.y * movement_acceleration.acceleration() * delta_time;
                }
                MovementAction::Jump => {
                    if is_grounded {
                        linear_velocity.y = jump_impulse.impulse();
                    }
                }
            }
        }
    }
}

/// Applies ['ControllerGravity'] to character controllers.
fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&ControllerGravity, &mut LinearVelocity)>,
) {
    // Precision is adjusted so that the example works with both the 'f32' and 'f64' features.
    let delta_time = time.delta_seconds();

    for (gravity, mut linear_velocity) in &mut controllers {
        linear_velocity.0 += gravity.gravitational_acceleration() * delta_time;
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        linear_velocity.x *= damping_factor.damping_factor();
        linear_velocity.z *= damping_factor.damping_factor();
    }
}

/// This system performs very basic collision response for kinematic character controllers by
/// pushing them along their contact normals by the current penetration depths.
#[allow(clippy::type_complexity)]
fn kinematic_controller_collisions(
    collisions: Res<Collisions>,
    collider_parents: Query<&ColliderParent, Without<Sensor>>,
    mut character_controllers: Query<
        (
            &RigidBody,
            &mut Position,
            &Rotation,
            &mut LinearVelocity,
            Option<&MaxSlopeAngle>,
        ),
        With<CharacterController>,
    >,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // If the collision didn't happen during this substep, skip the collision
        if !contacts.during_current_substep {
            continue;
        }

        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([collider_parent1, collider_parent2]) =
            collider_parents.get_many([contacts.entity1, contacts.entity2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first or second entity in
        // the collision
        let is_first: bool;
        let (rb, mut position, rotation, mut linear_velocity, max_slope_angle) =
            if let Ok(character) = character_controllers.get_mut(collider_parent1.get()) {
                is_first = true;
                character
            } else if let Ok(character) = character_controllers.get_mut(collider_parent2.get()) {
                is_first = false;
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers
        if !rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.global_normal1(rotation)
            } else {
                -manifold.global_normal2(rotation)
            };

            // Solve each penetrating contact in the manifold
            for contact in manifold.contacts.iter().filter(|c| c.penetration > 0.0) {
                position.0 += normal * contact.penetration;
            }

            // If the slope isn't too steep to walk on but the character is falling, reset vertical
            // velocity.
            if max_slope_angle
                .is_some_and(|angle| normal.angle_between(Vector::Y).abs() <= angle.angle())
                && linear_velocity.y < 0.0
            {
                linear_velocity.y = linear_velocity.y.max(0.0);
            }
        }
    }
}
