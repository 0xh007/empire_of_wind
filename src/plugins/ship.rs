use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy_asset_loader::prelude::*;
use bevy_water::WaterParam;
use bevy_xpbd_3d::components::ExternalForce;
use bevy_xpbd_3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(OnEnter(AppStates::Next), spawn_cube)
            .add_systems(Update, calculate_and_apply_buoyancy)
            .register_type::<BuoyancyMarker>()
            .add_systems(
                Update,
                read_buoyancy_objects.run_if(in_state(AppStates::Next)),
            )
            .add_systems(
                Update,
                update_voxel_solidity.run_if(in_state(AppStates::Next)),
            )
            // .add_systems(
            //     Update,
            //     visualize_voxel_grid.run_if(in_state(AppStates::Next)),
            // )
            // .add_systems(
            //     Update,
            //     visualize_ship_bounds.run_if(in_state(AppStates::Next)),
            // )
            .configure_loading_state(
                LoadingStateConfig::new(AppStates::AssetLoading).load_collection::<ShipAssets>(),
            )
            .add_systems(OnEnter(AppStates::Next), spawn_ship);
        // .add_systems(OnEnter(AppStates::Next), spawn_furniture)
        // .add_systems(OnEnter(AppStates::Next), spawn_food);
    }
}

const VOXEL_SIZE: f32 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vec3I {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3I {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Vec3I { x, y, z }
    }
}

#[derive(Component)]
struct VoxelVisual;

#[derive(Component)]
struct Buoyancy {
    voxels: Vec<Voxel>, // List of voxel data, possibly pulled from generate_voxel_grid
    needs_update: bool,
}

impl Buoyancy {
    fn from_voxels(voxels: Vec<Voxel>, needs_update: bool) -> Self {
        Self {
            voxels,
            needs_update,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct BuoyancyMarker;

#[derive(Bundle, Debug)]
struct ColliderBundle {
    name: Name,
    collider_shape: Collider,
    rigid_body_type: RigidBody,
    transform: TransformBundle,
}

#[derive(AssetCollection, Resource)]
struct ShipAssets {
    // #[asset(path = "models/export/ship/hull.glb#Scene0")]
    #[asset(path = "models/export/ship/carrack.glb#Scene0")]
    carrack_hull: Handle<Scene>,
}

#[derive(Debug, Clone, PartialEq)]
struct Voxel {
    position: Vec3,
    is_solid: bool,
}

fn update_voxel_solidity(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Buoyancy)>,
    mut spatial_query: SpatialQuery,
) {
    spatial_query.update_pipeline();

    for (entity, transform, mut buoyancy) in query.iter_mut() {
        if buoyancy.needs_update {
            for voxel in buoyancy.voxels.iter_mut() {
                let world_position = transform.translation + voxel.position;
                let voxel_collider = Collider::cuboid(VOXEL_SIZE, VOXEL_SIZE, VOXEL_SIZE);
                let intersects = spatial_query.shape_intersections(
                    &voxel_collider,
                    world_position,
                    Quat::IDENTITY, // Assuming no rotation for simplicity
                    SpatialQueryFilter::default(),
                );

                voxel.is_solid = !intersects.is_empty();
            }
            buoyancy.needs_update = false; // Reset update flag after processing
        }
    }
}

fn visualize_voxel_grid(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Buoyancy), Changed<Buoyancy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let voxel_visual_size = VOXEL_SIZE * 0.95; // Adjust size for visual gaps

    for (entity, transform, buoyancy) in query.iter() {
        for voxel in &buoyancy.voxels {
            if voxel.is_solid {
                // Transform for each voxel based on its position relative to the parent entity
                let voxel_position = transform.translation + voxel.position;

                // Spawn visual representation for each solid voxel
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube {
                            size: voxel_visual_size,
                        })),
                        material: materials.add(Color::rgb(0.5, 0.5, 1.0)), // Custom color
                        transform: Transform::from_translation(voxel_position),
                        ..default()
                    })
                    .insert(VoxelVisual {}); // Mark it visually if needed for tracking/deletion
            }
        }
    }
}

