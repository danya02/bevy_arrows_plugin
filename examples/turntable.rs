//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::time::Duration;

use bevy::prelude::*;
use bevy_arrows_plugin::{
    BevyArrowsPlugin,
    vec_arrow::{TargetCoordinateSpace, VecArrow},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::{Animator, Tween, TweeningPlugin, lens::TransformRotationLens};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // helpers
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(TweeningPlugin)
        // our plugin
        .add_plugins(BevyArrowsPlugin)
        // systems
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_instructions)
        .add_systems(Update, turntable_system)
        .add_systems(Update, on_space_press_roll)
        .add_systems(Update, update_instructions)
        .run();
}

#[derive(Component)]
struct TurntableMarker;

#[derive(Component)]
struct CubeMarker;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(0.0, -1.0, 0.0)),
        Name::new("Base"),
    ));
    // cube
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Name::new("Cube"),
            CubeMarker,
        ))
        .with_child((
            Name::new("X arrow"),
            VecArrow::new(Vec3::new(2.0, 0.0, 0.0), TargetCoordinateSpace::Local)
                .with_color(Color::linear_rgb(1.0, 0.0, 0.0)),
        ))
        .with_child((
            Name::new("Y arrow"),
            VecArrow::new(Vec3::new(0.0, 2.0, 0.0), TargetCoordinateSpace::Local)
                .with_color(Color::linear_rgb(0.0, 1.0, 0.0)),
        ))
        .with_child((
            Name::new("Z arrow"),
            VecArrow::new(Vec3::new(0.0, 0.0, 2.0), TargetCoordinateSpace::Local)
                .with_color(Color::linear_rgb(0.0, 0.0, 1.0)),
        ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        Name::new("Light"),
    ));

    // empty object at the center of the world
    let parent = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            TurntableMarker,
            Name::new("Turntable root"),
            InheritedVisibility::HIDDEN,
        ))
        .id();

    // camera
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("Camera"),
        ))
        .set_parent(parent);
}

fn turntable_system(mut query: Query<(&mut Transform, &TurntableMarker)>) {
    for (mut transform, _) in query.iter_mut() {
        transform.rotate_y(0.01);
    }
}

fn on_space_press_roll(
    mut commands: Commands,
    keypresses: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform), With<CubeMarker>>,
) {
    if keypresses.just_pressed(KeyCode::Space) {
        // Get the current rotation of the cube
        let (entity, transform) = query.single();
        // Create a tween that rotates the cube to a random angle

        let mut rng = rand::rng();

        let u = rng.random_range(0.0..1.0);
        let v = rng.random_range(0.0..1.0);
        let w = rng.random_range(0.0..1.0);
        let sqrt_u = f32::sqrt(u);
        let sqrt_neg_u = f32::sqrt(1.0 - u);

        let dest = Quat::from_xyzw(
            sqrt_neg_u * f32::sin(std::f32::consts::TAU * v),
            sqrt_neg_u * f32::cos(std::f32::consts::TAU * v),
            sqrt_u * f32::sin(std::f32::consts::TAU * w),
            sqrt_u * f32::cos(std::f32::consts::TAU * w),
        );

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(0.2),
            TransformRotationLens {
                start: transform.rotation,
                end: dest,
            },
        );

        commands.entity(entity).insert(Animator::new(tween));
    }
}

fn setup_instructions(mut commands: Commands) {
    commands.spawn((Text::default(), Node {
        position_type: PositionType::Absolute,
        top: Val::Px(12.0),
        left: Val::Px(12.0),
        ..default()
    }));
}

fn update_instructions(mut text: Single<&mut Text>) {
    text.0 = format!("Press space to roll the cube");
}
