use crate::prelude::*;
use bevy::prelude::*;
use bevy_water::*;

const WATER_HEIGHT: f32 = 2.0;

pub struct OceanPlugin;

impl Plugin for OceanPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaterSettings {
            height: WATER_HEIGHT,
            ..default()
        })
        .add_plugins(WaterPlugin)
        .add_systems(Update, update_water_interactables);
    }
}

fn update_water_interactables(
    water: WaterParam,
    mut water_interactables: Query<(&WaterInteractable, &mut Transform, &GlobalTransform)>,
    #[cfg(feature = "debug")] mut lines: ResMut<DebugLines>,
) {
    for (water_interactable, mut transform, global) in water_interactables.iter_mut() {
        let pos = global.translation();
        #[cfg(not(feature = "debug"))]
        water_interactable.sync_with_water(&water, pos, &mut transform);
        #[cfg(feature = "debug")]
        ship.update(&water, pos, &mut transform, &mut lines);
    }
}
