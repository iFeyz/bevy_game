use bevy::prelude::*;
use bevy::pbr::wireframe::WireframePlugin;
use bevy_atmosphere::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::player::PlayerPlugin;
use crate::camera::{CameraPlugin, CameraSettings, CameraMode};
use crate::ground::{Ground, toggle_wireframe};
use crate::water::{WaterPlugin, WaterMaterial, Water};
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};
use std::collections::HashMap;

// Chunk system for infinite terrain
#[derive(Resource, Default)]
pub struct WorldPosition {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub loaded_chunks: HashMap<(i32, i32), (Entity, Option<Entity>)>, // (terrain_entity, optional_water_entity)
    pub chunk_size: f32,
    pub render_distance: i32,
}

#[derive(Component)]
pub struct TerrainChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

const CHUNK_SIZE: f32 = 50.0;
const RENDER_DISTANCE: i32 = 3; // 3 chunks dans chaque direction
const WATER_LEVEL: f32 = 1.0; // Niveau de l'eau (remonté pour une meilleure visibilité)

// Linear interpolation between two colors
fn lerp_color(color1: [f32; 4], color2: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        color1[0] + (color2[0] - color1[0]) * t,
        color1[1] + (color2[1] - color1[1]) * t,
        color1[2] + (color2[2] - color1[2]) * t,
        color1[3] + (color2[3] - color1[3]) * t,
    ]
}

// Get smooth terrain color based on height (without water)
fn get_terrain_color(height: f32) -> [f32; 4] {
    // Define color stops (no water colors since water is separate)
    let sand_color = [0.8, 0.7, 0.4, 1.0];     // Sandy color for beach
    let grass_color = [0.3, 0.6, 0.2, 1.0];    // Green for grass
    let rock_color = [0.5, 0.4, 0.3, 1.0];     // Brown for rocks
    let snow_color = [0.9, 0.9, 0.9, 1.0];     // White for snow
    
    // Define height thresholds
    let sand_level = 0.3;
    let grass_level = 1.5;
    let rock_level = 3.0;
    let snow_level = 4.0;
    
    if height < sand_level {
        sand_color
    } else if height < grass_level {
        let t = (height - sand_level) / (grass_level - sand_level);
        lerp_color(sand_color, grass_color, t)
    } else if height < rock_level {
        let t = (height - grass_level) / (rock_level - grass_level);
        lerp_color(grass_color, rock_color, t)
    } else if height < snow_level {
        let t = (height - rock_level) / (snow_level - rock_level);
        lerp_color(rock_color, snow_color, t)
    } else {
        snow_color
    }
}

pub fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin);
    app.add_plugins(PlayerPlugin);
    app.add_plugins(WireframePlugin);
    app.add_plugins(WaterPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(AtmospherePlugin);
    
    // Initialize chunk system resources
    app.insert_resource(WorldPosition::default());
    app.insert_resource(ChunkManager {
        loaded_chunks: HashMap::new(),
        chunk_size: CHUNK_SIZE,
        render_distance: RENDER_DISTANCE,
    });
    
    app.add_systems(Startup, setup);
    app.add_systems(Update, (
        update_world_position,
        manage_chunks,
        camera_ui_system,
        toggle_wireframe,
    ));
    app.run();
}

// Update world position based on player/camera position
fn update_world_position(
    mut world_pos: ResMut<WorldPosition>,
    camera_query: Query<&Transform, (With<Camera>, Without<TerrainChunk>)>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let new_chunk_x = (camera_transform.translation.x / CHUNK_SIZE).floor() as i32;
        let new_chunk_z = (camera_transform.translation.z / CHUNK_SIZE).floor() as i32;
        
        if world_pos.chunk_x != new_chunk_x || world_pos.chunk_z != new_chunk_z {
            world_pos.chunk_x = new_chunk_x;
            world_pos.chunk_z = new_chunk_z;
            info!("Player moved to chunk ({}, {})", new_chunk_x, new_chunk_z);
        }
    }
}

