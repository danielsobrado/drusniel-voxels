use crate::voxel::{meshing::ChunkMesh, persistence, world::VoxelWorld};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PauseMenuState {
    pub open: bool,
    pub root_entity: Option<Entity>,
}

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component, Copy, Clone)]
enum PauseMenuButton {
    Save,
    Load,
    Resume,
}

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseMenuState>()
            .add_systems(Update, (toggle_pause_menu, handle_menu_buttons));
    }
}

fn toggle_pause_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<PauseMenuState>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if state.open {
        close_menu(&mut commands, &mut state);
    } else {
        open_menu(&mut commands, &asset_server, &mut state);
    }
}

fn open_menu(commands: &mut Commands, asset_server: &Res<AssetServer>, state: &mut PauseMenuState) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
                ..default()
            },
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
                    ..default()
                })
                .with_children(|menu| {
                    menu.spawn(TextBundle {
                        text: Text::from_section(
                            "Paused",
                            TextStyle {
                                font: font.clone(),
                                font_size: 30.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    });

                    spawn_button(menu, &font, "Save", PauseMenuButton::Save);
                    spawn_button(menu, &font, "Load", PauseMenuButton::Load);
                    spawn_button(menu, &font, "Resume", PauseMenuButton::Resume);
                });
        })
        .id();

    state.root_entity = Some(root);
    state.open = true;
}

fn spawn_button(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    action: PauseMenuButton,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(160.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.25, 0.25, 0.25, 0.9).into(),
                ..default()
            },
            action,
        ))
        .with_children(|button| {
            button.spawn(TextBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });
        });
}

fn close_menu(commands: &mut Commands, state: &mut PauseMenuState) {
    if let Some(root) = state.root_entity.take() {
        commands.entity(root).despawn_recursive();
    }
    state.open = false;
}

fn handle_menu_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &PauseMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut world: ResMut<VoxelWorld>,
    chunk_meshes: Query<Entity, With<ChunkMesh>>,
    mut state: ResMut<PauseMenuState>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            PauseMenuButton::Save => match persistence::save_world(&world) {
                Ok(()) => info!("World saved via pause menu"),
                Err(err) => warn!("Failed to save world: {}", err),
            },
            PauseMenuButton::Load => {
                for entity in chunk_meshes.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                match persistence::load_world() {
                    Ok(loaded_world) => {
                        *world = loaded_world;
                        info!("World loaded from disk via pause menu");
                    }
                    Err(err) => warn!("Failed to load world: {}", err),
                }
            }
            PauseMenuButton::Resume => {
                close_menu(&mut commands, &mut state);
            }
        }

        if !matches!(action, PauseMenuButton::Resume) {
            state.open = true;
        }
    }
}
