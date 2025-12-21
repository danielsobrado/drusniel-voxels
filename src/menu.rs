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
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|menu| {
                    menu.spawn((
                        Text::new("Paused"),
                        TextFont {
                            font: font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Save Button
                    menu.spawn((
                        Button,
                        Node {
                            width: Val::Px(160.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                        PauseMenuButton::Save,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Save"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Load Button
                    menu.spawn((
                        Button,
                        Node {
                            width: Val::Px(160.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                        PauseMenuButton::Load,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Load"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Resume Button
                    menu.spawn((
                        Button,
                        Node {
                            width: Val::Px(160.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                        PauseMenuButton::Resume,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Resume"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        })
        .id();

    state.root_entity = Some(root);
    state.open = true;
}



fn close_menu(commands: &mut Commands, state: &mut PauseMenuState) {
    if let Some(root) = state.root_entity.take() {
        commands.entity(root).despawn();
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
                    commands.entity(entity).despawn();
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
