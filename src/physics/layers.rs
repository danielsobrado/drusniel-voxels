use avian3d::prelude::{LayerMask, PhysicsLayer};

/// Physics collision layers for the voxel game.
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    /// Terrain chunks (static colliders).
    Terrain,
    /// Player character.
    Player,
    /// NPCs and creatures.
    Entity,
    /// Water volumes (sensors only).
    Water,
    /// Building pieces.
    Building,
    /// Projectiles.
    Projectile,
}

impl CollisionLayer {
    /// Player collides with terrain, buildings, entities.
    pub fn player_mask() -> LayerMask {
        LayerMask::from([
            CollisionLayer::Terrain,
            CollisionLayer::Building,
            CollisionLayer::Entity,
        ])
    }

    /// Terrain collides with everything except water.
    pub fn terrain_mask() -> LayerMask {
        LayerMask::from([
            CollisionLayer::Player,
            CollisionLayer::Entity,
            CollisionLayer::Building,
            CollisionLayer::Projectile,
        ])
    }
}
