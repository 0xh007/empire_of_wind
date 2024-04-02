use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_water::WaterParam;
use bevy_xpbd_3d::components::ExternalForce;
use bevy_xpbd_3d::prelude::*;

use crate::prelude::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppStates::Next), spawn_cube)
            .add_systems(Update, calculate_and_apply_buoyancy)
            .configure_loading_state(
                LoadingStateConfig::new(AppStates::AssetLoading).load_collection::<ShipAssets>(),
            )
            .add_systems(OnEnter(AppStates::Next), spawn_ship);
        // .add_systems(OnEnter(AppStates::Next), spawn_furniture)
        // .add_systems(OnEnter(AppStates::Next), spawn_food);
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
    #[asset(path = "models/export/ship/carrack_b.glb#Scene0")]
    carrack_hull: Handle<Scene>,
}

#[derive(Component)]
struct Buoyancy {
    voxels: Vec<Voxel>,
    cube_size: f32,
    voxel_size: f32,
}

impl Buoyancy {
    fn new(cube_size: f32, voxels_per_axis: usize) -> Self {
        let voxel_size = cube_size / voxels_per_axis as f32;
        let voxels = subdivide_cube_into_voxels(cube_size, voxels_per_axis, voxel_size);
        Self {
            voxels,
            cube_size,
            voxel_size,
        }
    }
}

impl Buoyancy {
    fn new_from_mesh(mesh: &Mesh, voxels_per_axis: usize) -> Self {
        // Pseudo-code to generate voxels based on mesh bounds and internal volume
        let voxels = generate_voxels_from_mesh(mesh, voxels_per_axis);
        Self { voxels }
    }
}

#[derive(Component)]
struct VoxelizedMesh {
    voxel_positions: Vec<Vec3>, // Positions of voxels in world space
    voxel_size: f32,            // Uniform size of each voxel (assuming cubic voxels)
}

struct Voxel {
    position: Vec3,
    is_solid: bool,
}

fn generate_voxel_grid(commands: &mut Commands, mesh: &Mesh, voxel_size: f32) {
    let bounds = calculate_mesh_bounds(mesh);
    let grid_size = calculate_grid_size(&bounds, voxel_size);

    let mut voxel_positions = Vec::new();

    // Generate grid positions
    for x in 0..grid_size.x {
        for y in 0..grid_size.y {
            for z in 0..grid_size.z {
                let position = Vec3::new(
                    x as f32 * voxel_size + bounds.min.x,
                    y as f32 * voxel_size + bounds.min.y,
                    z as f32 * voxel_size + bounds.min.z,
                );
                voxel_positions.push(position);

                commands.spawn().insert(Voxel {
                    position: Vec3I::new(x, y, z),
                    is_solid: false, // Initial state, to be updated based on mesh intersection
                });
            }
        }
    }

    // Create an entity with the VoxelizedMesh component
    commands.spawn().insert(VoxelizedMesh {
        voxel_positions,
        voxel_size,
    });
}

// Pseudo-function to demonstrate concept
fn generate_voxels_from_mesh(mesh: &Mesh, voxels_per_axis: usize) -> Vec<Voxel> {
    let mut voxels = Vec::new();
    // voxel generation logic, based on the mesh geometry
    voxels
}

pub fn read_buoyancy_objects(
    buoyancy_marker_query: Query<(Entity, &BuoyancyMarker), Added<BuoyancyMarker>>,
    mut commands: Commands,
    children: Query<&Children>,
    meshes: Res<Assets<Mesh>>,
    mesh_handles: Query<&Handle<Mesh>>,
) {
    for (entity, _) in buoyancy_marker_query.iter() {
        if let Some(mesh) = find_mesh(entity, &children, &meshes, &mesh_handles) {
            // Here you would call a function to convert the mesh to a voxel representation
            // for buoyancy calculations, rather than creating a collider.
            let buoyancy_voxels = generate_voxels_from_mesh(mesh);

            // Insert buoyancy component with the generated voxels
            commands.entity(entity).insert(Buoyancy {
                voxels: buoyancy_voxels,
                // You might need to adjust the struct to fit this new approach
            });
        } else {
            log::error!("Failed to find mesh for buoyancy object");
        }
    }
}

