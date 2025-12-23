use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Component)]
pub struct ParticlePlugin;

#[derive(Resource)]
pub struct ParticleRegistry {
    pub dig_effect: Handle<EffectAsset>,
}

#[derive(Message, Debug, Clone)]
pub struct SpawnParticleEvent {
    pub position: Vec3,
    pub particle_type: ParticleType,
}

#[derive(Debug, Clone)]
pub enum ParticleType {
    Dig,
}

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<HanabiPlugin>() {
            app.add_plugins(HanabiPlugin);
        }

        app.add_event::<SpawnParticleEvent>()
           .add_systems(Startup, setup_particles)
           .add_systems(Update, (handle_particle_events, despawn_finished_effects));
    }
}

fn setup_particles(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // TODO: Restore particle effect definition once Spawner type is identified
    // For now, we just create a placeholder handle to prevent crashes
    
    // Placeholder effect
    let effect = EffectAsset::default(); 
    let handle = effects.add(effect);
    
    commands.insert_resource(ParticleRegistry { dig_effect: handle });
}

fn handle_particle_events(
    mut commands: Commands,
    mut events: EventReader<SpawnParticleEvent>, 
    registry: Res<ParticleRegistry>,
) {
    for ev in events.read() {
        match ev.particle_type {
            ParticleType::Dig => {
                //  commands.spawn((
                //     ParticleEffect {
                //         handle: registry.dig_effect.clone(),
                //         ..default()
                //     },
                //     EffectProperties::default(),
                //     Transform::from_translation(ev.position),
                //     GlobalTransform::default(),
                //     Visibility::Visible,
                //     InheritedVisibility::default(),
                //     ViewVisibility::default(),
                //     AutoDespawnEffect,
                //  ));
            }
        }
    }
}

#[derive(Component)]
struct AutoDespawnEffect;

fn despawn_finished_effects(
    mut commands: Commands,
    query: Query<(Entity, &CompiledParticleEffect), With<AutoDespawnEffect>>,
) {
    for (entity, effect) in query.iter() {
        // if effect.is_finished() {
        //     commands.entity(entity).despawn();
        // }
    }
}
