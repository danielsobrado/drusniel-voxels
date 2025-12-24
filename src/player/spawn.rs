use bevy::prelude::*;

use super::{PlayerBundle, PlayerConfig};

/// Spawn the player at game start.
pub fn spawn_player(mut commands: Commands, config: Res<PlayerConfig>) {
    let spawn_position = Vec3::new(0.0, 50.0, 0.0);
    commands.spawn(PlayerBundle::new(spawn_position, config.clone()));
}
