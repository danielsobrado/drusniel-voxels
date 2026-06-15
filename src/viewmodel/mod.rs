use bevy::prelude::*;

/// Component marking the pickaxe viewmodel
#[derive(Component)]
pub struct PickaxeViewModel {
    /// Current swing animation progress (0.0 = idle, 1.0 = full swing)
    pub swing_progress: f32,
    /// Is currently swinging
    pub is_swinging: bool,
}

impl Default for PickaxeViewModel {
    fn default() -> Self {
        Self {
            swing_progress: 0.0,
            is_swinging: false,
        }
    }
}

/// Resource to track swing state
#[derive(Resource, Default)]
pub struct PickaxeState {
    pub swing_timer: f32,
    pub swing_duration: f32,
}

/// Spawn the pickaxe viewmodel as a child of the camera
pub fn spawn_pickaxe(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<Entity, With<crate::camera::controller::PlayerCamera>>,
) {
    if let Ok(camera_entity) = camera_query.single() {
        // Create a larger, more visible pickaxe
        
        // Handle (long thin box) - much larger
        let handle_mesh = meshes.add(Cuboid::new(0.08, 0.08, 0.8));
        let handle_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.35, 0.15), // Brighter brown
            emissive: LinearRgba::new(0.1, 0.05, 0.02, 1.0), // Slight glow
            perceptual_roughness: 0.7,
            ..default()
        });
        
        // Pickaxe head (larger flat box)
        let head_mesh = meshes.add(Cuboid::new(0.4, 0.12, 0.12));
        let head_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.75), // Brighter metal
            emissive: LinearRgba::new(0.1, 0.1, 0.12, 1.0), // Slight glow
            perceptual_roughness: 0.2,
            metallic: 0.8,
            ..default()
        });
        
        // Spawn pickaxe as child of camera - positioned more visible
        commands.entity(camera_entity).with_children(|parent| {
            // Parent entity for the whole pickaxe (we animate this)
            parent.spawn((
                Transform::from_xyz(0.5, -0.4, -0.7)
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.4, -0.6, 0.3)),
                Visibility::default(),
                PickaxeViewModel::default(),
            )).with_children(|pickaxe| {
                // Handle
                pickaxe.spawn((
                    Mesh3d(handle_mesh),
                    MeshMaterial3d(handle_material),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));
                
                // Head - at the end of handle
                pickaxe.spawn((
                    Mesh3d(head_mesh),
                    MeshMaterial3d(head_material),
                    Transform::from_xyz(0.0, 0.0, -0.4),
                ));
            });
        });
    }
}

/// System to trigger pickaxe swing when breaking blocks
pub fn trigger_swing_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut pickaxe_query: Query<&mut PickaxeViewModel>,
    mut state: ResMut<PickaxeState>,
) {
    // Start swing on left click
    if mouse.just_pressed(MouseButton::Left) {
        for mut pickaxe in pickaxe_query.iter_mut() {
            if !pickaxe.is_swinging {
                pickaxe.is_swinging = true;
                pickaxe.swing_progress = 0.0;
                state.swing_timer = 0.0;
                state.swing_duration = 0.25; // Quarter second swing
            }
        }
    }
}

/// System to animate the pickaxe swing
pub fn animate_pickaxe_system(
    time: Res<Time>,
    mut state: ResMut<PickaxeState>,
    mut pickaxe_query: Query<(&mut PickaxeViewModel, &mut Transform)>,
) {
    let dt = time.delta_secs();
    
    for (mut pickaxe, mut transform) in pickaxe_query.iter_mut() {
        if pickaxe.is_swinging {
            state.swing_timer += dt;
            pickaxe.swing_progress = (state.swing_timer / state.swing_duration).min(1.0);
            
            // Swing animation curve - quick down, slower return
            let swing_amount = if pickaxe.swing_progress < 0.4 {
                // Swing down (0 -> 0.4 progress = 0 -> 1 swing)
                pickaxe.swing_progress / 0.4
            } else {
                // Return (0.4 -> 1.0 progress = 1 -> 0 swing)
                1.0 - (pickaxe.swing_progress - 0.4) / 0.6
            };
            
            // Apply swing rotation and position offset
            let base_rotation = Quat::from_euler(EulerRot::XYZ, 0.4, -0.6, 0.3);
            let swing_rotation = Quat::from_euler(EulerRot::XYZ, 
                swing_amount * 1.0,  // Pitch down - more dramatic
                swing_amount * 0.4,  // Slight yaw
                swing_amount * -0.3, // Slight roll
            );
            
            transform.rotation = base_rotation * swing_rotation;
            
            // Move forward and down during swing
            transform.translation = Vec3::new(
                0.5 - swing_amount * 0.15,
                -0.4 - swing_amount * 0.2,
                -0.7 + swing_amount * 0.3,
            );
            
            // End swing
            if pickaxe.swing_progress >= 1.0 {
                pickaxe.is_swinging = false;
                pickaxe.swing_progress = 0.0;
                // Reset to idle position
                transform.translation = Vec3::new(0.5, -0.4, -0.7);
                transform.rotation = base_rotation;
            }
        }
    }
}

/// Subtle idle bob animation
pub fn idle_bob_system(
    time: Res<Time>,
    mut pickaxe_query: Query<(&PickaxeViewModel, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    
    for (pickaxe, mut transform) in pickaxe_query.iter_mut() {
        if !pickaxe.is_swinging {
            // Gentle idle bobbing
            let bob = (t * 2.0).sin() * 0.015;
            let sway = (t * 1.5).cos() * 0.008;
            
            transform.translation.y = -0.4 + bob;
            transform.translation.x = 0.5 + sway;
        }
    }
}

/// Plugin for the pickaxe viewmodel
pub struct PickaxePlugin;

impl Plugin for PickaxePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PickaxeState>()
            // Use PostStartup to ensure camera is spawned first
            .add_systems(PostStartup, spawn_pickaxe)
            .add_systems(Update, (
                trigger_swing_system,
                animate_pickaxe_system,
                idle_bob_system,
            ).chain());
    }
}
