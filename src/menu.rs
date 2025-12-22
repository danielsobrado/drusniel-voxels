use crate::chat::ChatState;
use crate::network::NetworkSession;
use crate::rendering::{capabilities::GraphicsCapabilities, ray_tracing::RayTracingSettings};
use crate::voxel::{meshing::ChunkMesh, persistence, world::VoxelWorld};
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
    window::{PrimaryWindow, WindowMode, WindowResolution},
    hierarchy::DespawnRecursiveExt,
};
use bevy::ui::{
    AlignItems, AlignSelf, FlexDirection, JustifyContent, PositionType,
    UiRect, Val,
};
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};

#[derive(Resource, Default, Clone)]
pub struct FavoriteServer {
    pub ip: String,
    pub port: String,
    pub password: String,
}

#[derive(Resource, Default)]
pub struct MultiplayerFormState {
    pub host_password: String,
    pub join_ip: String,
    pub join_port: String,
    pub join_password: String,
    pub favorites: Vec<FavoriteServer>,
    pub active_field: Option<MultiplayerField>,
}


#[derive(Resource, Default)]
pub struct PauseMenuState {
    pub open: bool,
    pub root_entity: Option<Entity>,
}

#[derive(Resource, Clone)]
pub struct SettingsState {
    pub dialog_root: Option<Entity>,
    pub active_tab: SettingsTab,
    pub graphics_quality: GraphicsQuality,
    pub anti_aliasing: AntiAliasing,
    pub ray_tracing: bool,
    pub display_mode: DisplayMode,
    pub resolution: UVec2,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            dialog_root: None,
            active_tab: SettingsTab::Graphics,
            graphics_quality: GraphicsQuality::Medium,
            anti_aliasing: AntiAliasing::Fxaa,
            ray_tracing: false,
            display_mode: DisplayMode::Bordered,
            resolution: UVec2::new(1920, 1080),
        }
    }
}

#[derive(Component, Copy, Clone)]
enum SettingsTabButton {
    Graphics,
    Gameplay,
}

#[derive(Component)]
struct SettingsDialogRoot;

#[derive(Component, Copy, Clone)]
struct GraphicsQualityOption(GraphicsQuality);

#[derive(Component, Copy, Clone)]
struct AntiAliasingOption(AntiAliasing);

#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct RayTracingOption(bool);

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum DisplayModeOption {
    Bordered,
    Borderless,
    Fullscreen,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct ResolutionOption(UVec2);

#[derive(Component)]
struct CloseSettingsButton;

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum SettingsTab {
    Graphics,
    Gameplay,
}

#[derive(Component)]
struct GraphicsTabContent;

#[derive(Component)]
struct GameplayTabContent;

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum GraphicsQuality {
    Low,
    Medium,
    High,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum AntiAliasing {
    None,
    Fxaa,
    Msaa4x,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum DisplayMode {
    Bordered,
    Borderless,
    Fullscreen,
}

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component, Copy, Clone)]
enum PauseMenuButton {
    Save,
    Load,
    Settings,
    StartServer,
    Connect,
    SaveFavorite,
    Resume,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum MultiplayerField {
    HostPassword,
    JoinIp,
    JoinPort,
    JoinPassword,
}

#[derive(Component, Copy, Clone)]
struct InputField {
    field: MultiplayerField,
}

#[derive(Component, Copy, Clone)]
struct InputText {
    field: MultiplayerField,
}

#[derive(Component)]
struct FavoritesList;

#[derive(Component, Copy, Clone)]
struct FavoriteButton(usize);

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseMenuState>()
            .init_resource::<SettingsState>()
            .init_resource::<MultiplayerFormState>()
            .init_resource::<ChatState>()
            .init_resource::<NetworkSession>()
            .add_systems(
                Update,
                (
                    toggle_pause_menu,
                    handle_menu_buttons,
                    handle_settings_buttons,
                    handle_input_interaction,
                    process_input_characters,
                    update_input_texts,
                    update_input_backgrounds,
                    update_settings_tab_backgrounds,
                    update_settings_content_visibility,
                    update_settings_graphics_backgrounds,
                    update_settings_aa_backgrounds,
                    update_settings_ray_tracing_backgrounds,
                    update_settings_display_mode_backgrounds,
                    update_settings_resolution_backgrounds,
                    handle_favorite_buttons,
                ),
            );
    }
}

