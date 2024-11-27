use bevy_asset::Assets;
use bevy_core_pipeline::core_3d::Camera3dBundle;
use bevy_ecs::system::{Commands, ResMut};
use bevy_hierarchy::BuildChildren;
use bevy_math::{primitives::{Cuboid, Sphere, Plane3d}, Vec2, Vec3};
use bevy_pbr::{DirectionalLight, DirectionalLightBundle, PbrBundle, PointLightBundle, StandardMaterial};
use bevy_rapier3d::{geometry::{Collider, CollisionGroups, Group}, prelude::{ColliderMassProperties, Restitution, RigidBody}};
use bevy_render::{color::Color, mesh::{Mesh, Meshable}, prelude::SpatialBundle, view::VisibilityBundle};
use bevy_transform::components::Transform;

use crate::{camera::PanOrbitCamera, drag::DraggableBundle};

const WORLD_SIZE: Vec2 = Vec2::new(2.5, 2.5);
const STATIC_GROUP: Group = Group::GROUP_1;
const OBJECT_GROUP: Group = Group::GROUP_2;
const WALL_HEIGHT: f32 = 0.075;
const WALL_WIDTH: f32 = 0.075;
const WALL_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let translation = Vec3::new(1.5, 2.0, 2.5);
    let focus = Vec3::ZERO;
    let transform = Transform::from_translation(translation)
        .looking_at(focus, Vec3::Y);

    commands
        .spawn(Camera3dBundle {
            transform,
            ..Default::default()
        })
        .insert(PanOrbitCamera {
            focus,
            radius: translation.length(),
            ..Default::default()
        })
        .insert(VisibilityBundle::default())
        .with_children(|commands| {
            commands.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    shadows_enabled: false,
                    illuminance: 1000.0,
                    ..Default::default()
                },
                transform: Transform::from_xyz(-2.5, 2.5, 2.5)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
            });
        });

    //lights (note ambient light use in the app resources)
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 0.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(-5.0, 5.0, 0.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 5.0, 5.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 5.0, -5.0),
        ..Default::default()
    });
    
    // north wall
    commands
        .spawn(Collider::cuboid((WORLD_SIZE.x - WALL_WIDTH) * 0.5, WALL_HEIGHT * 0.5, WALL_WIDTH * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, OBJECT_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(WORLD_SIZE.x - WALL_WIDTH, WALL_HEIGHT, WALL_WIDTH))),
            material: materials.add(WALL_COLOR),
            transform: Transform::from_xyz(-WALL_WIDTH * 0.5, WALL_HEIGHT * 0.5, (-WORLD_SIZE.y + WALL_WIDTH) * 0.5),
            ..Default::default()
        });

    // east wall
    commands
        .spawn(Collider::cuboid(WALL_WIDTH * 0.5, WALL_HEIGHT * 0.5, (WORLD_SIZE.y - WALL_WIDTH) * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, OBJECT_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(WALL_WIDTH, WALL_HEIGHT, WORLD_SIZE.y - WALL_WIDTH))),
            material: materials.add(WALL_COLOR),
            transform: Transform::from_xyz((WORLD_SIZE.x - WALL_WIDTH) * 0.5, WALL_HEIGHT * 0.5, -WALL_WIDTH * 0.5),
            ..Default::default()
        });

    // south wall
    commands
        .spawn(Collider::cuboid((WORLD_SIZE.x - WALL_WIDTH) * 0.5, WALL_HEIGHT * 0.5, WALL_WIDTH * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, OBJECT_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(WORLD_SIZE.x - WALL_WIDTH, WALL_HEIGHT, WALL_WIDTH))),
            material: materials.add(WALL_COLOR),
            transform: Transform::from_xyz(WALL_WIDTH * 0.5, WALL_HEIGHT * 0.5, (WORLD_SIZE.y - WALL_WIDTH) * 0.5),
            ..Default::default()
        });

    // west wall
    commands
        .spawn(Collider::cuboid(WALL_WIDTH * 0.5, WALL_HEIGHT * 0.5, (WORLD_SIZE.y - WALL_WIDTH) * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, OBJECT_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(WALL_WIDTH, WALL_HEIGHT, WORLD_SIZE.y - WALL_WIDTH))),
            material: materials.add(WALL_COLOR),
            transform: Transform::from_xyz((-WORLD_SIZE.x + WALL_WIDTH) * 0.5, WALL_HEIGHT * 0.5, WALL_WIDTH * 0.5),
            ..Default::default()
        });

    // floor
    commands
        .spawn(Collider::cuboid(0.5 * WORLD_SIZE.x, 0.1, 0.5 * WORLD_SIZE.y))
        .insert(CollisionGroups::new(STATIC_GROUP, OBJECT_GROUP))
        .insert(SpatialBundle::from_transform(Transform::from_xyz(0.0, -0.1, 0.0)))
        .with_children(|commands| {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Plane3d::default().mesh().size(WORLD_SIZE.x, WORLD_SIZE.y)),
                material: materials.add(Color::rgba(0.9, 0.9, 0.9, 1.0)),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                ..Default::default()
            });
        });

    // Add balls
    const BALL_RADIUS: f32 = 0.075;
    const BALL_MASS: f32 = 0.1;
    const BALL_COLOR: Color = Color::rgb(0.7, 0.0, 0.0);

    commands
        .spawn(Collider::ball(BALL_RADIUS))
        .insert(CollisionGroups::new(OBJECT_GROUP, OBJECT_GROUP | STATIC_GROUP))
        .insert(Restitution::new(1.0))
        .insert(RigidBody::Dynamic)
        .insert(ColliderMassProperties::Mass(BALL_MASS))
        .insert(DraggableBundle::default())
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere::new(BALL_RADIUS))),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_xyz(0.0, BALL_RADIUS, 0.0),
            ..Default::default()
        });


}

