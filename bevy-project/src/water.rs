use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    pbr::{MaterialPlugin, Material},
};

#[derive(Component)]
pub struct Water;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub time: f32,
}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            time: 0.0,
        }
    }
}

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default())
           .add_systems(Update, update_water_time);
    }
}

fn update_water_time(
    time: Res<Time>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    let current_time = time.elapsed_secs();
    
    for (_handle, material) in water_materials.iter_mut() {
        material.time = current_time;
    }
} 