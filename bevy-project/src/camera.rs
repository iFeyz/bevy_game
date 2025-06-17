use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use crate::player::Player;

#[derive(Resource, Default)]
pub struct CameraSettings {
    pub camera_mode: CameraMode,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub enum CameraMode {
    #[default]
    Free,
    Player,
}


#[derive(Default, Clone, Debug)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraSettings>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, free_camera_system)
            .add_systems(Update, camera_look)
            .add_systems(Update, camera_follow_player)
            .add_systems(Update, camera_mouse_look);
    }
}

#[derive(Component)]
pub struct FreeCamera;

pub fn free_camera_system(
    mut query : Query<&mut Transform, With<FreeCamera>>,
    keyboard_input : Res<ButtonInput<KeyCode>>,
    time : Res<Time>,
    camera_settings: Res<CameraSettings>,
) {
    if camera_settings.camera_mode != CameraMode::Free {
        return;
    }
    
    if let Ok(mut transform) = query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        let speed : f32 =  5.0;


        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += *transform.forward();
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= *transform.right();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *transform.right();
        }

        if keyboard_input.pressed(KeyCode::Space) {
            direction += *transform.up();
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            direction -= *transform.up();
        }

        transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();

    }
}

fn spawn_camera(
    mut commands : Commands,
) {
    commands.spawn((
        Camera3d::default(),
        FreeCamera,
        CameraPlayer::default(),
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}


pub fn camera_look(
    mut query : Query<&mut Transform, With<FreeCamera>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_settings: Res<CameraSettings>,

) {

    if camera_settings.camera_mode != CameraMode::Free {
        return;
    }

    if mouse_button_input.pressed(MouseButton::Right) {
        if let Ok(mut transform) = query.get_single_mut() {
            for motion in mouse_motion.read() {
                let sensitivity : f32 = 0.002;
                
                transform.rotate_y(-motion.delta.x * sensitivity);

                let right = transform.right();
                transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(*right, -motion.delta.y * sensitivity));

            }
        }
    }
}


#[derive(Component)]
pub struct CameraPlayer {
    pub player_id: i32,
    pub distance: f32,
    pub height: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for CameraPlayer {
    fn default() -> Self {
        Self {
            player_id : 0,
            distance: 10.0,
            height: 2.0,
            sensitivity: 0.01,
            yaw: 0.0,
            pitch: -0.3
        }
    } 
}


pub fn camera_follow_player(
    mut camera_query: Query<(&mut Transform, &CameraPlayer), (With<CameraPlayer>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<CameraPlayer>)>,
    time: Res<Time>,
    camera_settings: Res<CameraSettings>,
) {
    if camera_settings.camera_mode != CameraMode::Player {
        return;
    }

    if let (Ok((mut camera_transform, camera_settings)), Ok(player_transform)) = 
        (camera_query.get_single_mut(), player_query.get_single()) {
        
        let rot = Quat::from_euler(
            EulerRot::YXZ,
            camera_settings.yaw,
            camera_settings.pitch,
            0.0
        );
        
        let offset = rot * Vec3::new(0.0, 0.0, camera_settings.distance);
        let target_position = player_transform.translation + offset + Vec3::Y * camera_settings.height;
        
        let lerp_factor = 8.0 * time.delta_secs();
        camera_transform.translation = camera_transform.translation.lerp(target_position, lerp_factor);
        
        camera_transform.look_at(
            player_transform.translation + Vec3::Y * 0.5,
            Vec3::Y
        );
    }
}


pub fn camera_mouse_look(
    mut camera_query: Query<&mut CameraPlayer>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_settings: Res<CameraSettings>,
) {

    if camera_settings.camera_mode != CameraMode::Player {
        return;
    }

    if let (Ok(mut camera_player), Ok(mut player_transform)) = 
        (camera_query.get_single_mut(), player_query.get_single_mut()) 
    {
        if mouse_button_input.pressed(MouseButton::Right) {
            for motion in mouse_motion.read() {

                camera_player.yaw -= motion.delta.x * camera_player.sensitivity;
                
                player_transform.rotation = Quat::from_rotation_y(camera_player.yaw);
                
                camera_player.pitch -= motion.delta.y * camera_player.sensitivity;
                camera_player.pitch = camera_player.pitch.clamp(-1.2, 0.8);
                
                camera_player.yaw = camera_player.yaw.rem_euclid(std::f32::consts::TAU);
            }
        }
    }
}