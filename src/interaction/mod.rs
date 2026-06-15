use bevy::prelude::*;
use crate::voxel::world::VoxelWorld;
use crate::voxel::types::{VoxelType, Voxel};

/// Component to mark the block highlight entity
#[derive(Component)]
pub struct BlockHighlight;

/// Resource tracking the currently targeted block
#[derive(Resource, Default)]
pub struct TargetedBlock {
    pub position: Option<IVec3>,
    pub normal: Option<IVec3>,
    pub voxel_type: Option<VoxelType>,
}

/// Resource for the player's held block type
#[derive(Resource)]
pub struct HeldBlock {
    pub block_type: VoxelType,
}

impl Default for HeldBlock {
    fn default() -> Self {
        Self {
            block_type: VoxelType::Rock,
        }
    }
}

/// Maximum distance for block interaction
const INTERACTION_RANGE: f32 = 6.0;

/// Raycast step size for block detection
const RAY_STEP: f32 = 0.1;

/// Cast a ray and find the first solid block hit
pub fn raycast_blocks(
    origin: Vec3,
    direction: Vec3,
    world: &VoxelWorld,
    max_distance: f32,
) -> Option<(IVec3, IVec3)> {
    let mut pos = origin;
    let step = direction.normalize() * RAY_STEP;
    let mut prev_block = IVec3::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    );
    
    let steps = (max_distance / RAY_STEP) as i32;
    
    for _ in 0..steps {
        pos += step;
        let block_pos = IVec3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );
        
        if block_pos != prev_block {
            if let Some(voxel) = world.get_voxel(block_pos) {
                if voxel.is_solid() {
                    // Calculate which face we hit based on direction
                    let normal = prev_block - block_pos;
                    return Some((block_pos, normal));
                }
            }
            prev_block = block_pos;
        }
    }
    
    None
}

/// System to update the targeted block based on camera look direction
pub fn update_targeted_block(
    camera_query: Query<&Transform, With<crate::camera::controller::PlayerCamera>>,
    world: Res<VoxelWorld>,
    mut targeted: ResMut<TargetedBlock>,
) {
    if let Ok(transform) = camera_query.single() {
        let origin = transform.translation;
        let direction = transform.forward().as_vec3();
        
        if let Some((block_pos, normal)) = raycast_blocks(origin, direction, &world, INTERACTION_RANGE) {
            targeted.position = Some(block_pos);
            targeted.normal = Some(normal);
            targeted.voxel_type = world.get_voxel(block_pos);
        } else {
            targeted.position = None;
            targeted.normal = None;
            targeted.voxel_type = None;
        }
    }
}

/// System to handle block breaking (left click)
pub fn break_block_system(
    mouse: Res<ButtonInput<MouseButton>>,
    targeted: Res<TargetedBlock>,
    mut world: ResMut<VoxelWorld>,
    mut held: ResMut<HeldBlock>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let (Some(pos), Some(voxel_type)) = (targeted.position, targeted.voxel_type) {
            // Don't break bedrock
            if voxel_type != VoxelType::Bedrock {
                // Store the broken block type for placing
                held.block_type = voxel_type;
                
                // Set to air
                world.set_voxel(pos, VoxelType::Air);
                
                // Mark neighboring chunks dirty too (for proper mesh updates at edges)
                mark_neighbors_dirty(&mut world, pos);
            }
        }
    }
}

/// System to handle block placing (right click)
pub fn place_block_system(
    mouse: Res<ButtonInput<MouseButton>>,
    targeted: Res<TargetedBlock>,
    mut world: ResMut<VoxelWorld>,
    held: Res<HeldBlock>,
    camera_query: Query<&Transform, With<crate::camera::controller::PlayerCamera>>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        if let (Some(block_pos), Some(normal)) = (targeted.position, targeted.normal) {
            // Place block on the face we're looking at
            let place_pos = block_pos + normal;
            
            // Don't place if player is standing there
            if let Ok(camera_transform) = camera_query.single() {
                let player_pos = camera_transform.translation;
                let player_block = IVec3::new(
                    player_pos.x.floor() as i32,
                    player_pos.y.floor() as i32,
                    player_pos.z.floor() as i32,
                );
                let player_feet = IVec3::new(
                    player_pos.x.floor() as i32,
                    (player_pos.y - 1.8).floor() as i32,
                    player_pos.z.floor() as i32,
                );
                
                if place_pos == player_block || place_pos == player_feet {
                    return; // Can't place block where player is standing
                }
            }
            
            // Check if the position is valid (air or water)
            if let Some(existing) = world.get_voxel(place_pos) {
                if existing == VoxelType::Air || existing == VoxelType::Water {
                    world.set_voxel(place_pos, held.block_type);
                    mark_neighbors_dirty(&mut world, place_pos);
                }
            }
        }
    }
}