fn toggle_pause_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<PauseMenuState>,
    mut form_state: ResMut<MultiplayerFormState>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if state.open {
        close_menu(
            &mut commands,
            &mut state,
            &mut form_state,
            &mut settings_state,
        );
    } else {
        open_menu(&mut commands, &asset_server, &mut state, &form_state);
    }
}

fn open_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    state: &mut PauseMenuState,
    form_state: &MultiplayerFormState,
) {
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
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(16.0),
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


                    menu.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        ..default()
                    })
                    .with_children(|row: &mut ChildBuilder| {
                        spawn_button(row, &font, "Save", PauseMenuButton::Save);
                        spawn_button(row, &font, "Load", PauseMenuButton::Load);
                        spawn_button(row, &font, "Settings", PauseMenuButton::Settings);
                    });

                    menu.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|section| {
                        section.spawn((
                            Text::new("Host Game"),
                            TextFont {
                                font: font.clone(),
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        spawn_labeled_input(
                            section,
                            &font,
                            "Session Password",
                            "Required for clients",
                            MultiplayerField::HostPassword,
                        );

                        spawn_button(section, &font, "Start Server", PauseMenuButton::StartServer);
                    });

                    menu.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn((
                            Text::new("Join Game"),
                            TextFont {
                                font: font.clone(),
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        spawn_labeled_input(
                            section,
                            &font,
                            "Host IP",
                            "Enter IPv4 or IPv6",
                            MultiplayerField::JoinIp,
                        );
                        spawn_labeled_input(
                            section,
                            &font,
                            "Port",
                            "e.g. 7777",
                            MultiplayerField::JoinPort,
                        );
                        spawn_labeled_input(
                            section,
                            &font,
                            "Password",
                            "Session password",
                            MultiplayerField::JoinPassword,
                        );

                        section
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_button(row, &font, "Connect", PauseMenuButton::Connect);
                                spawn_button(
                                    row,
                                    &font,
                                    "Save Favorite",
                                    PauseMenuButton::SaveFavorite,
                                );
                            });

                        section
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(6.0),
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.8)),
                                FavoritesList,
                            ))
                            .with_children(|favorites| {
                                favorites.spawn((
                                    Text::new("Favorite Servers"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));

                                for (index, favorite) in form_state.favorites.iter().enumerate() {
                                    spawn_favorite_button(favorites, &font, index, favorite);
                                }
                            });
                    });

                    spawn_button(menu, &font, "Resume", PauseMenuButton::Resume);

                });
        })
        .id();

    state.root_entity = Some(root);
    state.open = true;
}


fn spawn_labeled_input(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    placeholder: &str,
    field: MultiplayerField,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|column: &mut ChildBuilder| {
            column.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            column
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(320.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
                    InputField { field },
                ))
                .with_children(|input: &mut ChildBuilder| {
                    input.spawn((
                        Text::new(placeholder),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                        InputText { field },
                    ));
                });
        });
}

fn spawn_button(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    action: PauseMenuButton,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(160.0),
                padding: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
            action,
        ))
        .with_children(|button: &mut ChildBuilder| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}