fn visualize_ship_bounds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &BuoyancyMarker, &Transform), Added<BuoyancyMarker>>,
    children: Query<&Children>,
    mesh_handles: Query<&Handle<Mesh>>,
) {
    for (entity, _, mesh_transform) in query.iter() {
        if let Some(mesh_handle) = find_mesh(entity, &children, &mesh_handles) {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let bounds = calculate_mesh_bounds(mesh);
                visualize_bounds(&mut commands, &mut meshes, &mut materials, bounds);
            }
        }
    }
}

fn visualize_bounds(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    bounds: (Vec3, Vec3),
) {
    let bbox_size = bounds.1 - bounds.0;
    let bbox_position = (bounds.0 + bounds.1) * 0.5;

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            bbox_size.x,
            bbox_size.y,
            bbox_size.z,
        ))),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0)),
        transform: Transform::from_translation(bbox_position),
        ..default()
    });
}

fn generate_voxel_grid(mesh: &Mesh, mesh_transform: &Transform) -> Vec<Voxel> {
    let bounds = calculate_mesh_bounds(mesh);
    let grid_size = calculate_grid_size(&bounds);
    let mut voxels = Vec::new();

    for x in 0..grid_size.x {
        for y in 0..grid_size.y {
            for z in 0..grid_size.z {
                let position = Vec3::new(
                    bounds.0.x + x as f32 * VOXEL_SIZE + VOXEL_SIZE / 2.0,
                    bounds.0.y + y as f32 * VOXEL_SIZE + VOXEL_SIZE / 2.0,
                    bounds.0.z + z as f32 * VOXEL_SIZE + VOXEL_SIZE / 2.0,
                ) + mesh_transform.translation;

                voxels.push(Voxel {
                    position,
                    is_solid: false, // Solidity will be updated based on spatial queries
                });
            }
        }
    }

    voxels
}

fn calculate_mesh_bounds(mesh: &Mesh) -> (Vec3, Vec3) {
    let positions = if let Some(VertexAttributeValues::Float32x3(pos)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        pos
    } else {
        panic!("Mesh does not contain position attribute.");
    };

    // Initialize min and max with the first vertex to ensure correctness.
    let mut min = Vec3::new(positions[0][0], positions[0][1], positions[0][2]);
    let mut max = min;

    for &vertex in positions.iter() {
        min = min.min(Vec3::from(vertex));
        max = max.max(Vec3::from(vertex));
    }
    println!("Calculated Bounds: Min: {:?}, Max: {:?}", min, max);
    (min, max)
}

fn calculate_grid_size(bounds: &(Vec3, Vec3)) -> Vec3I {
    let (min, max) = bounds;
    let size = *max - *min;

    Vec3I::new(
        (size.x / VOXEL_SIZE).ceil() as i32,
        (size.y / VOXEL_SIZE).ceil() as i32,
        (size.z / VOXEL_SIZE).ceil() as i32,
    )
}

pub fn read_buoyancy_objects(
    buoyancy_marker_query: Query<(Entity, &BuoyancyMarker, &Transform), Added<BuoyancyMarker>>,
    mut commands: Commands,
    children: Query<&Children>,
    parent_query: Query<&Parent>,
    meshes: Res<Assets<Mesh>>, // No need to mutate meshes here
    mesh_handles: Query<&Handle<Mesh>>,
) {
    for (entity, _, mesh_transform) in buoyancy_marker_query.iter() {
        println!(
            "Processing Entity: {:?}, Transform: {:?}",
            entity, mesh_transform
        );

        // Check if the entity has children (useful for checking if collider is a separate child)
        if let Ok(children) = children.get(entity) {
            println!("Children of Entity {:?}: {:?}", entity, children);
            for child in children.iter() {
                if let Ok(parent) = parent_query.get(*child) {
                    println!("Child {:?} is a child of {:?}", child, parent);
                }
            }
        }

        if let Some(mesh_handle) = find_mesh(entity, &children, &mesh_handles) {
            println!("Mesh handle found: {:?}", mesh_handle);
            if let Some(mesh) = meshes.get(mesh_handle) {
                println!("Generating voxel grid for mesh.");
                let voxels = generate_voxel_grid(mesh, mesh_transform);

                // Attach the Buoyancy component with the generated voxels
                commands
                    .entity(entity)
                    .insert(Buoyancy::from_voxels(voxels, true));

                if let Some(collider) = Collider::trimesh_from_mesh(mesh) {
                    println!("Inserting collider and dynamics components.");
                    commands.entity(entity).insert((
                        collider,
                        RigidBody::Dynamic,
                        Mass(2000.0),
                        LinearDamping(0.8),
                        AngularDamping(0.8),
                        ExternalForce::new(Vec3::ZERO).with_persistence(false),
                        Visibility::Hidden,
                    ));
                }
            } else {
                eprintln!(
                    "Failed to retrieve mesh from handle for entity marked with BuoyancyMarker"
                );
            }
        } else {
            eprintln!("Mesh not found for entity marked with BuoyancyMarker");
        }
    }
}

