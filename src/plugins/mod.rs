use bevy::app::{PluginGroup, PluginGroupBuilder};

mod atmosphere;
mod camera;
mod character;
mod fatigue;
mod input;
mod navmesh;
mod npc;
mod ocean;
mod player;
mod ship;
mod ship_builder;
mod sleep;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(atmosphere::AtmospherePlugin)
            .add(camera::CameraPlugin)
            .add(character::CharacterPlugin)
            .add(fatigue::FatiguePlugin)
            .add(input::InputPlugin)
            .add(navmesh::NavMeshPlugin)
            .add(npc::NpcPlugin)
            .add(ocean::OceanPlugin)
            .add(player::PlayerPlugin)
            .add(ship_builder::ShipBuilderPlugin)
            .add(sleep::SleepPlugin)
        // .add(ship::ShipPlugin)
    }
}
