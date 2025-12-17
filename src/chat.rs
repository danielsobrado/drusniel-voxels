use bevy::{input::keyboard::ReceivedCharacter, prelude::*};

use crate::menu::PauseMenuState;
use crate::network::NetworkSession;

const MAX_CHAT_MESSAGES: usize = 10;

#[derive(Resource, Debug)]
pub struct ChatState {
    pub active: bool,
    pub buffer: String,
    pub messages: Vec<ChatMessage>,
    pub username: String,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            active: false,
            buffer: String::new(),
            messages: Vec::new(),
            username: "Player".to_string(),
        }
    }
}

impl ChatState {
    pub fn push_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        if self.messages.len() > MAX_CHAT_MESSAGES {
            let overflow = self.messages.len() - MAX_CHAT_MESSAGES;
            self.messages.drain(0..overflow);
        }
    }

    pub fn push_system(&mut self, content: impl Into<String>) {
        self.push_message(ChatMessage::system(content));
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub user: String,
    pub content: String,
}

#[derive(Component)]
struct ChatOverlayRoot;

#[derive(Component)]
struct ChatLogText;

#[derive(Component)]
struct ChatInputText;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatState>()
            .add_systems(Startup, spawn_chat_overlay)
            .add_systems(
                Update,
                (
                    toggle_chat_input,
                    process_chat_characters,
                    submit_chat_message,
                    update_chat_log,
                    update_chat_prompt,
                ),
            );
    }
}

fn spawn_chat_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(12.0),
                        right: Val::Px(12.0),
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    min_width: Val::Px(280.0),
                    max_width: Val::Px(420.0),
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.45).into(),
                ..default()
            },
            ChatOverlayRoot,
        ))
        .with_children(|overlay| {
            overlay.spawn((
                TextBundle {
                    text: Text::from_section(
                        "", // filled by update_chat_log
                        TextStyle {
                            font: font.clone(),
                            font_size: 14.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                ChatLogText,
            ));

            overlay.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Press Ctrl+A to chat",
                        TextStyle {
                            font: font.clone(),
                            font_size: 13.0,
                            color: Color::srgba(0.9, 0.9, 0.9, 0.9),
                        },
                    ),
                    ..default()
                },
                ChatInputText,
            ));
        });
}

fn toggle_chat_input(keys: Res<ButtonInput<KeyCode>>, mut chat_state: ResMut<ChatState>) {
    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keys.just_pressed(KeyCode::KeyA)
    {
        chat_state.active = true;
        chat_state.buffer.clear();
    }

    if chat_state.active && keys.just_pressed(KeyCode::Escape) {
        chat_state.active = false;
        chat_state.buffer.clear();
    }
}

fn process_chat_characters(
    mut chat_state: ResMut<ChatState>,
    mut char_evr: EventReader<ReceivedCharacter>,
    keys: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<PauseMenuState>>,
) {
    if !chat_state.active || pause_state.as_ref().map(|p| p.open).unwrap_or(false) {
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        chat_state.buffer.pop();
    }

    for ev in char_evr.read() {
        let ch = ev.char;
        if ch == '\n' || ch == '\r' {
            continue;
        }
        chat_state.buffer.push(ch);
    }
}

fn submit_chat_message(
    keys: Res<ButtonInput<KeyCode>>,
    mut chat_state: ResMut<ChatState>,
    network: Res<NetworkSession>,
) {
    if !chat_state.active || !keys.just_pressed(KeyCode::Enter) {
        return;
    }

    if chat_state.buffer.is_empty() {
        chat_state.active = false;
        return;
    }

    if !network.is_connected() {
        chat_state.push_system("Cannot send chat: not connected");
    } else {
        chat_state.push_message(ChatMessage {
            user: chat_state.username.clone(),
            content: chat_state.buffer.clone(),
        });
    }

    chat_state.buffer.clear();
    chat_state.active = false;
}

fn update_chat_log(chat_state: Res<ChatState>, mut query: Query<&mut Text, With<ChatLogText>>) {
    if !chat_state.is_changed() {
        return;
    }

    if let Ok(mut text) = query.get_single_mut() {
        let body = chat_state
            .messages
            .iter()
            .map(|msg| format!("{}: {}", msg.user, msg.content))
            .collect::<Vec<_>>()
            .join("\n");

        text.sections[0].value = body;
    }
}

fn update_chat_prompt(
    chat_state: Res<ChatState>,
    mut query: Query<&mut Text, With<ChatInputText>>,
) {
    if !chat_state.is_changed() {
        return;
    }

    if let Ok(mut text) = query.get_single_mut() {
        if chat_state.active {
            text.sections[0].value = format!("{}: {}", chat_state.username, chat_state.buffer);
        } else {
            text.sections[0].value = "Press Ctrl+A to chat".to_string();
        }
    }
}

impl ChatMessage {
    fn system(content: impl Into<String>) -> Self {
        Self {
            user: "System".to_string(),
            content: content.into(),
        }
    }
}
