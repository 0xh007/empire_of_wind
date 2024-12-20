use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// The `DebugCamera` component is used to mark an entity as a debug camera in the game.
/// This component allows systems to identify and manipulate the debug camera specifically.
///
/// The debug camera is typically used for development and debugging purposes, providing
/// an alternative perspective that can be toggled during gameplay to help developers
/// inspect and diagnose issues in the game world.
///
/// # Usage
///
/// The `DebugCamera` component is attached to a camera entity during setup. This camera can
/// have various settings tailored for debugging, such as a freeform movement or different
/// rendering layers.
///
/// The `DebugCamera` component is integral to providing a versatile camera for development
/// and debugging, ensuring that it interacts correctly with various systems and inputs designed
/// for debugging purposes.
#[derive(Debug, Clone, Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct DebugCamera;