fn spawn_settings_dialog(
    commands: &mut Commands,
    root_entity: Option<Entity>,
    font: &Handle<Font>,
    settings_state: SettingsState,
    ray_tracing_supported: bool,
) -> Entity {
    let mut dialog_entity = commands.spawn((
        Node {
            width: Val::Percent(70.0),
            height: Val::Percent(70.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.95)),
        SettingsDialogRoot,
    ));

    dialog_entity.with_children(|dialog| {
        dialog.spawn((
            Text::new("Settings"),
            TextFont {
                font: font.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));

        dialog
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|tabs| {
                spawn_settings_tab_button(tabs, font, "Graphics", SettingsTabButton::Graphics);
                spawn_settings_tab_button(tabs, font, "Gameplay", SettingsTabButton::Gameplay);
            });

        dialog
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.12, 0.12, 0.95)),
            ))
            .with_children(|content| {
                content
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        Visibility::from(if settings_state.active_tab == SettingsTab::Graphics {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        }),
                        GraphicsTabContent,
                    ))
                    .with_children(|graphics| {
                        graphics.spawn((
                            Text::new("Graphics Quality"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        graphics
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Low",
                                    GraphicsQualityOption(GraphicsQuality::Low),
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Medium",
                                    GraphicsQualityOption(GraphicsQuality::Medium),
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "High",
                                    GraphicsQualityOption(GraphicsQuality::High),
                                );
                            });

                        graphics.spawn((
                            Text::new("Anti-Aliasing"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        graphics
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "None",
                                    AntiAliasingOption(AntiAliasing::None),
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "FXAA",
                                    AntiAliasingOption(AntiAliasing::Fxaa),
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "MSAA 4x",
                                    AntiAliasingOption(AntiAliasing::Msaa4x),
                                );
                            });

                        graphics.spawn((
                            Text::new("Ray Tracing"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        if !ray_tracing_supported {
                            graphics.spawn((
                                Text::new("Ray tracing requires a compatible GPU."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.8, 0.4, 0.4, 1.0)),
                            ));
                        }

                        graphics
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Off", RayTracingOption(false));
                                spawn_graphics_option(row, font, "On", RayTracingOption(true));
                            });

                        graphics.spawn((
                            Text::new("Display Mode"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        graphics
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Bordered",
                                    DisplayModeOption::Bordered,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Borderless",
                                    DisplayModeOption::Borderless,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Fullscreen",
                                    DisplayModeOption::Fullscreen,
                                );
                            });

                        graphics.spawn((
                            Text::new("Resolution"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        graphics
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                row_gap: Val::Px(8.0),
                                flex_wrap: FlexWrap::Wrap,
                                ..default()
                            })
                            .with_children(|row| {
                                for (label, size) in [
                                    ("1280x720", UVec2::new(1280, 720)),
                                    ("1600x900", UVec2::new(1600, 900)),
                                    ("1920x1080", UVec2::new(1920, 1080)),
                                    ("2560x1440", UVec2::new(2560, 1440)),
                                ] {
                                    spawn_graphics_option(row, font, label, ResolutionOption(size));
                                }
                            });
                    });

                content
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        Visibility::from(if settings_state.active_tab == SettingsTab::Gameplay {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        }),
                        GameplayTabContent,
                    ))
                    .with_children(|gameplay| {
                        gameplay.spawn((
                            Text::new("Gameplay settings coming soon."),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });

        dialog
            .spawn((
                Button,
                Node {
                    width: Val::Px(120.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                CloseSettingsButton,
            ))
            .with_children(|button: &mut ChildBuilder| {
                button.spawn((
                    Text::new("Close"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    });

    let dialog_id = dialog_entity.id();
    if let Some(root) = root_entity {
        commands.entity(root).add_child(dialog_id);
    }

    dialog_id
}

fn spawn_settings_tab_button(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    tab: SettingsTabButton,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.18, 0.18, 0.18, 0.9)),
            tab,
        ))
        .with_children(|button: &mut ChildBuilder| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_graphics_option<T: Component + Copy + Send + Sync + 'static>(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    tag: T,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            tag,
        ))
        .with_children(|button: &mut ChildBuilder| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn close_menu(
    commands: &mut Commands,
    state: &mut PauseMenuState,
    form_state: &mut MultiplayerFormState,
    settings_state: &mut SettingsState,
) {
    if let Some(root) = state.root_entity.take() {
        commands.entity(root).despawn();
    }
    close_settings_dialog(commands, settings_state);
    form_state.active_field = None;
    state.open = false;
}

fn handle_menu_buttons(
    mut interaction_query: Query<
        (&Interaction, &PauseMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut world: ResMut<VoxelWorld>,
    chunk_meshes: Query<Entity, With<ChunkMesh>>,
    mut state: ResMut<PauseMenuState>,
    mut settings_state: ResMut<SettingsState>,
    mut form_state: ResMut<MultiplayerFormState>,
    mut network: ResMut<NetworkSession>,
    mut chat: ResMut<ChatState>,
    favorites_list: Query<Entity, With<FavoritesList>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    capabilities: Res<GraphicsCapabilities>,
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
            PauseMenuButton::StartServer => {
                info!(
                    "Starting server with password '{}'",
                    form_state.host_password
                );
                network.server_running = true;
                network.host_password = form_state.host_password.clone();
                network.reset_client();

                chat.push_system("Server started");
            }
            PauseMenuButton::Connect => {
                if form_state.join_ip.is_empty() || form_state.join_port.is_empty() {
                    warn!("Cannot connect: IP or port missing");
                    chat.push_system("Cannot connect: IP or port missing");
                    continue;
                }

                if network.server_running
                    && !network.host_password.is_empty()
                    && form_state.join_password != network.host_password
                {
                    warn!("Cannot connect: password mismatch");
                    chat.push_system("Connection rejected: incorrect password");
                    continue;
                }

                let port = match form_state.join_port.parse::<u16>() {
                    Ok(port) => port,
                    Err(err) => {
                        warn!("Cannot connect: invalid port - {}", err);
                        chat.push_system("Cannot connect: invalid port");
                        continue;
                    }
                };

                let address = format!("{}:{}", form_state.join_ip, port);
                let mut socket_addrs = match address.to_socket_addrs() {
                    Ok(addrs) => addrs,
                    Err(err) => {
                        warn!("Cannot connect: invalid address - {}", err);
                        chat.push_system("Cannot connect: invalid address");
                        continue;
                    }
                };

                let Some(target_addr) = socket_addrs.next() else {
                    warn!("Cannot connect: no resolved addresses for {}", address);
                    chat.push_system("Cannot connect: address could not be resolved");
                    continue;
                };

                let start = Instant::now();
                let ping_result =
                    std::net::TcpStream::connect_timeout(&target_addr, Duration::from_secs(3));
                if let Err(err) = ping_result {
                    warn!("Cannot connect: ping/health check failed - {}", err);
                    chat.push_system("Connection failed: host unreachable");
                    network.reset_client();
                    continue;
                }

                let latency_ms = start.elapsed().as_millis();
                network.client_connected = true;
                network.connection_ip = Some(form_state.join_ip.clone());
                network.connection_port = Some(form_state.join_port.clone());
                network.last_latency_ms = Some(latency_ms);
                network.last_health_ok = true;

                info!("Connected to {} (latency: {} ms)", address, latency_ms);
                chat.push_message(crate::chat::ChatMessage {
                    user: chat.username.clone(),
                    content: format!("Connected to {} ({} ms latency)", address, latency_ms),
                });
            }
            PauseMenuButton::Settings => {
                if settings_state.dialog_root.is_none() {
                    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
                    settings_state.active_tab = SettingsTab::Graphics;
                    settings_state.dialog_root = Some(spawn_settings_dialog(
                        &mut commands,
                        state.root_entity,
                        &font,
                        settings_state.clone(),
                        capabilities.ray_tracing_supported,
                    ));
                }
            }
            PauseMenuButton::SaveFavorite => {
                if form_state.join_ip.is_empty() || form_state.join_port.is_empty() {
                    warn!("Cannot save favorite: IP or port missing");
                    continue;
                }

                let duplicate = form_state
                    .favorites
                    .iter()
                    .any(|fav| fav.ip == form_state.join_ip && fav.port == form_state.join_port);
                if duplicate {
                    warn!("Favorite already exists for this address");
                    continue;
                }

                let new_favorite = FavoriteServer {
                    ip: form_state.join_ip.clone(),
                    port: form_state.join_port.clone(),
                    password: form_state.join_password.clone(),
                };
                let index = form_state.favorites.len();
                form_state.favorites.push(new_favorite.clone());

                if let Ok(container) = favorites_list.get_single() {
                    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
                    commands.entity(container).with_children(|parent| {
                        spawn_favorite_button(parent, &font, index, &new_favorite);
                    });
                }

                info!(
                    "Saved favorite server {}:{}",
                    new_favorite.ip, new_favorite.port
                );
            }
            PauseMenuButton::Resume => {
                close_menu(
                    &mut commands,
                    &mut state,
                    &mut form_state,
                    &mut settings_state,
                );
            }
        }

        if !matches!(action, PauseMenuButton::Resume) {
            state.open = true;
        }
    }
}

fn handle_settings_buttons(
    mut commands: Commands,
    state: Res<PauseMenuState>,
    mut settings_state: ResMut<SettingsState>,
    mut ray_tracing_settings: ResMut<RayTracingSettings>,
    capabilities: Res<GraphicsCapabilities>,
    mut tab_query: Query<(&Interaction, &SettingsTabButton), (Changed<Interaction>, With<Button>)>,
    mut quality_query: Query<
        (&Interaction, &GraphicsQualityOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut aa_query: Query<(&Interaction, &AntiAliasingOption), (Changed<Interaction>, With<Button>)>,
    mut ray_tracing_query: Query<
        (&Interaction, &RayTracingOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut display_query: Query<
        (&Interaction, &DisplayModeOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut resolution_query: Query<
        (&Interaction, &ResolutionOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut close_query: Query<&Interaction, (Changed<Interaction>, With<CloseSettingsButton>)>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !state.open || settings_state.dialog_root.is_none() {
        return;
    }

    for (interaction, tab) in tab_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.active_tab = match tab {
                SettingsTabButton::Graphics => SettingsTab::Graphics,
                SettingsTabButton::Gameplay => SettingsTab::Gameplay,
            };
        }
    }

    for (interaction, option) in quality_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.graphics_quality = option.0;
        }
    }

    for (interaction, option) in aa_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.anti_aliasing = option.0;
        }
    }

    for (interaction, option) in ray_tracing_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if option.0 && !capabilities.ray_tracing_supported {
                warn!("Ray tracing is not supported on this GPU");
                continue;
            }

            settings_state.ray_tracing = option.0;
            ray_tracing_settings.enabled = option.0;
        }
    }

    for (interaction, option) in display_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.display_mode = match option {
                DisplayModeOption::Bordered => DisplayMode::Bordered,
                DisplayModeOption::Borderless => DisplayMode::Borderless,
                DisplayModeOption::Fullscreen => DisplayMode::Fullscreen,
            };
            apply_window_settings(&settings_state, &mut windows);
        }
    }

    for (interaction, option) in resolution_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.resolution = option.0;
            apply_window_settings(&settings_state, &mut windows);
        }
    }

    for interaction in close_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            close_settings_dialog(&mut commands, &mut settings_state);
        }
    }
}

