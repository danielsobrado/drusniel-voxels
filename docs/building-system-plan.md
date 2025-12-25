# Construction System Project Plan

## Overview

A modular building system inspired by Enshrouded/Valheim for a Bevy 0.17 voxel game with Surface Nets terrain. Uses Avian physics, Valheim-style stability propagation, and precomputed physics-based collapse.

---

## Phase 1: Core Data Structures & Configuration

### Task 1.1: Building Piece Registry

**Objective**: Define all building piece types, their properties, and snap point configurations.

**Technical Details**:
- Create `BuildingPieceType` enum with variants: Foundation, Wall, WallHalf, Doorframe, Window, Floor, Ceiling, RoofSide, RoofCorner, RoofPeak, Stairs, Pillar, Beam
- Each piece type has associated metadata: dimensions, snap points, material requirements, stability properties
- Use YAML configuration for piece definitions (data-driven)
- Implement `BuildingPieceRegistry` as a Bevy `Resource`

**Data Structures**:
```rust
pub struct PieceDefinition {
    pub id: PieceTypeId,
    pub name: String,
    pub dimensions: Vec3,
    pub snap_points: Vec<SnapPointDef>,
    pub collider_shape: ColliderShape,
    pub mesh_path: String,
    pub base_stability: f32,
    pub max_support: f32,
    pub vertical_loss: f32,
    pub horizontal_loss: f32,
    pub allowed_materials: Vec<MaterialId>,
}

pub struct SnapPointDef {
    pub local_offset: Vec3,
    pub direction: Vec3,  // Outward normal
    pub compatible_types: Vec<PieceTypeId>,
    pub snap_group: SnapGroup,  // Floor, Wall, Roof, etc.
}
```

**Research Items**:
- [ ] Analyze Valheim prefab structure for snap point patterns
- [ ] Document Enshrouded's 2M/4M piece size system
- [ ] List all piece variants needed for basic building (minimum viable set)
- [ ] Research Unity/Godot snap point implementations for patterns

**Deliverables**:
- `src/building/registry.rs` - Piece registry resource
- `src/building/types.rs` - Core type definitions
- `assets/config/building_pieces.yaml` - Piece definitions
- `assets/config/materials.yaml` - Material properties

---

### Task 1.2: Material System

**Objective**: Define building materials with stability properties matching Valheim's model.

**Technical Details**:
- Materials: Wood, HardWood, Stone, Metal, Thatch
- Each material defines: MaxSupport, MinSupport, VerticalLoss%, HorizontalLoss%
- Materials affect visual appearance (texture/mesh variants)
- Materials define crafting requirements

**Material Properties Table** (from Valheim analysis):
| Material | MaxSupport | MinSupport | VerticalLoss | HorizontalLoss |
|----------|------------|------------|--------------|----------------|
| Wood     | 100        | 10         | 12.5%        | 20%            |
| HardWood | 140        | 10         | 10%          | 16.7%          |
| Stone    | 1000       | 100        | 12.5%        | 100%           |
| Metal    | 1500       | 20         | 7.7%         | 7.7%           |
| Thatch   | 50         | 5          | 25%          | 40%            |

**Research Items**:
- [ ] Verify Valheim material values through wiki/decompilation
- [ ] Research material hierarchy reset rules (wood on stone = grounded)
- [ ] Document 7DTD mass/load formula as alternative reference
- [ ] List texture/mesh requirements per material per piece type

**Deliverables**:
- `src/building/materials.rs` - Material definitions
- `assets/config/materials.yaml` - Material configuration

---

### Task 1.3: Grid & Spatial Index

**Objective**: Implement spatial data structures for O(1) piece lookups and snap detection.

**Technical Details**:
- World-aligned grid with configurable cell size (default 2m matching piece sizes)
- `HashMap<IVec3, Entity>` for occupied cell lookup
- Spatial hash for snap point queries (separate from piece grid)
- Support for pieces spanning multiple cells
- Chunk-based storage for memory efficiency

