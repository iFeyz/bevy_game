use bevy::prelude::*;

#[derive(Default, Clone, Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_player);
    }
}

#[derive(Component)]
pub struct Player {
    pub id : i32,
}

fn spawn_player(
    mut commands : Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.9),
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Player { id: 01 }
    ));
}