fn find_mesh<'a>(
    parent: Entity,
    children_query: &'a Query<&Children>,
    meshes: &'a Assets<Mesh>,
    mesh_handles: &'a Query<&Handle<Mesh>>,
) -> Option<&'a Mesh> {
    if let Ok(children) = children_query.get(parent) {
        for child in children.iter() {
            if let Ok(mesh_handle) = mesh_handles.get(*child) {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    return Some(mesh);
                }
            }
        }
    }
    None
}

fn calculate_and_apply_buoyancy(
    water: WaterParam,
    mut query: Query<(
        &Buoyancy,
        &Transform,
        &mut ExternalForce,
        &ColliderDensity,
        &Collider,
    )>,
) {
    for (buoyancy, transform, mut external_force, collider_density, collider) in query.iter_mut() {
        let mut total_buoyancy_force = Vec3::ZERO;
        let gravity = 9.81;
        let cube_volume = buoyancy.cube_size.powi(3);
        let cube_weight = cube_volume * collider_density.0 * gravity;

        for voxel in &buoyancy.voxels {
            let world_position = transform.translation + voxel.position;
            let water_height = get_water_height_at_position(world_position, &water);
            let submerged_volume =
                calculate_submerged_volume(world_position, water_height, buoyancy.voxel_size);
            let buoyancy_force = Vec3::new(0.0, gravity * submerged_volume, 0.0);

            total_buoyancy_force += buoyancy_force;
        }

        // Limit the buoyancy force to not exceed the cube's weight
        if total_buoyancy_force.y > cube_weight {
            total_buoyancy_force.y = cube_weight;
        }

        external_force.apply_force(total_buoyancy_force);
    }
}

fn calculate_submerged_volume(world_position: Vec3, water_height: f32, voxel_size: f32) -> f32 {
    let bottom_of_voxel = world_position.y - voxel_size / 2.0;
    let top_of_voxel = world_position.y + voxel_size / 2.0;

    // If the top of the voxel is below the water, it's fully submerged
    if top_of_voxel <= water_height {
        return voxel_size.powi(3); // The volume of the voxel
    }
    // If the bottom of the voxel is above the water, it's not submerged
    else if bottom_of_voxel >= water_height {
        return 0.0;
    }
    // Otherwise, it's partially submerged
    else {
        let submerged_height = water_height - bottom_of_voxel;
        return submerged_height * voxel_size * voxel_size; // The submerged volume
    }
}

fn get_water_height_at_position(pos: Vec3, water: &WaterParam) -> f32 {
    let water_height = water.wave_point(pos).y;
    water_height
}

fn subdivide_cube_into_voxels(
    cube_size: f32,
    voxels_per_axis: usize,
    voxel_size: f32,
) -> Vec<Voxel> {
    let mut voxels = Vec::new();

    for x in 0..voxels_per_axis {
        for y in 0..voxels_per_axis {
            for z in 0..voxels_per_axis {
                let position = Vec3::new(
                    (x as f32 + 0.5) * voxel_size - cube_size / 2.0,
                    (y as f32 + 0.5) * voxel_size - cube_size / 2.0,
                    (z as f32 + 0.5) * voxel_size - cube_size / 2.0,
                );
                voxels.push(Voxel {
                    position,
                    is_receiver: true,
                });
            }
        }
    }
    voxels
}

fn spawn_cube(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube_size = 2.0;
    let voxels_per_axis = 5;

    let cube_mesh = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));
    let buoyancy_component = Buoyancy::new(cube_size, voxels_per_axis);
    let cube_density = 0.8;

    commands.spawn((
        PbrBundle {
            mesh: cube_mesh,
            material: materials.add(Color::rgb(0.2, 0.7, 0.9)),
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        LinearDamping(1.8),
        AngularDamping(1.8),
        ExternalForce::new(Vec3::ZERO).with_persistence(false),
        Collider::cuboid(cube_size / 2.0, cube_size / 2.0, cube_size / 2.0),
        ColliderDensity(cube_density),
        buoyancy_component,
    ));
}

fn spawn_ship(mut commands: Commands, ship_assets: Res<ShipAssets>) {
    commands.spawn((
        SceneBundle {
            scene: ship_assets.carrack_hull.clone(),
            ..default()
        },
        WaterInteractable::new(-0.4, -8.0, 9.0, -2.0, 2.0),
    ));
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
