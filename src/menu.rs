use crate::chat::ChatState;
use crate::environment::AtmosphereSettings;
use crate::network::NetworkSession;
use crate::rendering::{capabilities::GraphicsCapabilities, ray_tracing::RayTracingSettings};
use crate::voxel::{meshing::ChunkMesh, persistence, world::VoxelWorld};
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
    window::{PrimaryWindow, WindowMode, WindowResolution, MonitorSelection, VideoModeSelection},
};
// use bevy::prelude::ChildBuilder;
use bevy::ui::{
    AlignItems, AlignSelf, FlexDirection, JustifyContent, 
    UiRect, Val,
};

use std::net::ToSocketAddrs;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
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
struct ConnectTaskState {
    receiver: Option<Arc<Mutex<Receiver<ConnectOutcome>>>>,
}

enum ConnectOutcome {
    Success {
        ip: String,
        port: String,
        address: String,
        latency_ms: u128,
    },
    Failure {
        message: String,
    },
}


#[derive(Resource)]
pub struct PauseMenuState {
    pub open: bool,
    pub root_entity: Option<Entity>,
    pub current_screen: MenuScreen,
}

impl Default for PauseMenuState {
    fn default() -> Self {
        Self {
            open: false,
            root_entity: None,
            current_screen: MenuScreen::Main,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuScreen {
    Main,
    Multiplayer,
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
    pub day_length: DayLengthOption,
    pub time_scale: TimeScaleOption,
    pub rayleigh: RayleighOption,
    pub mie: MieOption,
    pub mie_direction: MieDirectionOption,
    pub exposure: ExposureOption,
    pub twilight_band: TwilightBandOption,
    pub night_brightness: NightBrightnessOption,
    pub fog_preset: FogPresetOption,
    pub cycle_enabled: bool,
    pub shadow_filtering: ShadowFiltering,
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
            day_length: DayLengthOption::Standard,
            time_scale: TimeScaleOption::RealTime,
            rayleigh: RayleighOption::Balanced,
            mie: MieOption::Standard,
            mie_direction: MieDirectionOption::Standard,
            exposure: ExposureOption::Neutral,
            twilight_band: TwilightBandOption::Medium,
            night_brightness: NightBrightnessOption::Balanced,
            fog_preset: FogPresetOption::Balanced,
            cycle_enabled: true,
            shadow_filtering: ShadowFiltering::Gaussian,
        }
    }
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum ShadowFiltering {
    Gaussian,
    Hardware2x2,
    Temporal,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct ShadowFilteringOption(pub ShadowFiltering);

#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct DayNightCycleOption(pub bool);

#[derive(Component, Copy, Clone)]
enum SettingsTabButton {
    Graphics,
    Gameplay,
    Atmosphere,
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
pub enum SettingsTab {
    Graphics,
    Gameplay,
    Atmosphere,
}

#[derive(Component)]
struct GraphicsTabContent;

#[derive(Component)]
struct GameplayTabContent;

#[derive(Component)]
struct AtmosphereTabContent;

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum GraphicsQuality {
    Low,
    Medium,
    High,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum AntiAliasing {
    None,
    Fxaa,
    Msaa4x,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum DisplayMode {
    Bordered,
    Borderless,
    Fullscreen,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum DayLengthOption {
    Short,
    Standard,
    Long,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum TimeScaleOption {
    Slow,
    RealTime,
    Fast,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum RayleighOption {
    Gentle,
    Balanced,
    Vivid,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum MieOption {
    Soft,
    Standard,
    Dense,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum MieDirectionOption {
    Broad,
    Standard,
    Forward,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum ExposureOption {
    Low,
    Neutral,
    High,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum TwilightBandOption {
    Narrow,
    Medium,
    Wide,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum NightBrightnessOption {
    Dim,
    Balanced,
    Bright,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum FogPresetOption {
    Clear,
    Balanced,
    Misty,
}

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component, Copy, Clone)]
enum PauseMenuButton {
    Save,
    Load,
    Settings,
    Multiplayer,
    StartServer,
    Connect,
    SaveFavorite,
    BackToMain,
    Resume,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub enum MultiplayerField {
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
            .init_resource::<ConnectTaskState>()
            .init_resource::<ChatState>()
            .init_resource::<NetworkSession>()
            .add_systems(
                Update,
                (
                    toggle_pause_menu,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_menu_buttons,
                ),
            )
            .add_systems(
                Update,
                (
                    poll_connect_task_results,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_settings_tabs,
                    handle_graphics_settings,
                    handle_atmosphere_settings,
                    handle_close_settings,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_input_interaction,
                ),
            )
            .add_systems(
                Update,
                (
                    process_input_characters,
                ),
            )

            .add_systems(
                Update,
                (
                    update_input_texts,
                    update_input_backgrounds,
                ),
            )
            .add_systems(
                Update,
                (
                    update_settings_tab_backgrounds,
                    update_settings_content_visibility,
                    update_settings_graphics_backgrounds,
                    update_settings_aa_backgrounds,
                ),
            )
            .add_systems(
                Update,
                (
                    update_settings_ray_tracing_backgrounds,
                    update_settings_display_mode_backgrounds,
                    update_settings_resolution_backgrounds,
                    update_settings_shadow_filtering_backgrounds,
                    update_day_length_backgrounds,
                ),
            )
            .add_systems(
                Update,
                (
                    update_time_scale_backgrounds,
                    update_rayleigh_backgrounds,
                    update_mie_backgrounds,
                    update_mie_direction_backgrounds,
                ),
            )
            .add_systems(
                Update,
                (
                    update_exposure_backgrounds,
                    update_twilight_backgrounds,
                    update_night_backgrounds,
                    update_fog_backgrounds,
                    update_cycle_backgrounds,
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
            match state.current_screen {
                MenuScreen::Main => {
                    spawn_main_menu(parent, &font);
                }
                MenuScreen::Multiplayer => {
                    spawn_multiplayer_menu(parent, &font, form_state);
                }
            }
        })
        .id();

    state.root_entity = Some(root);
    state.open = true;
}

fn spawn_main_menu(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        ))
        .with_children(|menu| {
            // Title
            menu.spawn((
                Text::new("Game Menu"),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Main menu buttons
            spawn_button(menu, font, "Load", PauseMenuButton::Load);
            spawn_button(menu, font, "Save", PauseMenuButton::Save);
            spawn_button(menu, font, "Multiplayer", PauseMenuButton::Multiplayer);
            spawn_button(menu, font, "Settings", PauseMenuButton::Settings);
            spawn_button(menu, font, "Resume", PauseMenuButton::Resume);
        });
}

fn spawn_multiplayer_menu(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    form_state: &MultiplayerFormState,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(30.0)),
                max_width: Val::Px(500.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        ))
        .with_children(|menu| {
            // Title
            menu.spawn((
                Text::new("Multiplayer"),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Host Game Section
            menu.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
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
                    font,
                    "Session Password",
                    "Required for clients",
                    MultiplayerField::HostPassword,
                );

                spawn_button(section, font, "Start Server", PauseMenuButton::StartServer);
            });

            // Join Game Section
            menu.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
            ))
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
                    font,
                    "Host IP",
                    "Enter IPv4 or IPv6",
                    MultiplayerField::JoinIp,
                );
                spawn_labeled_input(
                    section,
                    font,
                    "Port",
                    "e.g. 7777",
                    MultiplayerField::JoinPort,
                );
                spawn_labeled_input(
                    section,
                    font,
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
                        spawn_button(row, font, "Connect", PauseMenuButton::Connect);
                        spawn_button(
                            row,
                            font,
                            "Save Favorite",
                            PauseMenuButton::SaveFavorite,
                        );
                    });

                // Favorites List
                section
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            padding: UiRect::all(Val::Px(8.0)),
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
                            spawn_favorite_button(favorites, font, index, favorite);
                        }
                    });
            });

            // Back button
            spawn_button(menu, font, "Back", PauseMenuButton::BackToMain);
        });
}


fn spawn_labeled_input(
    parent: &mut ChildSpawnerCommands,
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
        .with_children(|column: &mut ChildSpawnerCommands| {
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
                .with_children(|input| {
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
    parent: &mut ChildSpawnerCommands,
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
        .with_children(|button| {
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
                spawn_settings_tab_button(tabs, font, "Atmosphere", SettingsTabButton::Atmosphere);
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
                            Text::new("Shadow Filtering"),
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
                                    "Gaussian",
                                    ShadowFilteringOption(ShadowFiltering::Gaussian),
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Hard 2x2",
                                    ShadowFilteringOption(ShadowFiltering::Hardware2x2),
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Temporal",
                                    ShadowFilteringOption(ShadowFiltering::Temporal),
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

                content
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        Visibility::from(if settings_state.active_tab == SettingsTab::Atmosphere {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        }),
                        AtmosphereTabContent,
                    ))
                    .with_children(|atmosphere| {
                        atmosphere.spawn((
                            Text::new("Day/Night Cycle"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Enabled", DayNightCycleOption(true));
                                spawn_graphics_option(row, font, "Disabled", DayNightCycleOption(false));
                            });
                        atmosphere.spawn((
                            Text::new("Day Length"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "10 min", DayLengthOption::Short);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "30 min",
                                    DayLengthOption::Standard,
                                );
                                spawn_graphics_option(row, font, "60 min", DayLengthOption::Long);
                            });

                        atmosphere.spawn((
                            Text::new("Time Scale"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "0.5x time", TimeScaleOption::Slow);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "1x time",
                                    TimeScaleOption::RealTime,
                                );
                                spawn_graphics_option(row, font, "2x time", TimeScaleOption::Fast);
                            });

                        atmosphere.spawn((
                            Text::new("Rayleigh (Sky)"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Gentle", RayleighOption::Gentle);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Balanced",
                                    RayleighOption::Balanced,
                                );
                                spawn_graphics_option(row, font, "Vivid", RayleighOption::Vivid);
                            });

                        atmosphere.spawn((
                            Text::new("Mie (Haze)"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Soft haze", MieOption::Soft);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Standard",
                                    MieOption::Standard,
                                );
                                spawn_graphics_option(row, font, "Dense glow", MieOption::Dense);
                            });

                        atmosphere.spawn((
                            Text::new("Mie Direction"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Broad", MieDirectionOption::Broad);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Standard",
                                    MieDirectionOption::Standard,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Forward",
                                    MieDirectionOption::Forward,
                                );
                            });

                        atmosphere.spawn((
                            Text::new("Exposure"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Low", ExposureOption::Low);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Neutral",
                                    ExposureOption::Neutral,
                                );
                                spawn_graphics_option(row, font, "Bright", ExposureOption::High);
                            });

                        atmosphere.spawn((
                            Text::new("Twilight Band"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Narrow",
                                    TwilightBandOption::Narrow,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Medium",
                                    TwilightBandOption::Medium,
                                );
                                spawn_graphics_option(row, font, "Wide", TwilightBandOption::Wide);
                            });

                        atmosphere.spawn((
                            Text::new("Night Brightness"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Dim", NightBrightnessOption::Dim);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Balanced",
                                    NightBrightnessOption::Balanced,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Bright",
                                    NightBrightnessOption::Bright,
                                );
                            });

                        atmosphere.spawn((
                            Text::new("Fog Density"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        atmosphere
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Clear", FogPresetOption::Clear);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Balanced",
                                    FogPresetOption::Balanced,
                                );
                                spawn_graphics_option(row, font, "Misty", FogPresetOption::Misty);
                            });
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
            .with_children(|button| {
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
    parent: &mut ChildSpawnerCommands,
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
        .with_children(|button| {
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
    parent: &mut ChildSpawnerCommands,
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
        .with_children(|button: &mut ChildSpawnerCommands| {
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
    state.current_screen = MenuScreen::Main;
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
    mut connect_tasks: ResMut<ConnectTaskState>,
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
                if connect_tasks.receiver.is_some() {
                    warn!("Connection attempt already in progress");
                    chat.push_system("Connection already in progress");
                    continue;
                }

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
                let join_ip = form_state.join_ip.clone();
                let join_port = form_state.join_port.clone();
                let (tx, rx) = mpsc::channel();

                connect_tasks.receiver = Some(Arc::new(Mutex::new(rx)));
                chat.push_system(format!("Connecting to {}...", address));

                thread::spawn(move || {
                    let result = (|| -> Result<ConnectOutcome, String> {
                        let mut socket_addrs = address
                            .to_socket_addrs()
                            .map_err(|err| format!("Cannot connect: invalid address - {}", err))?;

                        let Some(target_addr) = socket_addrs.next() else {
                            return Err(format!(
                                "Cannot connect: no resolved addresses for {}",
                                address
                            ));
                        };

                        let start = Instant::now();
                        std::net::TcpStream::connect_timeout(&target_addr, Duration::from_secs(3))
                            .map_err(|err| {
                                format!("Cannot connect: ping/health check failed - {}", err)
                            })?;

                        let latency_ms = start.elapsed().as_millis();

                        Ok(ConnectOutcome::Success {
                            ip: join_ip,
                            port: join_port,
                            address,
                            latency_ms,
                        })
                    })();

                    let outcome = match result {
                        Ok(success) => success,
                        Err(message) => ConnectOutcome::Failure { message },
                    };

                    let _ = tx.send(outcome);
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
            PauseMenuButton::Multiplayer => {
                // Switch to multiplayer screen
                state.current_screen = MenuScreen::Multiplayer;
                // Close and reopen menu to show multiplayer screen
                if let Some(root) = state.root_entity {
                    commands.entity(root).despawn();
                }
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
                        spawn_multiplayer_menu(parent, &font, &form_state);
                    })
                    .id();
                state.root_entity = Some(root);
            }
            PauseMenuButton::BackToMain => {
                // Switch back to main menu
                state.current_screen = MenuScreen::Main;
                // Close and reopen menu to show main screen
                if let Some(root) = state.root_entity {
                    commands.entity(root).despawn();
                }
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
                        spawn_main_menu(parent, &font);
                    })
                    .id();
                state.root_entity = Some(root);
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

                if let Ok(container) = favorites_list.single() {
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

fn poll_connect_task_results(
    mut connect_tasks: ResMut<ConnectTaskState>,
    mut network: ResMut<NetworkSession>,
    mut chat: ResMut<ChatState>,
) {
    let Some(receiver) = connect_tasks.receiver.as_ref() else {
        return;
    };

    let result = receiver
        .lock()
        .map(|receiver| receiver.try_recv())
        .unwrap_or_else(|err| {
            warn!("Failed to check connection result: {}", err);
            Err(TryRecvError::Disconnected)
        });

    match result {
        Ok(ConnectOutcome::Success {
            ip,
            port,
            address,
            latency_ms,
        }) => {
            network.client_connected = true;
            network.connection_ip = Some(ip);
            network.connection_port = Some(port);
            network.last_latency_ms = Some(latency_ms);
            network.last_health_ok = true;

            info!("Connected to {} (latency: {} ms)", address, latency_ms);
            let username = chat.username.clone();
            chat.push_message(crate::chat::ChatMessage {
                user: username,
                content: format!("Connected to {} ({} ms latency)", address, latency_ms),
            });
            connect_tasks.receiver = None;
        }
        Ok(ConnectOutcome::Failure { message }) => {
            warn!("{}", message);
            chat.push_system(message);
            network.reset_client();
            connect_tasks.receiver = None;
        }
        Err(TryRecvError::Disconnected) => {
            warn!("Connection attempt ended unexpectedly");
            chat.push_system("Connection failed: internal error");
            network.reset_client();
            connect_tasks.receiver = None;
        }
        Err(TryRecvError::Empty) => {
            // Still waiting
        }
    }
}

fn handle_settings_tabs(
    state: Res<PauseMenuState>,
    mut settings_state: ResMut<SettingsState>,
    mut tab_query: Query<(&Interaction, &SettingsTabButton), (Changed<Interaction>, With<Button>)>,
) {
    if !state.open || settings_state.dialog_root.is_none() {
        return;
    }

    for (interaction, tab) in tab_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.active_tab = match tab {
                SettingsTabButton::Graphics => SettingsTab::Graphics,
                SettingsTabButton::Gameplay => SettingsTab::Gameplay,
                SettingsTabButton::Atmosphere => SettingsTab::Atmosphere,
            };
        }
    }
}

fn handle_graphics_settings(
    mut _commands: Commands,
    state: Res<PauseMenuState>,
    mut settings_state: ResMut<SettingsState>,
    mut ray_tracing_settings: ResMut<RayTracingSettings>,
    capabilities: Res<GraphicsCapabilities>,
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
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut shadow_filtering_query: Query<
        (&Interaction, &ShadowFilteringOption),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !state.open || settings_state.dialog_root.is_none() {
        return;
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

    for (interaction, option) in shadow_filtering_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.shadow_filtering = option.0;
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
}

fn handle_atmosphere_settings(
    state: Res<PauseMenuState>,
    mut settings_state: ResMut<SettingsState>,
    mut day_length_query: Query<
        (&Interaction, &DayLengthOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut time_scale_query: Query<
        (&Interaction, &TimeScaleOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut rayleigh_query: Query<
        (&Interaction, &RayleighOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut mie_query: Query<(&Interaction, &MieOption), (Changed<Interaction>, With<Button>)>,
    mut mie_direction_query: Query<
        (&Interaction, &MieDirectionOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut exposure_query: Query<
        (&Interaction, &ExposureOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut twilight_query: Query<
        (&Interaction, &TwilightBandOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut night_query: Query<
        (&Interaction, &NightBrightnessOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut fog_query: Query<(&Interaction, &FogPresetOption), (Changed<Interaction>, With<Button>)>,
    mut cycle_query: Query<
        (&Interaction, &DayNightCycleOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut atmosphere_settings: ResMut<AtmosphereSettings>,
    mut fog_config: ResMut<crate::atmosphere::FogConfig>,
) {
    if !state.open || settings_state.dialog_root.is_none() {
        return;
    }

    for (interaction, option) in cycle_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.cycle_enabled = option.0;
            atmosphere_settings.cycle_enabled = option.0;
        }
    }

    for (interaction, option) in day_length_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.day_length = *option;
            atmosphere_settings.day_length = match option {
                DayLengthOption::Short => 600.0,
                DayLengthOption::Standard => 1800.0,
                DayLengthOption::Long => 3600.0,
            };
        }
    }

    for (interaction, option) in time_scale_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.time_scale = *option;
            atmosphere_settings.time_scale = match option {
                TimeScaleOption::Slow => 0.5,
                TimeScaleOption::RealTime => 1.0,
                TimeScaleOption::Fast => 2.0,
            };
        }
    }

    let base_rayleigh = Vec3::new(5.5, 13.0, 22.4) * 0.0012;
    let base_mie = Vec3::splat(0.005);

    for (interaction, option) in rayleigh_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.rayleigh = *option;
            atmosphere_settings.rayleigh = match option {
                RayleighOption::Gentle => base_rayleigh * 0.7,
                RayleighOption::Balanced => base_rayleigh,
                RayleighOption::Vivid => base_rayleigh * 1.4,
            };
        }
    }

    for (interaction, option) in mie_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.mie = *option;
            atmosphere_settings.mie = match option {
                MieOption::Soft => Vec3::splat(0.0035),
                MieOption::Standard => base_mie,
                MieOption::Dense => Vec3::splat(0.0075),
            };
        }
    }

    for (interaction, option) in mie_direction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.mie_direction = *option;
            atmosphere_settings.mie_direction = match option {
                MieDirectionOption::Broad => 0.5,
                MieDirectionOption::Standard => 0.7,
                MieDirectionOption::Forward => 0.85,
            };
        }
    }

    for (interaction, option) in exposure_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.exposure = *option;
            atmosphere_settings.exposure = match option {
                ExposureOption::Low => 0.9,
                ExposureOption::Neutral => 1.2,
                ExposureOption::High => 1.6,
            };
        }
    }

    for (interaction, option) in twilight_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.twilight_band = *option;
            atmosphere_settings.twilight_band = match option {
                TwilightBandOption::Narrow => 0.35,
                TwilightBandOption::Medium => 0.6,
                TwilightBandOption::Wide => 0.9,
            };
        }
    }

    for (interaction, option) in night_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.night_brightness = *option;
            atmosphere_settings.night_floor = match option {
                NightBrightnessOption::Dim => 0.04,
                NightBrightnessOption::Balanced => 0.08,
                NightBrightnessOption::Bright => 0.12,
            };
        }
    }

    for (interaction, option) in fog_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.fog_preset = *option;
            atmosphere_settings.fog_density = match option {
                FogPresetOption::Clear => Vec2::new(0.0006, 0.0014),
                FogPresetOption::Balanced => Vec2::new(0.0009, 0.0022),
                FogPresetOption::Misty => Vec2::new(0.0012, 0.003),
            };
            // Also update volumetric fog density
            fog_config.volume.density = match option {
                FogPresetOption::Clear => 0.015,
                FogPresetOption::Balanced => 0.04,
                FogPresetOption::Misty => 0.08,
            };
        }
    }
}

fn handle_close_settings(
    mut commands: Commands,
    state: Res<PauseMenuState>,
    mut settings_state: ResMut<SettingsState>,
    mut close_query: Query<&Interaction, (Changed<Interaction>, With<CloseSettingsButton>)>,
) {
    if !state.open || settings_state.dialog_root.is_none() {
        return;
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
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    window.mode = match settings_state.display_mode {
        DisplayMode::Bordered | DisplayMode::Borderless => WindowMode::Windowed,
        DisplayMode::Fullscreen => WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current),
    };
    window.decorations = matches!(settings_state.display_mode, DisplayMode::Bordered);
    window.resolution = WindowResolution::new(
        settings_state.resolution.x as u32,
        settings_state.resolution.y as u32,
    );
}

fn close_settings_dialog(commands: &mut Commands, settings_state: &mut SettingsState) {
    if let Some(entity) = settings_state.dialog_root.take() {
        commands.entity(entity).despawn();
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
            SettingsTabButton::Atmosphere => settings_state.active_tab == SettingsTab::Atmosphere,
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
    mut visibility_queries: ParamSet<(
        Query<&mut Visibility, With<GraphicsTabContent>>,
        Query<&mut Visibility, With<GameplayTabContent>>,
        Query<&mut Visibility, With<AtmosphereTabContent>>,
    )>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    if let Ok(mut graphics_visibility) = visibility_queries.p0().single_mut() {
        *graphics_visibility = if settings_state.active_tab == SettingsTab::Graphics {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut gameplay_visibility) = visibility_queries.p1().single_mut() {
        *gameplay_visibility = if settings_state.active_tab == SettingsTab::Gameplay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut atmosphere_visibility) = visibility_queries.p2().single_mut() {
        *atmosphere_visibility = if settings_state.active_tab == SettingsTab::Atmosphere {
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


fn update_settings_shadow_filtering_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&ShadowFilteringOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.shadow_filtering == option.0;
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

fn update_cycle_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&DayNightCycleOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.cycle_enabled == option.0;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_day_length_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&DayLengthOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.day_length == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_time_scale_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&TimeScaleOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.time_scale == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_rayleigh_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&RayleighOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.rayleigh == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_mie_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&MieOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.mie == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_mie_direction_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&MieDirectionOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.mie_direction == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_exposure_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&ExposureOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.exposure == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_twilight_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&TwilightBandOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.twilight_band == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_night_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&NightBrightnessOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.night_brightness == *option;
        *background = if active {
            Color::srgba(0.32, 0.42, 0.35, 0.95).into()
        } else {
            Color::srgba(0.2, 0.2, 0.2, 0.9).into()
        };
    }
}

fn update_fog_backgrounds(
    settings_state: Res<SettingsState>,
    mut query: Query<(&FogPresetOption, &mut BackgroundColor)>,
) {
    if settings_state.dialog_root.is_none() {
        return;
    }

    for (option, mut background) in query.iter_mut() {
        let active = settings_state.fog_preset == *option;
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
    mut char_evr: MessageReader<KeyboardInput>,
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
    parent: &mut ChildSpawnerCommands,
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
        .with_children(|button: &mut ChildSpawnerCommands| {
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