/// Mark a block and its neighbors as dirty for mesh regeneration
fn mark_neighbors_dirty(world: &mut VoxelWorld, pos: IVec3) {
    // Mark the chunk containing this block
    let chunk_pos = VoxelWorld::world_to_chunk(pos);
    if let Some(chunk) = world.get_chunk_mut(chunk_pos) {
        chunk.mark_dirty();
    }
    
    // Check if we're at a chunk boundary and mark neighbor chunks
    let local = VoxelWorld::world_to_local(pos);
    
    let offsets = [
        (local.x == 0, IVec3::new(-1, 0, 0)),
        (local.x == 15, IVec3::new(1, 0, 0)),
        (local.y == 0, IVec3::new(0, -1, 0)),
        (local.y == 15, IVec3::new(0, 1, 0)),
        (local.z == 0, IVec3::new(0, 0, -1)),
        (local.z == 15, IVec3::new(0, 0, 1)),
    ];
    
    for (at_edge, offset) in offsets {
        if at_edge {
            let neighbor_chunk = chunk_pos + offset;
            if let Some(chunk) = world.get_chunk_mut(neighbor_chunk) {
                chunk.mark_dirty();
            }
        }
    }
}

/// System to render block highlight wireframe
pub fn render_block_highlight(
    targeted: Res<TargetedBlock>,
    mut gizmos: Gizmos,
) {
    if let Some(pos) = targeted.position {
        let center = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);
        let half_size = Vec3::splat(0.505); // Slightly larger than block

        // Draw wireframe cube
        gizmos.cuboid(
            Transform::from_translation(center).with_scale(half_size * 2.0),
            Color::srgba(1.0, 1.0, 1.0, 0.8),
        );
    }
}

/// System to debug voxel info when D is pressed
pub fn debug_voxel_info_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    targeted: Res<TargetedBlock>,
    world: Res<VoxelWorld>,
    camera_query: Query<&Transform, With<crate::camera::controller::PlayerCamera>>,
    sun_query: Query<(&Transform, &DirectionalLight), With<crate::environment::Sun>>,
    ambient: Res<AmbientLight>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        info!("=== DEBUG VOXEL INFO ===");

        // Sun/Light info
        if let Ok((sun_transform, sun_light)) = sun_query.single() {
            let sun_dir = sun_transform.forward().as_vec3();
            info!("Sun direction: ({:.2}, {:.2}, {:.2})", sun_dir.x, sun_dir.y, sun_dir.z);
            info!("Sun illuminance: {:.0}", sun_light.illuminance);
            info!("Sun shadows enabled: {}", sun_light.shadows_enabled);
        }
        info!("Ambient brightness: {:.0}", ambient.brightness);

        // Camera position
        if let Ok(camera) = camera_query.single() {
            let pos = camera.translation;
            info!("Camera pos: ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);

            let block_pos = IVec3::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );
            let chunk_pos = VoxelWorld::world_to_chunk(block_pos);
            info!("Camera chunk: {:?}", chunk_pos);
        }

        // Targeted block info
        if let (Some(pos), Some(voxel_type)) = (targeted.position, targeted.voxel_type) {
            let chunk_pos = VoxelWorld::world_to_chunk(pos);
            let local_pos = VoxelWorld::world_to_local(pos);

            info!("Targeted block:");
            info!("  World pos: {:?}", pos);
            info!("  Chunk pos: {:?}", chunk_pos);
            info!("  Local pos: {:?}", local_pos);
            info!("  Voxel type: {:?}", voxel_type);
            info!("  Atlas index: {}", voxel_type.atlas_index());
            info!("  Is solid: {}", voxel_type.is_solid());
            info!("  Is transparent: {}", voxel_type.is_transparent());

            // Check neighbors
            info!("  Neighbors:");
            let neighbors = [
                ("Top   (+Y)", IVec3::new(0, 1, 0)),
                ("Bottom(-Y)", IVec3::new(0, -1, 0)),
                ("North (+Z)", IVec3::new(0, 0, 1)),
                ("South (-Z)", IVec3::new(0, 0, -1)),
                ("East  (+X)", IVec3::new(1, 0, 0)),
                ("West  (-X)", IVec3::new(-1, 0, 0)),
            ];

            for (name, offset) in neighbors {
                let neighbor_pos = pos + offset;
                let neighbor_chunk = VoxelWorld::world_to_chunk(neighbor_pos);

                match world.get_voxel(neighbor_pos) {
                    Some(n_type) => {
                        let same_chunk = neighbor_chunk == chunk_pos;
                        info!("    {}: {:?} (atlas:{}) chunk:{:?} same_chunk:{}",
                            name, n_type, n_type.atlas_index(), neighbor_chunk, same_chunk);
                    }
                    None => {
                        info!("    {}: NONE (outside world) chunk:{:?}", name, neighbor_chunk);
                    }
                }
            }

            // Check skylight - count solid blocks above
            info!("  Skylight check (blocks above):");
            let mut solid_above = 0;
            for y_offset in 1..=20 {
                let check_pos = pos + IVec3::new(0, y_offset, 0);
                if let Some(voxel) = world.get_voxel(check_pos) {
                    if voxel.is_solid() {
                        solid_above += 1;
                        info!("    y+{}: {:?} (SOLID)", y_offset, voxel);
                    }
                } else {
                    break; // Outside world
                }
            }
            info!("    Total solid blocks above (20 checked): {}", solid_above);

            // Check if chunk exists
            if world.get_chunk(chunk_pos).is_some() {
                info!("  Chunk exists: YES");
            } else {
                info!("  Chunk exists: NO!");
            }
        } else {
            info!("No block targeted");
        }

        info!("========================");
    }
}

/// Plugin for block interaction
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TargetedBlock>()
            .init_resource::<HeldBlock>()
            .add_systems(Update, (
                update_targeted_block,
                break_block_system,
                place_block_system,
                render_block_highlight,
                debug_voxel_info_system,
            ).chain());
    }
}
