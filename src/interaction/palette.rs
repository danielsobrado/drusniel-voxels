use crate::chat::ChatState;
use crate::menu::PauseMenuState;
use crate::props::{Prop, PropAssets, PropConfig, PropType};
use crate::voxel::types::VoxelType;
use bevy::input::keyboard::ReceivedCharacter;
use bevy::prelude::*;

#[derive(Clone, PartialEq)]
pub enum PlacementSelection {
    Voxel(VoxelType),
    Prop { id: String, prop_type: PropType },
}

#[derive(Clone)]
pub struct PaletteItem {
    pub label: String,
    pub tags: Vec<String>,
    pub selection: PlacementSelection,
}

#[derive(Resource, Default)]
pub struct PaletteItems(pub Vec<PaletteItem>);

#[derive(Resource, Default)]
pub struct PlacementPaletteState {
    pub open: bool,
    pub search: String,
    pub items_initialized: bool,
    pub needs_redraw: bool,
    pub active_selection: Option<PlacementSelection>,
    pub root: Option<Entity>,
}

#[derive(Component)]
struct PaletteRoot;

#[derive(Component)]
struct PaletteList;

#[derive(Component)]
struct PaletteSearchText;

#[derive(Component)]
struct PaletteSelectionText;

#[derive(Component)]
struct PaletteItemButton(usize);

pub fn initialize_palette_items(
    mut items: ResMut<PaletteItems>,
    mut palette: ResMut<PlacementPaletteState>,
    config: Res<PropConfig>,
) {
    if palette.items_initialized {
        return;
    }

    let mut all_items = Vec::new();

    for voxel in [
        VoxelType::TopSoil,
        VoxelType::SubSoil,
        VoxelType::Rock,
        VoxelType::Sand,
        VoxelType::Clay,
        VoxelType::Water,
        VoxelType::Wood,
        VoxelType::Leaves,
        VoxelType::DungeonWall,
        VoxelType::DungeonFloor,
    ] {
        all_items.push(PaletteItem {
            label: format!("{:?}", voxel),
            tags: voxel_tags(voxel),
            selection: PlacementSelection::Voxel(voxel),
        });
    }

    for (category, list) in [
        (PropType::Tree, config.props.trees.as_slice()),
        (PropType::Rock, config.props.rocks.as_slice()),
        (PropType::Bush, config.props.bushes.as_slice()),
        (PropType::Flower, config.props.flowers.as_slice()),
    ] {
        for def in list {
            let mut tags = vec![format!("{:?}", category).to_lowercase(), "prop".to_string()];
            for spawn in &def.spawn_on {
                tags.push(spawn.to_lowercase());
            }
            all_items.push(PaletteItem {
                label: def.id.clone(),
                tags,
                selection: PlacementSelection::Prop {
                    id: def.id.clone(),
                    prop_type: category,
                },
            });
        }
    }

    items.0 = all_items;
    palette.items_initialized = true;
    palette.needs_redraw = palette.open;
}

pub fn toggle_palette(
    keys: Res<ButtonInput<KeyCode>>,
    pause_state: Res<PauseMenuState>,
    chat_state: Option<Res<ChatState>>,
    mut palette: ResMut<PlacementPaletteState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if pause_state.open || chat_state.as_ref().map(|c| c.active).unwrap_or(false) {
        return;
    }

    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    palette.open = !palette.open;

    if palette.open {
        spawn_palette_ui(&mut commands, &asset_server, &mut palette);
        palette.needs_redraw = true;
    } else {
        despawn_palette_ui(&mut commands, &mut palette);
    }
}

pub fn handle_palette_input(
    mut palette: ResMut<PlacementPaletteState>,
    pause_state: Res<PauseMenuState>,
    chat_state: Option<Res<ChatState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut char_evr: EventReader<ReceivedCharacter>,
    mut commands: Commands,
) {
    if !palette.open || pause_state.open || chat_state.as_ref().map(|c| c.active).unwrap_or(false) {
        return;
    }

    let mut changed = false;

    if keys.just_pressed(KeyCode::Escape) {
        palette.open = false;
        palette.needs_redraw = true;
        despawn_palette_ui(&mut commands, &mut palette);
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        palette.search.pop();
        changed = true;
    }

    for ev in char_evr.read() {
        if !ev.char.is_control() {
            palette.search.push(ev.char);
            changed = true;
        }
    }

    if changed {
        palette.needs_redraw = true;
    }
}