fn apply_window_settings(
    settings_state: &SettingsState,
    windows: &mut Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    window.mode = match settings_state.display_mode {
        DisplayMode::Bordered | DisplayMode::Borderless => WindowMode::Windowed,
        DisplayMode::Fullscreen => WindowMode::Fullscreen,
    };
    window.decorations = matches!(settings_state.display_mode, DisplayMode::Bordered);
    window.resolution = WindowResolution::new(
        settings_state.resolution.x as f32,
        settings_state.resolution.y as f32,
    );
}

fn close_settings_dialog(commands: &mut Commands, settings_state: &mut SettingsState) {
    if let Some(entity) = settings_state.dialog_root.take() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_settings_tab_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&SettingsTabButton, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (tab, mut background) in query.iter_mut() {
        let active = match tab {
            SettingsTabButton::Graphics => settings_state.active_tab == SettingsTab::Graphics,
            SettingsTabButton::Gameplay => settings_state.active_tab == SettingsTab::Gameplay,
        };

        *background = if active {
            Color::srgba(0.35, 0.35, 0.45, 0.95).into()
        } else {
            Color::srgba(0.18, 0.18, 0.18, 0.9).into()
        };
    }
}

fn update_settings_content_visibility(
    settings_state: Res<SettingsState>,
    mut graphics_query: Query<&mut Visibility, With<GraphicsTabContent>>,
    mut gameplay_query: Query<&mut Visibility, With<GameplayTabContent>>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    if let Ok(mut graphics_visibility) = graphics_query.get_single_mut() {
        *graphics_visibility = if settings_state.active_tab == SettingsTab::Graphics {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut gameplay_visibility) = gameplay_query.get_single_mut() {
        *gameplay_visibility = if settings_state.active_tab == SettingsTab::Gameplay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_settings_graphics_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&GraphicsQualityOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.graphics_quality == option.0;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_settings_aa_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&AntiAliasingOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.anti_aliasing == option.0;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_settings_ray_tracing_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&RayTracingOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.ray_tracing == option.0;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_settings_display_mode_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&DisplayModeOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = match option {
            DisplayModeOption::Bordered => settings_state.display_mode == DisplayMode::Bordered,
            DisplayModeOption::Borderless => settings_state.display_mode == DisplayMode::Borderless,
            DisplayModeOption::Fullscreen => settings_state.display_mode == DisplayMode::Fullscreen,
        };

        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_settings_resolution_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&ResolutionOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.resolution == option.0;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn handle_input_interaction(
    mut form_state: ResMut<MultiplayerFormState>,
    state: Res<PauseMenuState>,
    mut query: Query<(&Interaction, &InputField), (Changed<Interaction>, With<Button>)>,
) {
    if !state.open {
        return;
    }

    for (interaction, input) in query.iter_mut() {
        if *interaction == Interaction::Pressed {
            form_state.active_field = Some(input.field);
        }
    }
}

fn process_input_characters(
    mut form_state: ResMut<MultiplayerFormState>,
    state: Res<PauseMenuState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut char_evr: EventReader<KeyboardInput>,
) {
    if !state.open {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        form_state.active_field = None;
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        if let Some(field) = form_state.active_field {
            let target = get_field_value_mut(&mut form_state, field);
            target.pop();
        }
    }

    for ev in char_evr.read() {
        if !ev.state.is_pressed() {
            continue;
        }

        if let Key::Character(ch_str) = &ev.logical_key {
            let ch = ch_str.chars().next().unwrap_or(' ');
            if let Some(field) = form_state.active_field {
                let target = get_field_value_mut(&mut form_state, field);

                match field {
                    MultiplayerField::JoinPort => {
                        if ch.is_ascii_digit() {
                            target.push(ch);
                        }
                    }
                    _ => {
                        target.push(ch);
                    }
                }
            }
        }
    }
}

fn get_field_value_mut<'a>(
    form_state: &'a mut MultiplayerFormState,
    field: MultiplayerField,
) -> &'a mut String {
    match field {
        MultiplayerField::HostPassword => &mut form_state.host_password,
        MultiplayerField::JoinIp => &mut form_state.join_ip,
        MultiplayerField::JoinPort => &mut form_state.join_port,
        MultiplayerField::JoinPassword => &mut form_state.join_password,
    }
}

fn update_input_texts(
    form_state: Res<MultiplayerFormState>,
    state: Res<PauseMenuState>,
    mut query: Query<(&InputText, &mut Text)>,
) {
    if !state.open {
        return;
    }

    for (field, mut text) in query.iter_mut() {
        let value = match field.field {
            MultiplayerField::HostPassword => &form_state.host_password,
            MultiplayerField::JoinIp => &form_state.join_ip,
            MultiplayerField::JoinPort => &form_state.join_port,
            MultiplayerField::JoinPassword => &form_state.join_password,
        };

        let display_value = if value.is_empty() {
            match field.field {
                MultiplayerField::HostPassword => "Required for clients",
                MultiplayerField::JoinIp => "Enter IPv4 or IPv6",
                MultiplayerField::JoinPort => "e.g. 7777",
                MultiplayerField::JoinPassword => "Session password",
            }
        } else {
            value
        };

        text.0 = display_value.to_string();
    }
}

fn update_input_backgrounds(
    form_state: Res<MultiplayerFormState>,
    state: Res<PauseMenuState>,
    mut query: Query<(&InputField, &mut BackgroundColor)>,
) {
    if !state.open {
        return;
    }

    for (field, mut background) in query.iter_mut() {
        let is_active = form_state.active_field == Some(field.field);
        *background = if is_active {
            Color::srgba(0.3, 0.35, 0.45, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.95).into()
        };
    }
}

fn spawn_favorite_button(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    index: usize,
    favorite: &FavoriteServer,
) {
    let label = format!("{}:{}", favorite.ip, favorite.port);
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.18, 0.18, 0.18, 0.9)),
            FavoriteButton(index),
        ))
        .with_children(|button: &mut ChildBuilder| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn handle_favorite_buttons(
    mut form_state: ResMut<MultiplayerFormState>,
    state: Res<PauseMenuState>,
    mut query: Query<(&Interaction, &FavoriteButton), (Changed<Interaction>, With<Button>)>,
) {
    if !state.open {
        return;
    }

    for (interaction, favorite) in query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(entry) = form_state.favorites.get(favorite.0).cloned() {
            form_state.join_ip = entry.ip;
            form_state.join_port = entry.port;
            form_state.join_password = entry.password;
            form_state.active_field = None;
            info!("Loaded favorite server {}:{}", form_state.join_ip, form_state.join_port);
        }
    }
}
