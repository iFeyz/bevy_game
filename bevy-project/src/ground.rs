use bevy::{pbr::wireframe::Wireframe, prelude::*};

#[derive(Component)]
pub struct Ground;


pub fn toggle_wireframe(
    mut commands : Commands,
    landscapes_wireframe : Query<Entity, (With<Ground>,With<Wireframe>)>,
    landscapes_solid : Query<Entity, (With<Ground>,Without<Wireframe>)>,
    input : Res<ButtonInput<KeyCode>>
) {
    if input.just_pressed(KeyCode::KeyK) {
        for ground in &landscapes_solid {
            println!("Adding wireframe to ground");
            commands.entity(ground).insert(Wireframe);
        }
        for ground in &landscapes_wireframe {
            println!("Removing wireframe from ground");
            commands.entity(ground).remove::<Wireframe>();
        }
    }
}