pub fn handle_palette_item_click(
    mut interactions: Query<(&Interaction, &PaletteItemButton), Changed<Interaction>>,
    items: Res<PaletteItems>,
    mut palette: ResMut<PlacementPaletteState>,
    mut held: ResMut<crate::interaction::HeldBlock>,
) {
    if !palette.open {
        return;
    }

    for (interaction, button) in interactions.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Some(item) = items.0.get(button.0).cloned() {
                palette.active_selection = Some(item.selection.clone());
                palette.needs_redraw = true;

                if let PlacementSelection::Voxel(voxel) = item.selection {
                    held.block_type = voxel;
                }
            }
        }
    }
}

pub fn refresh_palette_ui(
    items: Res<PaletteItems>,
    mut palette: ResMut<PlacementPaletteState>,
    mut search_query: Query<&mut Text, With<PaletteSearchText>>,
    mut selection_query: Query<&mut Text, With<PaletteSelectionText>>,
    list_query: Query<Entity, With<PaletteList>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if !palette.open || !palette.needs_redraw {
        return;
    }

    if let Ok(mut search_text) = search_query.get_single_mut() {
        search_text.sections[0].value = format!("Search: {}", palette.search);
    }

    if let Ok(mut selection_text) = selection_query.get_single_mut() {
        selection_text.sections[0].value = match &palette.active_selection {
            Some(PlacementSelection::Voxel(v)) => format!("Selected: {:?}", v),
            Some(PlacementSelection::Prop { id, prop_type }) => {
                format!("Selected: {} ({:?})", id, prop_type)
            }
            None => "Selected: (none)".to_string(),
        };
    }

    if let Ok(list_entity) = list_query.get_single() {
        commands.entity(list_entity).despawn_descendants();

        let search_lower = palette.search.to_lowercase();

        let mut matches: Vec<(usize, &PaletteItem)> = items
            .0
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if search_lower.is_empty() {
                    return true;
                }
                let label_match = item.label.to_lowercase().contains(&search_lower);
                let tag_match = item
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&search_lower));
                label_match || tag_match
            })
            .collect();

        matches.sort_by(|a, b| a.1.label.cmp(&b.1.label));

        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        for (index, item) in matches.iter().take(40) {
            let is_selected = palette
                .active_selection
                .as_ref()
                .map(|sel| sel == &item.selection)
                .unwrap_or(false);

            commands.entity(list_entity).with_children(|list| {
                list.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                            margin: UiRect::bottom(Val::Px(6.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        background_color: if is_selected {
                            Color::srgba(0.25, 0.4, 0.7, 0.8).into()
                        } else {
                            Color::srgba(0.15, 0.15, 0.18, 0.85).into()
                        },
                        ..default()
                    },
                    PaletteItemButton(*index),
                ))
                .with_children(|button| {
                    button.spawn(TextBundle {
                        text: Text::from_section(
                            &item.label,
                            TextStyle {
                                font: font.clone(),
                                font_size: 16.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    });

                    if !item.tags.is_empty() {
                        button.spawn(TextBundle {
                            text: Text::from_section(
                                format!("Tags: {}", item.tags.join(", ")),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 12.0,
                                    color: Color::srgba(0.8, 0.8, 0.8, 0.9),
                                },
                            ),
                            ..default()
                        });
                    }
                });
            });
        }
    }

    palette.needs_redraw = false;
}

