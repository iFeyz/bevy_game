use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::player::PlayerPlugin;
use crate::camera::{CameraPlugin, CameraSettings, CameraMode};

pub fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin);
    app.add_plugins(PlayerPlugin);
    app.add_plugins(CameraPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Update, camera_ui_system);
    app.run();
}

#[derive(Component)]
struct Ground;

fn setup(
    mut commands : Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Ground,
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));


    
}

fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
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
