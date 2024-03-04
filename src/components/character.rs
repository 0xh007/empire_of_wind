use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*};

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

impl MovementAcceleration {
    pub fn acceleration(&self) -> Scalar {
        self.0
    }
}

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

impl MovementDampingFactor {
    pub fn damping_factor(&self) -> Scalar {
        self.0
    }
}

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

impl JumpImpulse {
    pub fn impulse(&self) -> Scalar {
        self.0
    }
}

/// Represents the gravitational force applied to a character controller.
///
/// This component stores the acceleration vector due to gravity, which affects the character's
/// movement by applying a constant force in a specified direction, typically downwards.
#[derive(Component)]
pub struct ControllerGravity(Vector);

impl ControllerGravity {
    /// Retrieves the gravitational acceleration vector for the character controller.
    ///
    /// This vector defines the direction and magnitude of the gravitational force. For example,
    /// a standard Earth-like gravity would be represented as a vector pointing downwards
    /// along the negative Y-axis with a magnitude of approximately 9.81.
    ///
    /// Returns:
    /// A `Vector` representing the current gravitational acceleration affecting the character.
    pub fn gravitational_acceleration(&self) -> Vector {
        self.0
    }
}

/// The maximum angle a slope can have for a character controller to be able to climb and jump. If
/// the slope is steeper than this angle, the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

impl MaxSlopeAngle {
    pub fn angle(&self) -> Scalar {
        self.0
    }
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, PI * 0.45)
    }
}

/// A bundle that contains the components needed for a basic kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    // gravity: ControllerGravity,
    movement: MovementBundle,
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Vector::NEG_Y,
            )
            .with_max_time_of_impact(0.2),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
        self
    }
}
