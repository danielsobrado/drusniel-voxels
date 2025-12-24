use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

use crate::physics::PhysicsLayer;

/// Player marker component.
#[derive(Component)]
pub struct Player;

/// Player configuration loaded from YAML.
#[derive(Component, Clone, Resource)]
pub struct PlayerConfig {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_height: f32,
    pub float_height: f32,
    pub capsule_radius: f32,
    pub capsule_height: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            walk_speed: 6.0,
            run_speed: 12.0,
            jump_height: 2.0,
            float_height: 1.5,
            capsule_radius: 0.35,
            capsule_height: 1.8,
        }
    }
}

/// Bundle for spawning a player entity.
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub config: PlayerConfig,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
    pub collision_layers: CollisionLayers,
    pub tnua_controller: TnuaController,
    pub tnua_sensor: TnuaAvian3dSensorShape,
}

impl PlayerBundle {
    pub fn new(position: Vec3, config: PlayerConfig) -> Self {
        let half_height = (config.capsule_height - config.capsule_radius * 2.0) / 2.0;

        Self {
            player: Player,
            config: config.clone(),
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(config.capsule_radius, half_height * 2.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            collision_layers: CollisionLayers::new(
                PhysicsLayer::Player,
                PhysicsLayer::player_mask(),
            ),
            tnua_controller: TnuaController::default(),
            tnua_sensor: TnuaAvian3dSensorShape(Collider::capsule(
                config.capsule_radius * 0.9,
                half_height * 1.8,
            )),
        }
    }
}
