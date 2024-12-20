use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A marker component used to designate an area that a player can enter.
///
/// The `AreaEnterMarker` component is used to identify entities that represent
/// areas in the game world where specific actions should be taken when a player
/// enters them. It works in conjunction with the `AreaExitMarker` component and
/// the `manage_active_areas` system to handle area entry and exit events.
///
/// # Usages
/// - Adding the component to an entity marks it as an area entrance.
/// - Used in systems to detect when a player enters the marked area.
#[derive(Debug, Clone, Eq, PartialEq, Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AreaEnterMarker;