pub fn place_prop_from_palette(
    mouse: Res<ButtonInput<MouseButton>>,
    edit_mode: Res<crate::interaction::EditMode>,
    delete_mode: Res<crate::interaction::DeleteMode>,
    drag_state: Res<crate::interaction::DragState>,
    targeted: Res<crate::interaction::TargetedBlock>,
    palette: Res<PlacementPaletteState>,
    prop_assets: Res<PropAssets>,
    mut commands: Commands,
) {
    if !palette.open && palette.active_selection.is_none() {
        return;
    }

    if !edit_mode.enabled || delete_mode.enabled || drag_state.dragged_block.is_some() {
        return;
    }

    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(PlacementSelection::Prop { id, prop_type }) = &palette.active_selection else {
        return;
    };

    let Some((block_pos, normal)) = (targeted.position, targeted.normal) else {
        return;
    };

    let place_pos = block_pos + normal;
    let translation = Vec3::new(
        place_pos.x as f32 + 0.5,
        place_pos.y as f32 + 0.5,
        place_pos.z as f32 + 0.5,
    );

    let Some(scene) = prop_assets.scenes.get(id) else {
        return;
    };

    commands.spawn((
        SceneRoot(scene.clone()),
        Transform::from_translation(translation),
        Prop {
            id: id.clone(),
            prop_type: *prop_type,
        },
    ));
}

fn spawn_palette_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    palette: &mut ResMut<PlacementPaletteState>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    right: Val::Px(20.0),
                    width: Val::Px(360.0),
                    max_height: Val::Px(640.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(10.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::srgba(0.05, 0.05, 0.07, 0.9).into(),
                ..default()
            },
            PaletteRoot,
        ))
        .with_children(|root| {
            root.spawn(TextBundle {
                text: Text::from_section(
                    "Placement Palette (Tab to close)",
                    TextStyle {
                        font: font.clone(),
                        font_size: 18.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });

            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Search:",
                        TextStyle {
                            font: font.clone(),
                            font_size: 14.0,
                            color: Color::srgba(0.85, 0.85, 0.85, 1.0),
                        },
                    ),
                    ..default()
                },
                PaletteSearchText,
            ));

            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Selected: (none)",
                        TextStyle {
                            font: font.clone(),
                            font_size: 14.0,
                            color: Color::srgba(0.85, 0.85, 0.85, 1.0),
                        },
                    ),
                    ..default()
                },
                PaletteSelectionText,
            ));

            root.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        max_height: Val::Px(520.0),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    background_color: Color::srgba(0.1, 0.1, 0.12, 0.8).into(),
                    ..default()
                },
                PaletteList,
            ));

            root.spawn(TextBundle {
                text: Text::from_section(
                    "Right click while editing to place props. Voxels use the held block.",
                    TextStyle {
                        font: font.clone(),
                        font_size: 12.0,
                        color: Color::srgba(0.8, 0.8, 0.8, 0.8),
                    },
                ),
                ..default()
            });
        })
        .id();

    palette.root = Some(root);
}

fn despawn_palette_ui(commands: &mut Commands, palette: &mut ResMut<PlacementPaletteState>) {
    if let Some(entity) = palette.root.take() {
        commands.entity(entity).despawn_recursive();
    }
}

fn voxel_tags(voxel: VoxelType) -> Vec<String> {
    match voxel {
        VoxelType::TopSoil => vec!["material".into(), "soil".into(), "ground".into()],
        VoxelType::SubSoil => vec!["material".into(), "soil".into()],
        VoxelType::Rock => vec!["material".into(), "stone".into()],
        VoxelType::Sand => vec!["material".into(), "sand".into()],
        VoxelType::Clay => vec!["material".into(), "clay".into()],
        VoxelType::Water => vec!["liquid".into(), "water".into()],
        VoxelType::Wood => vec!["material".into(), "wood".into(), "tree".into()],
        VoxelType::Leaves => vec!["material".into(), "foliage".into()],
        VoxelType::DungeonWall => vec!["material".into(), "dungeon".into()],
        VoxelType::DungeonFloor => vec!["material".into(), "dungeon".into()],
        VoxelType::Air | VoxelType::Bedrock => vec!["hidden".into()],
    }
}