**Data Structures**:
```rust
#[derive(Resource)]
pub struct BuildingGrid {
    pub cell_size: f32,
    pub cells: HashMap<IVec3, GridCell>,
}

pub struct GridCell {
    pub piece_entity: Entity,
    pub piece_type: PieceTypeId,
    pub snap_points_here: Vec<(Entity, usize)>,  // (piece, snap_index)
}

#[derive(Resource)]
pub struct SnapPointIndex {
    pub cell_size: f32,  // Smaller than grid for precision
    pub points: HashMap<IVec3, Vec<SnapPointRef>>,
}
```

**Research Items**:
- [ ] Benchmark HashMap vs BTreeMap for sparse 3D grids
- [ ] Research bevy_spatial or similar crates for spatial indexing
- [ ] Analyze memory usage patterns for large builds (1000+ pieces)
- [ ] Document optimal cell sizes for different piece scales

**Deliverables**:
- `src/building/grid.rs` - Grid and spatial index
- `src/building/spatial.rs` - Snap point spatial queries

---

## Phase 2: Placement System

### Task 2.1: Ghost Preview & Validation

**Objective**: Show placement preview with validity feedback before confirming placement.

**Technical Details**:
- Ghost entity with semi-transparent material
- Color coding: Green (valid), Red (invalid), Blue (snapped)
- Validation checks: terrain clearance, piece overlap, snap compatibility, build zone
- Ghost follows cursor raycast against terrain and existing pieces
- Rotation in 90° increments (Q/E keys or scroll wheel)

**Systems**:
1. `update_ghost_position` - Raycast and position ghost
2. `update_ghost_snap` - Find nearest valid snap point
3. `validate_placement` - Check all placement rules
4. `update_ghost_visual` - Set material based on validity

**Research Items**:
- [ ] Research Bevy material swapping for ghost transparency
- [ ] Document Enshrouded's snap detection radius and priority
- [ ] Analyze Valheim's placement validation order
- [ ] Research overlap detection with Avian shape casts

**Deliverables**:
- `src/building/ghost.rs` - Ghost preview entity management
- `src/building/validation.rs` - Placement validation rules
- `assets/shaders/ghost_material.wgsl` - Transparent ghost shader

---

### Task 2.2: Snap Point Detection

**Objective**: Find and prioritize valid snap points for piece placement.

**Technical Details**:
- Query spatial index for nearby snap points within radius
- Filter by compatibility (piece type, snap group)
- Score candidates by: distance, direction alignment, snap group priority
- Handle multi-snap (piece connecting to multiple points simultaneously)
- Compute final transform from winning snap point pair

**Algorithm**:
```
1. Get cursor world position from raycast
2. Query snap index for points within SNAP_RADIUS (default 0.5m)
3. For each candidate snap point on existing pieces:
   a. Check if new piece has compatible snap point
   b. Calculate alignment score (direction dot product)
   c. Calculate distance score (inverse distance)
   d. Combined score = alignment * 0.6 + distance * 0.4
4. Sort by score, return best match
5. Compute transform: align new piece snap to target snap
```

**Research Items**:
- [ ] Document Valheim snap point priority system
- [ ] Research multi-point snapping (floors snapping to 4 walls)
- [ ] Analyze "Extra Snap Points Made Easy" mod for extended patterns
- [ ] Benchmark spatial query performance with 100+ nearby points

**Deliverables**:
- `src/building/snap.rs` - Snap detection and scoring
- `src/building/transform.rs` - Transform computation from snap pairs

---

### Task 2.3: Terrain Integration

**Objective**: Handle building placement on Surface Nets terrain.

**Technical Details**:
- Foundations can embed partially into terrain
- Sample terrain SDF at placement corners to determine ground contact
- Optional terrain carving (SDF subtraction) for embedded foundations
- Ground contact grants "grounded" status for stability
- Terrain modification triggers chunk remesh

**Terrain Sampling**:
```rust
fn check_terrain_contact(
    terrain: &SdfTerrain,
    piece_bounds: &Aabb,
    embed_depth: f32,
) -> TerrainContact {
    // Sample SDF at bottom corners
    let corners = piece_bounds.bottom_corners();
    let samples: Vec<f32> = corners.iter()
        .map(|p| terrain.sample(*p))
        .collect();
    
    // Negative SDF = inside terrain
    let embedded_count = samples.iter().filter(|s| **s < 0.0).count();
    
    TerrainContact {
        is_grounded: embedded_count >= 2,
        embed_depths: samples,
    }
}
```

