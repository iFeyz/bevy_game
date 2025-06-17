# Création d'un Jeu avec Bevy Engine
## Tutoriel 01: Configuration du projet, joueur et système de caméra

Ce tutoriel vous guidera à travers les étapes de création d'un jeu 3D basique avec le moteur Bevy en Rust, en implémentant un système de caméra, un terrain et un personnage jouable.

## Table des matières
1. [Configuration du projet](#configuration-du-projet)
2. [Création du client de base](#création-du-client-de-base)
3. [Ajout du terrain et des éléments de base](#ajout-du-terrain-et-des-éléments-de-base)
4. [Création du joueur](#création-du-joueur)
5. [Implémentation de la caméra libre](#implémentation-de-la-caméra-libre)
6. [Caméra suivant le joueur](#caméra-suivant-le-joueur)
7. [Sélection entre les modes de caméra](#sélection-entre-les-modes-de-caméra)
8. [Points techniques sur Bevy](#points-techniques-sur-bevy)

## Configuration du projet

Commençons par créer un nouveau projet Rust :

```bash
cargo new bevy-project
```

Ouvrez le fichier `src/main.rs` et remplacez son contenu par un simple "Hello World" :

```rust
fn main() {
    println!("Hello, world!");
}
```

Modifiez le fichier `Cargo.toml` pour ajouter les dépendances de Bevy :

```toml
[package]
name = "bevy-project"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { version = "0.15", features = [
    "bevy_core_pipeline",
    "bevy_render",
    "bevy_asset",
    "bevy_pbr",
    "x11",
    "serialize",
    "bevy_window",
    "png",
]}
bevy_egui = "0.33.0"
```

## Création du client de base

Modifions `src/main.rs` pour créer un lanceur qui nous permettra de faciliter la création de tests par la suite :

```rust
mod client;
use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    match args.next().as_ref().map(|s| s.as_str()) {
        Some("client") => {
            println!("Running on client mode");
            client::run();
        }
        _ => {
            println!("Usage : {} [client]", args.next().unwrap());
        }
    }
}
```

Créons ensuite notre fichier client avec une interface basique en utilisant egui :

```rust
// src/client.rs
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiContextPass};

pub fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin);
    app.add_systems(Update, ui_example_system);
    app.run();
}

fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}
```

Vous devriez voir une fenêtre comme celle-ci :

![Fenêtre avec interface minimale](/image/img00.png)

## Ajout du terrain et des éléments de base

Modifions notre fichier `src/client.rs` pour ajouter un terrain, une lumière et une caméra :

```rust
// src/client.rs
pub fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Update, ui_example_system);
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

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

Le résultat devrait ressembler à ceci :

![Terrain de base](/image/img01.png)

## Création du joueur

Créons un module séparé pour notre joueur dans `src/player.rs` :

```rust
// src/player.rs
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
```

Maintenant, modifions notre fichier client pour ajouter notre plugin joueur :

```rust
// src/client.rs
mod player;
use player::PlayerPlugin;

pub fn run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin);
    app.add_plugins(PlayerPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Update, ui_example_system);
    app.run();
}
```

![Joueur dans la scène](/image/img03.png)

## Implémentation de la caméra libre

Créons un nouveau fichier pour gérer notre système de caméra dans `src/camera.rs` :

```rust
// src/camera.rs
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

#[derive(Default, Clone, Debug)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, free_camera_system)
            .add_systems(Update, camera_look);
    }
}

#[derive(Component)]
pub struct FreeCamera;

pub fn free_camera_system(
    mut query : Query<&mut Transform, With<FreeCamera>>,
    keyboard_input : Res<ButtonInput<KeyCode>>,
    time : Res<Time>,
) {
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
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn camera_look(
    mut query : Query<&mut Transform, With<FreeCamera>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
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
```

N'oubliez pas de retirer l'ancienne caméra du système de configuration et d'ajouter le plugin de caméra dans le client.

## Caméra suivant le joueur

Ajoutons un deuxième mode de caméra qui suivra le joueur dans `src/camera.rs` :

```rust
// Dans impl Plugin for CameraPlugin
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, free_camera_system)
            .add_systems(Update, camera_look)
            .add_systems(Update, camera_follow_player)
            .add_systems(Update, camera_mouse_look);
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
) {
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
) {
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
```

## Sélection entre les modes de caméra

Ajoutons une ressource pour pouvoir changer entre les différents modes de caméra :

```rust
// Ajouter dans src/camera.rs
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

// Modifier la fonction spawn_camera pour inclure tous les composants nécessaires
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

// Mettre à jour le Plugin pour initialiser la ressource
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
```

Maintenant, ajoutez le paramètre `camera_settings: Res<CameraSettings>` dans chacune des fonctions de caméra et conditionnez leur exécution avec :

```rust
// Pour la caméra libre
if camera_settings.camera_mode != CameraMode::Free {
    return;
}

// Pour la caméra joueur
if camera_settings.camera_mode != CameraMode::Player {
    return;
}
```

Vous pouvez maintenant choisir entre la caméra libre et la caméra qui suit le joueur !

![Interface avec sélection de caméra](/image/img04.png)

## Points techniques sur Bevy

### Architecture ECS (Entity-Component-System)

Bevy est construit autour d'une architecture ECS, qui est un paradigme de conception où :

- **Entities** : Simples identifiants uniques qui représentent les objets du jeu
- **Components** : Données pures attachées aux entités (ex: `Transform`, `Player`, `FreeCamera`)
- **Systems** : Fonctions qui traitent ces données (ex: `free_camera_system`, `camera_follow_player`)

Cette architecture offre plusieurs avantages :
- **Performance** : Traitement par lots des données de même type
- **Modularité** : Composants et systèmes facilement réutilisables
- **Flexibilité** : Composition dynamique d'entités à partir de composants

### Système de Plugin

```rust
#[derive(Default, Clone, Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
    }
}
```

Les plugins dans Bevy sont des modules réutilisables qui encapsulent une fonctionnalité. Le trait `Plugin` avec sa méthode `build` vous permet d'ajouter des systèmes, des ressources et des événements à votre application de manière organisée.

### Cycle de vie et ordonnancement des systèmes

Bevy utilise différentes étapes d'exécution pour ses systèmes :
- `Startup` : Systèmes exécutés une seule fois au démarrage (`spawn_player`, `setup`)
- `Update` : Systèmes exécutés à chaque frame (`free_camera_system`, `ui_example_system`)

L'ordre d'exécution est déterminé par les dépendances entre systèmes ou peut être explicitement configuré.

### Composants et Query

Les composants sont de simples structures de données associées aux entités :

```rust
#[derive(Component)]
pub struct Player {
    pub id: i32,
}
```

Les systèmes utilisent des "Query" pour accéder aux composants :

```rust
pub fn free_camera_system(
    mut query: Query<&mut Transform, With<FreeCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
)
```

Cette query récupère toutes les entités qui ont à la fois un composant `Transform` (mutable) et un composant `FreeCamera`.

### Filtres de requête complexes

Les systèmes peuvent utiliser des filtres complexes dans leurs requêtes :

```rust
mut camera_query: Query<(&mut Transform, &CameraPlayer), (With<CameraPlayer>, Without<Player>)>,
player_query: Query<&Transform, (With<Player>, Without<CameraPlayer>)>,
```

- `With<T>` : L'entité doit avoir le composant T
- `Without<T>` : L'entité ne doit pas avoir le composant T

Ceci permet d'éviter les conflits lorsqu'on manipule plusieurs entités avec des composants similaires.

### Ressources (Resources)

Les ressources sont des données globales partagées entre les systèmes :

```rust
#[derive(Resource, Default)]
pub struct CameraSettings {
    pub camera_mode: CameraMode,
}
```

Elles sont accessibles dans les systèmes via :
- `Res<T>` pour un accès en lecture seule
- `ResMut<T>` pour un accès en lecture-écriture

### Gestion des transformations 3D

Le système de caméra utilise des opérations vectorielles et des quaternions pour les rotations :

```rust
let rot = Quat::from_euler(
    EulerRot::YXZ,
    camera_settings.yaw,
    camera_settings.pitch,
    0.0
);

let offset = rot * Vec3::new(0.0, 0.0, camera_settings.distance);
```

Les quaternions (`Quat`) sont utilisés pour représenter des rotations 3D sans problèmes de gimbal lock que l'on rencontre avec les angles d'Euler.

### Interpolation linéaire (Lerp)

Pour des mouvements fluides, Bevy utilise l'interpolation linéaire :

```rust
let lerp_factor = 8.0 * time.delta_secs();
camera_transform.translation = camera_transform.translation.lerp(target_position, lerp_factor);
```

Cette technique permet de créer des transitions douces entre deux positions, rendant les mouvements de caméra plus naturels.

### Gestion des entrées et événements

Bevy fournit des systèmes pour gérer les entrées utilisateur :

```rust
keyboard_input: Res<ButtonInput<KeyCode>>,
mouse_button_input: Res<ButtonInput<MouseButton>>,
mut mouse_motion: EventReader<MouseMotion>,
```

- `ButtonInput` : Pour l'état des boutons (souris/clavier)
- `EventReader` : Pour les flux d'événements (mouvements de souris)

### Spawning d'entités

La création d'entités se fait avec la méthode `spawn` :

```rust
commands.spawn((
    Camera3d::default(),
    FreeCamera,
    CameraPlayer::default(),
    Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

Cette méthode crée une nouvelle entité avec tous les composants fournis.

### Gestion des assets (maillages et matériaux)

Les ressources graphiques sont gérées par des gestionnaires d'assets spécialisés :

```rust
Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
MeshMaterial3d(materials.add(StandardMaterial {
    base_color: Color::srgb(0.3, 0.6, 0.9),
    metallic: 0.1,
    perceptual_roughness: 0.8,
    ..default()
})),
```

`Assets<T>` est un gestionnaire de ressources qui attribue des handles uniques à chaque asset chargé ou créé.

### Intégration avec egui

Bevy s'intègre facilement avec la bibliothèque d'interface utilisateur egui :

```rust
fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}
```

Cette intégration permet de créer rapidement des interfaces utilisateur pour votre jeu.

---

Et voilà ! Vous avez maintenant une compréhension plus approfondie des concepts techniques de Bevy utilisés dans ce tutoriel. Dans le prochain tutoriel, nous ajouterons des contrôles de joueur, des collisions et d'autres fonctionnalités pour rendre notre jeu plus interactif.