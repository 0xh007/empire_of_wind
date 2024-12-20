use bevy::prelude::*;

pub use components::*;
use systems::*;

use crate::asset_management::AppStates;

mod components;
mod systems;

/// Plugin for managing the player entity within the game.
///
/// The `PlayerPlugin` is responsible for setting up the main player entity in the game world.
/// It registers the necessary components and sets up systems to handle player spawning and control.
///
/// # Components
/// - `Player`: A marker component indicating that an entity is the main player.
///
/// # Systems
/// - `spawn_player`: Spawns the player entity with the necessary components for 3D physics and movement control.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(OnEnter(AppStates::Running), spawn_player);
    }
}