**Research Items**:
- [ ] Document fast_surface_nets SDF modification API
- [ ] Research chunk dirty flagging for partial remesh
- [ ] Analyze Valheim terrain flattening tool implementation
- [ ] Benchmark terrain carving performance impact

**Deliverables**:
- `src/building/terrain.rs` - Terrain sampling and carving
- Integration with existing `src/voxel/` terrain system

---

### Task 2.4: Piece Spawning

**Objective**: Spawn building piece entities with all required components.

**Technical Details**:
- Load mesh and materials from registry
- Create Avian static collider from piece definition
- Add to building grid and snap index
- Initialize stability component
- Trigger stability recalculation for connected pieces
- Play placement sound/particle effect

**Entity Components**:
```rust
#[derive(Bundle)]
pub struct BuildingPieceBundle {
    // Identity
    pub piece: BuildingPiece,
    pub piece_type: PieceTypeId,
    pub material: BuildingMaterial,
    
    // Spatial
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub grid_position: GridPosition,
    
    // Physics
    pub rigid_body: RigidBody,  // Static
    pub collider: Collider,
    pub collision_layers: CollisionLayers,
    
    // Building system
    pub stability: Stability,
    pub connections: PieceConnections,
    pub snap_points: ActiveSnapPoints,
    
    // Rendering
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub visibility: Visibility,
}
```

**Research Items**:
- [ ] Research Bevy asset loading patterns for mesh/material pairs
- [ ] Document Avian static collider creation from mesh
- [ ] Analyze batched entity spawning for performance
- [ ] Research audio cue integration (bevy_kira_audio or similar)

**Deliverables**:
- `src/building/spawn.rs` - Piece spawning system
- `src/building/components.rs` - All building components

---

## Phase 3: Stability System

### Task 3.1: Support Graph

**Objective**: Track piece connections as a directed graph for stability propagation.

**Technical Details**:
- Each piece maintains list of supporting pieces (incoming edges)
- Each piece maintains list of supported pieces (outgoing edges)
- Graph updates on piece placement/destruction
- Detect "grounded" status through graph traversal to terrain-touching pieces
- Support material hierarchy resets (wood on stone = new grounded root)

**Data Structures**:
```rust
#[derive(Component)]
pub struct PieceConnections {
    pub supports_me: Vec<Entity>,    // Pieces I rest on
    pub i_support: Vec<Entity>,      // Pieces resting on me
    pub snap_connections: Vec<(Entity, SnapConnection)>,
}

pub struct SnapConnection {
    pub my_snap_index: usize,
    pub their_snap_index: usize,
    pub connection_type: ConnectionType,  // Vertical, Horizontal, Diagonal
}

#[derive(Resource)]
pub struct StabilityGraph {
    pub grounded_pieces: HashSet<Entity>,
    pub dirty_pieces: HashSet<Entity>,  // Need recalculation
}
```

**Research Items**:
- [ ] Research petgraph crate for graph algorithms
- [ ] Document Valheim's WearNTear connection detection
- [ ] Analyze BFS vs DFS for stability propagation order
- [ ] Research incremental graph updates vs full rebuild

**Deliverables**:
- `src/building/graph.rs` - Support graph structure
- `src/building/connections.rs` - Connection detection

---

### Task 3.2: Stability Calculation

**Objective**: Implement Valheim-style stability value propagation.

**Technical Details**:
- Grounded pieces start at MaxSupport for their material
- Stability propagates through connections with directional loss
- Vertical connections: lose VerticalLoss% per step
- Horizontal connections: lose HorizontalLoss% per step
- Piece is stable if current_stability >= MinSupport
- Material hierarchy: placing on higher-tier material resets to grounded

