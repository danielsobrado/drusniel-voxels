pub mod wolf;
pub mod rabbit;
pub mod inventory;

use bevy::prelude::*;

pub use wolf::{Wolf, WolfSpawned};
pub use rabbit::{Rabbit, RabbitSpawned};
pub use inventory::{Inventory, ItemType, ItemDrop};

/// Component for entities with health
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
        }
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Component to mark entities that should be removed
#[derive(Component)]
pub struct Dead;

/// System to handle entity death
pub fn handle_death(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform), (Without<Dead>, Changed<Health>)>,
) {
    for (entity, health, transform) in query.iter() {
        if health.is_dead() {
            info!("Entity died at {:?}", transform.translation);
            
            // Mark as dead
            commands.entity(entity).insert(Dead);
            
            // Check if it's a wolf and drop fur
            commands.entity(entity).insert(ItemDrop {
                item_type: ItemType::Fur,
                position: transform.translation,
            });
        }
    }
}

/// System to process item drops and add to inventory
pub fn process_item_drops(
    mut commands: Commands,
    query: Query<(Entity, &ItemDrop)>,
    mut inventory: ResMut<Inventory>,
) {
    for (entity, drop) in query.iter() {
        inventory.add_item(drop.item_type);
        info!("Collected {:?}! Inventory: {:?}", drop.item_type, inventory);
        
        // Remove the drop entity
        commands.entity(entity).despawn();
    }
}

/// System to despawn dead entities
pub fn despawn_dead(
    mut commands: Commands,
    query: Query<Entity, With<Dead>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Plugin for entity system
pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Inventory>()
            .init_resource::<WolfSpawned>()
            .init_resource::<RabbitSpawned>()
            .add_systems(Startup, rabbit::setup_rabbit_assets)
            .add_systems(Update, (
                wolf::spawn_wolves,
                wolf::animate_wolves,
                rabbit::spawn_rabbits,
                rabbit::animate_rabbits,
                rabbit::fix_rabbit_textures,
                handle_death,
                process_item_drops,
                despawn_dead.after(process_item_drops),
            ));
    }
}