// Manage chunk loading and unloading
fn manage_chunks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    world_pos: Res<WorldPosition>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    terrain_chunks: Query<Entity, With<TerrainChunk>>,
    water_chunks: Query<Entity, With<Water>>,
) {
    if !world_pos.is_changed() {
        return;
    }
    
    let player_chunk_x = world_pos.chunk_x;
    let player_chunk_z = world_pos.chunk_z;
    let render_distance = chunk_manager.render_distance;
    
    // Collect chunks that should be loaded
    let mut required_chunks = std::collections::HashSet::new();
    for x in (player_chunk_x - render_distance)..=(player_chunk_x + render_distance) {
        for z in (player_chunk_z - render_distance)..=(player_chunk_z + render_distance) {
            required_chunks.insert((x, z));
        }
    }
    
    // Remove chunks that are too far (both terrain and water)
    let mut chunks_to_remove = Vec::new();
    for (chunk_pos, (terrain_entity, water_entity_opt)) in chunk_manager.loaded_chunks.iter() {
        if !required_chunks.contains(chunk_pos) {
            chunks_to_remove.push(*chunk_pos);
            // Supprimer le terrain
            commands.entity(*terrain_entity).despawn_recursive();
            // Supprimer l'eau si elle existe
            if let Some(water_entity) = water_entity_opt {
                commands.entity(*water_entity).despawn_recursive();
            }
            info!("Removed chunk at ({}, {}) - terrain and water", chunk_pos.0, chunk_pos.1);
        }
    }
    for chunk_pos in chunks_to_remove {
        chunk_manager.loaded_chunks.remove(&chunk_pos);
    }
    
    // Add new chunks that need to be loaded
    for chunk_pos in required_chunks {
        if !chunk_manager.loaded_chunks.contains_key(&chunk_pos) {
            let (terrain_entity, water_entity_opt) = spawn_chunk(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut water_materials,
                chunk_pos.0,
                chunk_pos.1,
            );
            chunk_manager.loaded_chunks.insert(chunk_pos, (terrain_entity, water_entity_opt));
            info!("Created chunk at ({}, {}) - terrain and water", chunk_pos.0, chunk_pos.1);
        }
    }
}

// Generate water mesh for areas below water level
fn generate_water_mesh(
    world_offset_x: f32,
    world_offset_z: f32,
    subdivisions: u32,
) -> Option<Mesh> {
    info!("Generating water mesh for offset ({}, {})", world_offset_x, world_offset_z);
    
    // Check if this chunk needs water by sampling terrain heights
    let main_noise = BasicMulti::<Perlin>::new(1)
        .set_octaves(8)           
        .set_frequency(0.05)
        .set_persistence(0.6)     
        .set_lacunarity(2.0);
        
    let detail_noise = BasicMulti::<Perlin>::new(2)
        .set_octaves(3)
        .set_frequency(0.03)
        .set_persistence(0.4)
        .set_lacunarity(2.0);

    let mut has_water = false;
    let step = CHUNK_SIZE / subdivisions as f32;
    let half_size = CHUNK_SIZE / 2.0;
    
    // Sample multiple points in the chunk to see if any are below water level
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let local_x = (x as f32 * step) - half_size;
            let local_z = (z as f32 * step) - half_size;
            
            let world_x = local_x + world_offset_x;
            let world_z = local_z + world_offset_z;
            
            // Calculate terrain height at this point
            let main_val = main_noise.get([world_x as f64, world_z as f64, 42.0]) * 22.0;
            let detail_val = detail_noise.get([world_x as f64, world_z as f64, 100.0]) * 3.0;
            let terrain_height = (main_val + detail_val) as f32;
            
            // If any point is below water level, we need water for this chunk
            if terrain_height < WATER_LEVEL {
                has_water = true;
                break;
            }
        }
        if has_water {
            break;
        }
    }
    
    if !has_water {
        return None; // No water needed for this chunk
    }
    
    // Create a simple water plane for this chunk
    let mesh = Mesh::from(
        Plane3d::default()
            .mesh()
            .size(CHUNK_SIZE, CHUNK_SIZE)
            .subdivisions(subdivisions)
    );
    
    Some(mesh)
}