**Algorithm**:
```
1. Mark all terrain-touching pieces as grounded (stability = MaxSupport)
2. Add grounded pieces to processing queue
3. While queue not empty:
   a. Pop piece P from queue
   b. For each piece C that P supports:
      - Calculate connection type (vertical/horizontal)
      - new_stability = P.stability * (1 - loss_factor)
      - If material_hierarchy_reset(P, C): new_stability = C.MaxSupport
      - If new_stability > C.stability:
        - C.stability = new_stability
        - Add C to queue
4. Any piece with stability < MinSupport is unstable
```

**Research Items**:
- [ ] Verify Valheim stability formula through testing
- [ ] Document edge cases: diagonal connections, multi-support
- [ ] Research stability calculation frequency (every frame? on change?)
- [ ] Analyze Valheim's 50-component-per-cycle throttling

**Deliverables**:
- `src/building/stability.rs` - Stability calculation system
- `src/building/propagation.rs` - BFS propagation algorithm

---

### Task 3.3: Visual Feedback

**Objective**: Show stability status through color-coded visual feedback.

**Technical Details**:
- Color gradient: Blue (grounded) → Green → Yellow → Orange → Red (unstable)
- Toggle with key press (H for "health" or stability view)
- Overlay shader or vertex colors on building pieces
- Update colors when stability changes
- Optional: particle effects on low-stability pieces

**Color Mapping**:
```rust
fn stability_to_color(stability: f32, max: f32, min: f32) -> Color {
    let normalized = (stability - min) / (max - min);
    match normalized {
        n if n >= 1.0 => Color::BLUE,      // Grounded
        n if n >= 0.75 => Color::GREEN,
        n if n >= 0.50 => Color::YELLOW,
        n if n >= 0.25 => Color::ORANGE,
        _ => Color::RED,                    // Near collapse
    }
}
```

**Research Items**:
- [ ] Research Bevy material parameter animation
- [ ] Document Valheim's stability color implementation
- [ ] Analyze vertex color vs overlay approach performance
- [ ] Research UI overlay for stability percentage display

**Deliverables**:
- `src/building/stability_visual.rs` - Visual feedback system
- `assets/shaders/stability_overlay.wgsl` - Optional overlay shader

---

## Phase 4: Destruction & Collapse

### Task 4.1: Piece Destruction

**Objective**: Handle building piece removal with graph updates.

**Technical Details**:
- Remove piece from grid and snap index
- Update support graph (remove edges)
- Mark all previously-supported pieces as dirty
- Trigger stability recalculation
- Drop resources/items from destroyed piece
- Check for collapse cascade

**Destruction Flow**:
```
1. Player destroys piece P
2. For each piece C in P.i_support:
   a. Remove P from C.supports_me
   b. Mark C as dirty
3. For each piece S in P.supports_me:
   a. Remove P from S.i_support
4. Remove P from grid, snap index, stability graph
5. Despawn P entity
6. Run stability recalculation on dirty pieces
7. Identify pieces now below MinSupport
8. Trigger collapse for unstable pieces
```

**Research Items**:
- [ ] Research destruction animation/particle effects
- [ ] Document item drop system integration
- [ ] Analyze cascading destruction performance
- [ ] Research undo/redo for accidental destruction

**Deliverables**:
- `src/building/destruction.rs` - Piece removal system
- Integration with inventory/item drop systems

---

### Task 4.2: Collapse Detection

**Objective**: Identify when pieces should collapse and prepare physics simulation.

**Technical Details**:
- After stability recalculation, collect pieces below MinSupport
- Group connected unstable pieces into "collapse clusters"
- Precompute initial velocities based on support direction
- Determine collapse trigger: immediate vs delayed (dramatic effect)
- Option: show warning particles/sounds before collapse

**Collapse Cluster Detection**:
```rust
fn find_collapse_clusters(
    unstable: &[Entity],
    graph: &StabilityGraph,
) -> Vec<CollapseCluster> {
    // Union-Find to group connected unstable pieces
    let mut clusters = Vec::new();
    let mut visited = HashSet::new();
    
    for &piece in unstable {
        if visited.contains(&piece) { continue; }
        
        let cluster = bfs_connected_unstable(piece, unstable, graph);
        for &p in &cluster {
            visited.insert(p);
        }
        clusters.push(CollapseCluster {
            pieces: cluster,
            center_of_mass: compute_com(&cluster),
            total_mass: compute_mass(&cluster),
        });
    }
    clusters
}
```

