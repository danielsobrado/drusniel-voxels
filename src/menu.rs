use crate::chat::ChatState;
use crate::network::NetworkSession;
use crate::voxel::{meshing::ChunkMesh, persistence, world::VoxelWorld};
use bevy::{input::keyboard::ReceivedCharacter, prelude::*};
use std::net::ToSocketAddrs;
use std::sync::mpsc::{self, Receiver, TryRecvError};
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
    receiver: Option<Receiver<ConnectOutcome>>,
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
            .init_resource::<MultiplayerFormState>()
            .init_resource::<ConnectTaskState>()
            .init_resource::<ChatState>()
            .init_resource::<NetworkSession>()
            .add_systems(
                Update,
                (
                    toggle_pause_menu,
                    handle_menu_buttons,
                    poll_connect_task_results,
                    handle_input_interaction,
                    process_input_characters,
                    update_input_texts,
                    update_input_backgrounds,
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
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if state.open {
        close_menu(&mut commands, &mut state, &mut form_state);
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

                    spawn_button(menu, &font, "Save", PauseMenuButton::Save);
                    spawn_button(menu, &font, "Load", PauseMenuButton::Load);

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

fn close_menu(
    commands: &mut Commands,
    state: &mut PauseMenuState,
    form_state: &mut MultiplayerFormState,
) {
    if let Some(root) = state.root_entity.take() {
        commands.entity(root).despawn_recursive();
    }
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
    mut form_state: ResMut<MultiplayerFormState>,
    mut connect_tasks: ResMut<ConnectTaskState>,
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

                connect_tasks.receiver = Some(rx);
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
                close_menu(&mut commands, &mut state, &mut form_state);
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
    let Some(receiver) = connect_tasks.receiver.take() else {
        return;
    };

    match receiver.try_recv() {
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
            chat.push_message(crate::chat::ChatMessage {
                user: chat.username.clone(),
                content: format!("Connected to {} ({} ms latency)", address, latency_ms),
            });
        }
        Ok(ConnectOutcome::Failure { message }) => {
            warn!("{}", message);
            chat.push_system(message);
            network.reset_client();
        }
        Err(TryRecvError::Disconnected) => {
            warn!("Connection attempt ended unexpectedly");
            chat.push_system("Connection failed: internal error");
            network.reset_client();
        }
        Err(TryRecvError::Empty) => {
            connect_tasks.receiver = Some(receiver);
        }
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