// Spawn a single terrain chunk at the given coordinates
fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    water_materials: &mut ResMut<Assets<WaterMaterial>>,
    chunk_x: i32,
    chunk_z: i32,
) -> (Entity, Option<Entity>) { // Retourne (terrain_entity, optional_water_entity)
    // Create terrain mesh
    let mut terrain = Mesh::from(
        Plane3d::default()
            .mesh()
            .size(CHUNK_SIZE, CHUNK_SIZE)
            .subdivisions(50)  // Good balance between detail and performance
    );
    
    let terrain_material = StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    };
    
    // Calculate world offset for this chunk
    let world_offset_x = chunk_x as f32 * CHUNK_SIZE;
    let world_offset_z = chunk_z as f32 * CHUNK_SIZE;
    
    // Deform the terrain
    if let Some(VertexAttributeValues::Float32x3(positions)) = terrain.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        let main_noise = BasicMulti::<Perlin>::new(1)
            .set_octaves(8)           
            .set_frequency(0.05)
            .set_persistence(0.6)     
            .set_lacunarity(2.0);
            
        let detail_noise = BasicMulti::<Perlin>::new(2)
            .set_octaves(3)
            .set_frequency(0.03)
            .set_persistence(0.4)
            .set_lacunarity(2.0);
        
        let mut colors = Vec::new();
        
        for pos in positions.iter_mut() {
            // Apply world offset to get correct world coordinates
            let world_x = pos[0] + world_offset_x;
            let world_z = pos[2] + world_offset_z;
            
            // Generate height using world coordinates for seamless chunks
            let main_val = main_noise.get([world_x as f64, world_z as f64, 42.0]) * 22.0;
            let detail_val = detail_noise.get([world_x as f64, world_z as f64, 100.0]) * 3.0;
            
            let height = main_val + detail_val;
            pos[1] = height as f32;
            
            // Get color based on height
            let color = get_terrain_color(pos[1]);
            colors.push(color);
        }
        
        terrain.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        terrain.compute_normals();
    }
    
    // Spawn terrain chunk
    let terrain_entity = commands.spawn((
        Mesh3d(meshes.add(terrain)),
        MeshMaterial3d(materials.add(terrain_material)),
        Transform::from_translation(Vec3::new(world_offset_x, 0.0, world_offset_z)),
        TerrainChunk { chunk_x, chunk_z },
        Ground,
    )).id();
    
    // Generate water mesh only for areas below water level
    let water_entity = if let Some(water_mesh) = generate_water_mesh(world_offset_x, world_offset_z, 20) {
        info!("Creating water for chunk ({}, {})", chunk_x, chunk_z);
        
        Some(commands.spawn((
            Mesh3d(meshes.add(water_mesh)),
            MeshMaterial3d(water_materials.add(WaterMaterial::default())),
            Transform::from_translation(Vec3::new(world_offset_x, WATER_LEVEL, world_offset_z)),
            Water,
            TerrainChunk { chunk_x, chunk_z },
        )).id())
    } else {
        info!("No water needed for chunk ({}, {})", chunk_x, chunk_z);
        None
    };
    
    (terrain_entity, water_entity)
}

fn setup(mut commands: Commands) {
    // Only spawn lighting, chunks will be managed by the chunk system
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

}

fn camera_ui_system(
    mut contexts: EguiContexts,
    mut camera_settings: ResMut<CameraSettings>,
) {
    egui::Window::new("Camera")
        .default_size([200.0, 200.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Camera Mode");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.radio_value(&mut camera_settings.camera_mode, CameraMode::Free, "Free Camera").clicked() {
                    info!("Free Camera mode");
                }
                if ui.radio_value(&mut camera_settings.camera_mode, CameraMode::Player, "Player Camera").clicked() {
                    info!("Player Camera mode");
                }
            });
        });
}