**Research Items**:
- [ ] Research Union-Find algorithm for clustering
- [ ] Document Valheim's collapse cascade timing
- [ ] Analyze dramatic collapse effects (delay, sound, camera shake)
- [ ] Research collapse prediction for multiplayer sync

**Deliverables**:
- `src/building/collapse_detection.rs` - Instability detection
- `src/building/collapse_cluster.rs` - Cluster grouping

---

### Task 4.3: Physics-Based Collapse

**Objective**: Convert static pieces to dynamic rigid bodies for physics simulation.

**Technical Details**:
- Precompute collapse when stability approaches MinSupport (early warning)
- On collapse trigger, convert pieces from Static to Dynamic rigid bodies
- Apply initial angular velocity based on support loss direction
- Enable piece-to-piece and piece-to-terrain collision
- Despawn pieces after settling or timeout
- Spawn debris/items at rest positions

**Precomputation Strategy**:
```rust
#[derive(Component)]
pub struct CollapsePrecomputed {
    pub initial_velocity: Vec3,
    pub initial_angular: Vec3,
    pub estimated_trajectory: Vec<Vec3>,  // For multiplayer prediction
    pub time_to_collapse: f32,
}

fn precompute_collapse(
    query: Query<(Entity, &Stability, &Transform, &PieceConnections)>,
    threshold: f32,  // e.g., MinSupport * 1.2 = "warning zone"
) {
    for (entity, stability, transform, connections) in query.iter() {
        if stability.current < threshold && stability.current >= stability.min {
            // Compute expected fall direction from support geometry
            let support_center = average_support_position(&connections);
            let to_center = (support_center - transform.translation).normalize();
            
            // Initial tip direction is away from remaining support
            let tip_direction = -to_center;
            let angular = tip_direction.cross(Vec3::Y) * 2.0;
            
            commands.entity(entity).insert(CollapsePrecomputed {
                initial_velocity: Vec3::ZERO,
                initial_angular: angular,
                estimated_trajectory: simulate_trajectory(transform, angular),
                time_to_collapse: (stability.current - stability.min) / DECAY_RATE,
            });
        }
    }
}
```

**Avian Conversion**:
```rust
fn trigger_collapse(
    mut commands: Commands,
    collapse_pieces: Query<(Entity, &CollapsePrecomputed)>,
) {
    for (entity, precomputed) in collapse_pieces.iter() {
        commands.entity(entity)
            // Convert to dynamic
            .remove::<RigidBody>()
            .insert(RigidBody::Dynamic)
            // Apply precomputed impulses
            .insert(LinearVelocity(precomputed.initial_velocity))
            .insert(AngularVelocity(precomputed.initial_angular))
            // Change collision layer for piece-piece collision
            .insert(CollisionLayers::new(
                PhysicsLayer::Debris,
                [PhysicsLayer::Terrain, PhysicsLayer::Debris, PhysicsLayer::Building],
            ))
            // Mark for cleanup
            .insert(CollapsingPiece {
                despawn_timer: Timer::from_seconds(5.0, TimerMode::Once),
            });
    }
}
```

**Research Items**:
- [ ] Benchmark Avian dynamic body conversion performance
- [ ] Research Avian sleep detection for settled pieces
- [ ] Document debris density limits (max simultaneous dynamic bodies)
- [ ] Analyze multiplayer collapse synchronization strategies

**Deliverables**:
- `src/building/collapse_physics.rs` - Physics conversion system
- `src/building/collapse_precompute.rs` - Early trajectory calculation
- `src/building/debris.rs` - Debris cleanup and item spawning

---

### Task 4.4: Collapse Optimization

**Objective**: Ensure collapse simulation doesn't tank framerate.

**Technical Details**:
- Limit maximum simultaneous dynamic pieces (e.g., 50)
- Use simplified colliders for debris (convex hull, not trimesh)
- Implement LOD for distant collapses (instant despawn, no physics)
- Frame budget: allocate max N ms per frame to collapse simulation
- Pool debris entities for reuse