fn find_mesh(
    parent: Entity,
    children_query: &Query<&Children>,
    mesh_handles: &Query<&Handle<Mesh>>,
) -> Option<Handle<Mesh>> {
    if let Ok(children) = children_query.get(parent) {
        for child in children.iter() {
            if let Ok(mesh_handle) = mesh_handles.get(*child) {
                return Some(mesh_handle.clone());
            }
        }
    }
    None
}

fn get_water_height_at_position(pos: Vec3, water: &WaterParam) -> f32 {
    let water_height = water.wave_point(pos).y;
    water_height
}

fn spawn_ship(mut commands: Commands, ship_assets: Res<ShipAssets>) {
    commands.spawn((SceneBundle {
        scene: ship_assets.carrack_hull.clone(),
        ..default()
    },));
}

fn calculate_and_apply_buoyancy(
    water: WaterParam,
    mut query: Query<(&Buoyancy, &Transform, &mut ExternalForce, &ColliderDensity)>,
) {
    let gravity = 9.81; // Acceleration due to gravity in m/s^2

    for (buoyancy, transform, mut external_force, collider_density) in query.iter_mut() {
        for voxel in &buoyancy.voxels {
            let world_position = transform.translation + voxel.position;
            let water_height = get_water_height_at_position(world_position, &water);
            let submerged_volume =
                calculate_submerged_volume(world_position, water_height, VOXEL_SIZE);
            let buoyancy_force =
                Vec3::new(0.0, gravity * submerged_volume * collider_density.0, 0.0);

            // Applying the force at the voxel's position relative to the ship's center of mass
            external_force.apply_force_at_point(buoyancy_force, voxel.position, voxel.position);
        }
    }
}

fn calculate_submerged_volume(world_position: Vec3, water_height: f32, voxel_size: f32) -> f32 {
    let bottom_of_voxel = world_position.y - voxel_size / 2.0;
    let top_of_voxel = world_position.y + voxel_size / 2.0;

    if top_of_voxel <= water_height {
        voxel_size.powi(3) // Fully submerged
    } else if bottom_of_voxel >= water_height {
        0.0 // Not submerged
    } else {
        let submerged_height = water_height - bottom_of_voxel;
        submerged_height * voxel_size * voxel_size // Partially submerged volume
    }
}

// TODO: Eviction notice
fn spawn_furniture(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a bed
    commands.spawn((
        Name::new("Bed"),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(5.0, 0.15, 5.0)),
            material: materials.add(Color::MAROON),
            transform: Transform {
                translation: Vec3::new(-14.155, 7.8825, -0.147),
                rotation: Quat::from_rotation_z(-9.8367f32.to_radians()),
                scale: Vec3::ONE,
            },
            ..default()
        },
        SleepArea,
    ));
}

// TODO: Eviction notice
fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Food"),
        Food,
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.2).mesh().ico(5).unwrap()),
            material: materials.add(Color::RED),
            transform: Transform::from_xyz(13.167, 7.1885, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::sphere(0.2),
    ));
}
