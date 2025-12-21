use crate::chat::ChatState;
use crate::environment::AtmosphereSettings;
use crate::network::NetworkSession;
use crate::voxel::{meshing::ChunkMesh, persistence, world::VoxelWorld};
use bevy::{
    input::keyboard::ReceivedCharacter,
    prelude::*,
    window::{PrimaryWindow, WindowMode, WindowResolution},
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
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            dialog_root: None,
            active_tab: SettingsTab::Graphics,
            graphics_quality: GraphicsQuality::Medium,
            anti_aliasing: AntiAliasing::Fxaa,
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
        }
    }
}

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
    Atmosphere,
}

#[derive(Component)]
struct GraphicsTabContent;

#[derive(Component)]
struct GameplayTabContent;

#[derive(Component)]
struct AtmosphereTabContent;

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

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum DayLengthOption {
    Short,
    Standard,
    Long,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum TimeScaleOption {
    Slow,
    RealTime,
    Fast,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum RayleighOption {
    Gentle,
    Balanced,
    Vivid,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum MieOption {
    Soft,
    Standard,
    Dense,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum MieDirectionOption {
    Broad,
    Standard,
    Forward,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum ExposureOption {
    Low,
    Neutral,
    High,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum TwilightBandOption {
    Narrow,
    Medium,
    Wide,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum NightBrightnessOption {
    Dim,
    Balanced,
    Bright,
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
enum FogPresetOption {
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
                    update_settings_display_mode_backgrounds,
                    update_settings_resolution_backgrounds,
                    update_day_length_backgrounds,
                    update_time_scale_backgrounds,
                    update_rayleigh_backgrounds,
                    update_mie_backgrounds,
                    update_mie_direction_backgrounds,
                    update_exposure_backgrounds,
                    update_twilight_backgrounds,
                    update_night_backgrounds,
                    update_fog_backgrounds,
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
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(16.0),
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

                    menu.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_button(row, &font, "Save", PauseMenuButton::Save);
                        spawn_button(row, &font, "Load", PauseMenuButton::Load);
                        spawn_button(row, &font, "Settings", PauseMenuButton::Settings);
                    });

                    menu.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(TextBundle {
                            text: Text::from_section(
                                "Host Game",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 22.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        spawn_labeled_input(
                            section,
                            &font,
                            "Session Password",
                            "Required for clients",
                            MultiplayerField::HostPassword,
                        );

                        spawn_button(section, &font, "Start Server", PauseMenuButton::StartServer);
                    });

                    menu.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(TextBundle {
                            text: Text::from_section(
                                "Join Game",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 22.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

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
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(10.0),
                                    ..default()
                                },
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
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(6.0),
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                                    ..default()
                                },
                                background_color: Color::srgba(0.05, 0.05, 0.05, 0.8).into(),
                                ..default()
                            })
                            .insert(FavoritesList)
                            .with_children(|favorites| {
                                favorites.spawn(TextBundle {
                                    text: Text::from_section(
                                        "Favorite Servers",
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: 18.0,
                                            color: Color::WHITE,
                                        },
                                    ),
                                    ..default()
                                });

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
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ..default()
        })
        .with_children(|column| {
            column.spawn(TextBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });

            column
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(320.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.2, 0.2, 0.2, 0.95).into(),
                        ..default()
                    },
                    InputField { field },
                ))
                .with_children(|input| {
                    input.spawn((
                        TextBundle {
                            text: Text::from_section(
                                placeholder,
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    color: Color::srgba(0.8, 0.8, 0.8, 0.9),
                                },
                            ),
                            ..default()
                        },
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

fn spawn_settings_dialog(
    commands: &mut Commands,
    root_entity: Option<Entity>,
    font: &Handle<Font>,
    settings_state: SettingsState,
) -> Entity {
    let mut dialog_entity = commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(70.0),
                height: Val::Percent(70.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            background_color: Color::srgba(0.08, 0.08, 0.08, 0.95).into(),
            ..default()
        },
        SettingsDialogRoot,
    ));

    dialog_entity.with_children(|dialog| {
        dialog.spawn(TextBundle {
            text: Text::from_section(
                "Settings",
                TextStyle {
                    font: font.clone(),
                    font_size: 28.0,
                    color: Color::WHITE,
                },
            ),
            ..default()
        });

        dialog
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|tabs| {
                spawn_settings_tab_button(tabs, font, "Graphics", SettingsTabButton::Graphics);
                spawn_settings_tab_button(tabs, font, "Gameplay", SettingsTabButton::Gameplay);
                spawn_settings_tab_button(tabs, font, "Atmosphere", SettingsTabButton::Atmosphere);
            });

        dialog
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                background_color: Color::srgba(0.12, 0.12, 0.12, 0.95).into(),
                ..default()
            })
            .with_children(|content| {
                content
                    .spawn((
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            visibility: if settings_state.active_tab == SettingsTab::Graphics {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            },
                            ..default()
                        },
                        GraphicsTabContent,
                    ))
                    .with_children(|graphics| {
                        graphics.spawn(TextBundle {
                            text: Text::from_section(
                                "Graphics Quality",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        graphics
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
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

                        graphics.spawn(TextBundle {
                            text: Text::from_section(
                                "Anti-Aliasing",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        graphics
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
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

                        graphics.spawn(TextBundle {
                            text: Text::from_section(
                                "Display Mode",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        graphics
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
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

                        graphics.spawn(TextBundle {
                            text: Text::from_section(
                                "Resolution",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        graphics
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    row_gap: Val::Px(8.0),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },
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
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            visibility: if settings_state.active_tab == SettingsTab::Gameplay {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            },
                            ..default()
                        },
                        GameplayTabContent,
                    ))
                    .with_children(|gameplay| {
                        gameplay.spawn(TextBundle {
                            text: Text::from_section(
                                "Gameplay settings coming soon.",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 18.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });
                    });

                content
                    .spawn((
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            visibility: if settings_state.active_tab == SettingsTab::Atmosphere {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            },
                            ..default()
                        },
                        AtmosphereTabContent,
                    ))
                    .with_children(|atmosphere| {
                        atmosphere.spawn(TextBundle {
                            text: Text::from_section(
                                "Day/Night Cycle",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
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

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "0.5x time",
                                    TimeScaleOption::Slow,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "1x time",
                                    TimeScaleOption::RealTime,
                                );
                                spawn_graphics_option(row, font, "2x time", TimeScaleOption::Fast);
                            });

                        atmosphere.spawn(TextBundle {
                            text: Text::from_section(
                                "Scattering Colors",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Soft blue",
                                    RayleighOption::Gentle,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Balanced",
                                    RayleighOption::Balanced,
                                );
                                spawn_graphics_option(row, font, "Vivid", RayleighOption::Vivid);
                            });

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(row, font, "Soft haze", MieOption::Soft);
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Balanced haze",
                                    MieOption::Standard,
                                );
                                spawn_graphics_option(row, font, "Dense glow", MieOption::Dense);
                            });

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Wide glow",
                                    MieDirectionOption::Broad,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Forward",
                                    MieDirectionOption::Standard,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Focused",
                                    MieDirectionOption::Forward,
                                );
                            });

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
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

                        atmosphere.spawn(TextBundle {
                            text: Text::from_section(
                                "Twilight & Night",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Tight",
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

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Darker nights",
                                    NightBrightnessOption::Dim,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Balanced nights",
                                    NightBrightnessOption::Balanced,
                                );
                                spawn_graphics_option(
                                    row,
                                    font,
                                    "Bright nights",
                                    NightBrightnessOption::Bright,
                                );
                            });

                        atmosphere.spawn(TextBundle {
                            text: Text::from_section(
                                "Fog Density",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });

                        atmosphere
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                },
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
                ButtonBundle {
                    style: Style {
                        width: Val::Px(120.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.25, 0.25, 0.25, 0.9).into(),
                    ..default()
                },
                CloseSettingsButton,
            ))
            .with_children(|button| {
                button.spawn(TextBundle {
                    text: Text::from_section(
                        "Close",
                        TextStyle {
                            font: font.clone(),
                            font_size: 18.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                });
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
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.18, 0.18, 0.18, 0.9).into(),
                ..default()
            },
            tab,
        ))
        .with_children(|button| {
            button.spawn(TextBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: 18.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });
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
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.2, 0.2, 0.2, 0.9).into(),
                ..default()
            },
            tag,
        ))
        .with_children(|button| {
            button.spawn(TextBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });
        });
}

fn close_menu(
    commands: &mut Commands,
    state: &mut PauseMenuState,
    form_state: &mut MultiplayerFormState,
    settings_state: &mut SettingsState,
) {
    if let Some(root) = state.root_entity.take() {
        commands.entity(root).despawn_recursive();
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
    mut tab_query: Query<(&Interaction, &SettingsTabButton), (Changed<Interaction>, With<Button>)>,
    mut quality_query: Query<
        (&Interaction, &GraphicsQualityOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut aa_query: Query<(&Interaction, &AntiAliasingOption), (Changed<Interaction>, With<Button>)>,
    mut display_query: Query<
        (&Interaction, &DisplayModeOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut resolution_query: Query<
        (&Interaction, &ResolutionOption),
        (Changed<Interaction>, With<Button>),
    >,
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
    mut close_query: Query<&Interaction, (Changed<Interaction>, With<CloseSettingsButton>)>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut atmosphere_settings: ResMut<AtmosphereSettings>,
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

    let base_rayleigh = Vec3::new(5.5, 13.0, 22.4) * 0.0012;
    let base_mie = Vec3::splat(0.005);

    for (interaction, option) in day_length_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.day_length = *option;
            atmosphere_settings.day_length = match option {
                DayLengthOption::Short => 600.0,
                DayLengthOption::Standard => 1800.0,
                DayLengthOption::Long => 3600.0,
            };
            atmosphere_settings.time %= atmosphere_settings.day_length;
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
    mut graphics_query: Query<&mut Visibility, With<GraphicsTabContent>>,
    mut gameplay_query: Query<&mut Visibility, With<GameplayTabContent>>,
    mut atmosphere_query: Query<&mut Visibility, With<AtmosphereTabContent>>,
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

    if let Ok(mut atmosphere_visibility) = atmosphere_query.get_single_mut() {
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
    mut char_evr: EventReader<ReceivedCharacter>,
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
        if ev.char.is_control() {
            continue;
        }

        if let Some(field) = form_state.active_field {
            let target = get_field_value_mut(&mut form_state, field);

            match field {
                MultiplayerField::JoinPort => {
                    if ev.char.is_ascii_digit() {
                        target.push(ev.char);
                    }
                }
                _ => {
                    target.push(ev.char);
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

        text.sections[0].value = display_value.to_string();
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
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.18, 0.18, 0.18, 0.9).into(),
                ..default()
            },
            FavoriteButton(index),
        ))
        .with_children(|button| {
            button.spawn(TextBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });
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

        if let Some(entry) = form_state.favorites.get(favorite.0) {
            form_state.join_ip = entry.ip.clone();
            form_state.join_port = entry.port.clone();
            form_state.join_password = entry.password.clone();
            form_state.active_field = None;
            info!("Loaded favorite server {}:{}", entry.ip, entry.port);
        }
    }
}