**Optimization Strategies**:
```rust
#[derive(Resource)]
pub struct CollapseConfig {
    pub max_dynamic_pieces: usize,      // 50
    pub max_collapse_time: f32,         // 5 seconds
    pub lod_distance: f32,              // 50m - beyond this, instant collapse
    pub frame_budget_ms: f32,           // 2ms max per frame
    pub use_simplified_colliders: bool, // true
}

fn manage_collapse_budget(
    config: Res<CollapseConfig>,
    active: Query<Entity, With<CollapsingPiece>>,
    pending: Query<(Entity, &CollapsePrecomputed)>,
) {
    let active_count = active.iter().count();
    let available_slots = config.max_dynamic_pieces.saturating_sub(active_count);
    
    // Only trigger more collapses if under budget
    let to_trigger: Vec<_> = pending.iter()
        .take(available_slots)
        .collect();
    
    // Trigger limited batch
    for (entity, _) in to_trigger {
        trigger_collapse(entity);
    }
}
```

**Research Items**:
- [ ] Benchmark Avian with 50 vs 100 vs 200 dynamic bodies
- [ ] Research convex hull generation for simplified colliders
- [ ] Document entity pooling patterns in Bevy
- [ ] Analyze frame time profiling for physics systems

**Deliverables**:
- `src/building/collapse_budget.rs` - Performance management
- `src/building/collider_simplify.rs` - Simplified debris colliders

---

## Phase 5: Building UI & Tools

### Task 5.1: Building Menu

**Objective**: UI for selecting building pieces and materials.

**Technical Details**:
- Radial menu or hotbar for piece categories
- Sub-menus for piece variants within category
- Material selector (unlocked materials only)
- Show resource requirements and current inventory
- Keyboard shortcuts for quick selection

**UI Structure**:
```
[1] Foundations  →  [Wood Floor 2x2] [Stone Floor 2x2] ...
[2] Walls        →  [Wall] [Half Wall] [Doorframe] [Window] ...
[3] Roofs        →  [26° Side] [45° Side] [Corner] [Peak] ...
[4] Stairs       →  [Straight] [Spiral] [Ladder] ...
[5] Decorations  →  [Pillar] [Beam] [Fence] ...
[M] Materials    →  [Wood] [Stone] [Metal] ...
```

**Research Items**:
- [ ] Research bevy_egui radial menu implementation
- [ ] Document Valheim/Enshrouded building menu UX patterns
- [ ] Analyze controller support for building UI
- [ ] Research drag-and-drop inventory integration

**Deliverables**:
- `src/building/ui/menu.rs` - Building menu system
- `src/building/ui/piece_selector.rs` - Piece selection widget
- `src/building/ui/material_selector.rs` - Material picker

---

### Task 5.2: Building Tools

**Objective**: Implement hammer, repair, and demolish tools.

**Technical Details**:
- Hammer: Primary build tool, shows ghost, places pieces
- Repair: Restore damaged pieces (if damage system exists)
- Demolish: Destroy pieces, recover partial resources
- Tool switching with number keys or radial menu
- Tool-specific cursor and interaction feedback

**Tool States**:
```rust
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum BuildingTool {
    #[default]
    None,
    Hammer,
    Repair,
    Demolish,
}

#[derive(Resource)]
pub struct BuildingToolState {
    pub selected_piece: Option<PieceTypeId>,
    pub selected_material: MaterialId,
    pub rotation: u8,  // 0-3 for 90° increments
    pub snap_enabled: bool,
}
```

**Research Items**:
- [ ] Document Valheim hammer tool mechanics
- [ ] Research tool equip/unequip animation integration
- [ ] Analyze repair cost formulas
- [ ] Research demolish resource return rates

**Deliverables**:
- `src/building/tools/mod.rs` - Tool system
- `src/building/tools/hammer.rs` - Build tool
- `src/building/tools/demolish.rs` - Destruction tool

---

### Task 5.3: Build Zone System

**Objective**: Define areas where building is permitted.

**Technical Details**:
- Workbench/Altar defines build zone center
- Configurable radius (default 30m, upgradeable)
- Visual indicator of build zone boundary
- Multiple zones can overlap
- Cannot build in certain areas (dungeons, spawn points)

**Research Items**:
- [ ] Document Enshrouded Flame Altar zone system
- [ ] Research Valheim workbench coverage mechanics
- [ ] Analyze zone visualization approaches (shader, decal, particles)
- [ ] Document restricted zone implementation

**Deliverables**:
- `src/building/zones.rs` - Build zone management
- `src/building/zone_visual.rs` - Zone boundary rendering

---

## Phase 6: Persistence & Multiplayer Prep

### Task 6.1: Save/Load

**Objective**: Serialize and deserialize building state.

**Technical Details**:
- Save: piece type, material, transform, stability, connections
- Compact binary format (bincode) for release
- Human-readable (RON) for debugging
- Incremental saves (only modified pieces)
- Load rebuilds grid, snap index, and stability graph

**Save Format**:
```rust
#[derive(Serialize, Deserialize)]
pub struct BuildingSaveData {
    pub version: u32,
    pub pieces: Vec<PieceSaveData>,
    pub zones: Vec<ZoneSaveData>,
}

#[derive(Serialize, Deserialize)]
pub struct PieceSaveData {
    pub id: u64,
    pub piece_type: String,
    pub material: String,
    pub position: [f32; 3],
    pub rotation: u8,
    pub stability: f32,
    pub connected_to: Vec<u64>,
}
```

**Research Items**:
- [ ] Research bevy_save crate for Bevy 0.17
- [ ] Document incremental save strategies
- [ ] Analyze save file compression (LZ4, zstd)
- [ ] Research save migration for piece definition changes

**Deliverables**:
- `src/building/save.rs` - Serialization system
- `src/building/load.rs` - Deserialization and reconstruction

---

### Task 6.2: Multiplayer Considerations

**Objective**: Design building system for future multiplayer support.

**Technical Details**:
- Placement requests sent to server for validation
- Server authoritative for stability calculations
- Collapse events broadcast with precomputed trajectories
- Client-side prediction for placement feedback
- Conflict resolution for simultaneous modifications

**Network Events**:
```rust
pub enum BuildingNetEvent {
    PlaceRequest { piece_type: PieceTypeId, transform: Transform },
    PlaceConfirm { entity_id: u64, piece_data: PieceSaveData },
    PlaceReject { reason: String },
    DestroyRequest { entity_id: u64 },
    DestroyConfirm { entity_id: u64, drop_items: Vec<ItemDrop> },
    CollapseEvent { cluster: CollapseCluster, trajectories: Vec<Trajectory> },
    StabilitySync { updates: Vec<(u64, f32)> },
}
```

**Research Items**:
- [ ] Research bevy_replicon or lightyear for networking
- [ ] Document client-side prediction patterns
- [ ] Analyze bandwidth for building sync (pieces per second)
- [ ] Research deterministic physics for collapse sync

**Deliverables**:
- `src/building/network.rs` - Network event definitions
- Documentation: multiplayer architecture design doc

---

## Phase 7: Polish & Optimization

### Task 7.1: Audio & Effects

**Objective**: Add sound effects and particles for building actions.

**Technical Details**:
- Placement sounds per material type
- Destruction sounds with debris audio
- Collapse rumble with distance attenuation
- Snap feedback sound (subtle click)
- Construction dust particles
- Collapse debris particles

**Research Items**:
- [ ] Research bevy_kira_audio for spatial audio
- [ ] Source/create placeholder sound effects
- [ ] Document bevy_hanabi particle integration
- [ ] Analyze audio pooling for many simultaneous sounds

**Deliverables**:
- `src/building/audio.rs` - Sound effect system
- `src/building/particles.rs` - Particle effects
- `assets/audio/building/` - Sound assets
- Particle configurations in YAML

---

### Task 7.2: Performance Profiling

**Objective**: Identify and fix performance bottlenecks.

**Technical Details**:
- Profile stability calculation with 500+ pieces
- Profile snap detection with dense piece clusters
- Profile collapse simulation with 50 dynamic bodies
- Identify memory usage patterns
- Optimize hot paths

**Profiling Targets**:
| System | Target | Budget |
|--------|--------|--------|
| Stability recalc (full) | < 5ms | On piece change only |
| Stability recalc (incremental) | < 0.5ms | Per dirty piece |
| Snap detection | < 0.1ms | Per frame during placement |
| Ghost update | < 0.2ms | Per frame during placement |
| Collapse physics | < 2ms | Per frame during collapse |

**Research Items**:
- [ ] Research Tracy profiler integration with Bevy
- [ ] Document Bevy system ordering for parallelization
- [ ] Analyze cache-friendly data layouts
- [ ] Research SIMD optimization for stability math

**Deliverables**:
- Performance benchmark suite
- Optimization report with findings
- Optimized implementations where needed

---

## Dependency Graph

```
Phase 1 (Foundation)
├── 1.1 Piece Registry
├── 1.2 Material System
└── 1.3 Grid & Spatial Index

Phase 2 (Placement) - depends on Phase 1
├── 2.1 Ghost Preview ─────┐
├── 2.2 Snap Detection ────┼── 2.4 Piece Spawning
├── 2.3 Terrain Integration┘
└── 2.4 Piece Spawning

Phase 3 (Stability) - depends on Phase 2
├── 3.1 Support Graph
├── 3.2 Stability Calculation ── depends on 3.1
└── 3.3 Visual Feedback ──────── depends on 3.2

Phase 4 (Destruction) - depends on Phase 3
├── 4.1 Piece Destruction
├── 4.2 Collapse Detection ───── depends on 4.1
├── 4.3 Physics Collapse ─────── depends on 4.2
└── 4.4 Collapse Optimization ── depends on 4.3

Phase 5 (UI) - can parallel with Phase 3+
├── 5.1 Building Menu
├── 5.2 Building Tools
└── 5.3 Build Zone System

Phase 6 (Persistence) - depends on Phase 4
├── 6.1 Save/Load
└── 6.2 Multiplayer Prep

Phase 7 (Polish) - depends on Phase 6
├── 7.1 Audio & Effects
└── 7.2 Performance Profiling
```

---

## File Structure

```
src/building/
├── mod.rs
├── components.rs          # All building ECS components
├── types.rs               # Core type definitions
├── constants.rs           # Magic numbers, thresholds
│
├── registry/
│   ├── mod.rs
│   ├── pieces.rs          # Piece definitions
│   └── materials.rs       # Material properties
│
├── grid/
│   ├── mod.rs
│   ├── spatial.rs         # Grid structure
│   └── snap_index.rs      # Snap point spatial hash
│
├── placement/
│   ├── mod.rs
│   ├── ghost.rs           # Preview entity
│   ├── validation.rs      # Placement rules
│   ├── snap.rs            # Snap detection
│   └── terrain.rs         # Terrain integration
│
├── stability/
│   ├── mod.rs
│   ├── graph.rs           # Support graph
│   ├── calculation.rs     # Propagation algorithm
│   └── visual.rs          # Color feedback
│
├── collapse/
│   ├── mod.rs
│   ├── detection.rs       # Instability detection
│   ├── cluster.rs         # Grouping algorithm
│   ├── physics.rs         # Avian conversion
│   ├── precompute.rs      # Early trajectory calc
│   └── budget.rs          # Performance limits
│
├── tools/
│   ├── mod.rs
│   ├── hammer.rs
│   ├── demolish.rs
│   └── repair.rs
│
├── ui/
│   ├── mod.rs
│   ├── menu.rs
│   └── selectors.rs
│
├── zones.rs               # Build zone management
├── destruction.rs         # Piece removal
├── spawn.rs               # Piece creation
├── save.rs                # Serialization
├── load.rs                # Deserialization
├── audio.rs               # Sound effects
├── particles.rs           # Visual effects
└── plugin.rs              # Main plugin
```

---

## Configuration Files

```
assets/config/building/
├── pieces.yaml            # All piece definitions
├── materials.yaml         # Material properties
├── stability.yaml         # Stability thresholds
├── collapse.yaml          # Physics settings
├── zones.yaml             # Build zone defaults
└── audio.yaml             # Sound mappings
```