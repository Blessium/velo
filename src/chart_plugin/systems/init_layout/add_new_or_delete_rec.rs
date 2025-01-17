use bevy::prelude::*;

use crate::chart_plugin::ui_helpers::{get_tooltip, ButtonAction, GenericButton, Tooltip};

pub fn add_new_delete_rec(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    font: Handle<Font>,
    create_component: ButtonAction,
    delete_component: ButtonAction,
) -> Entity {
    let node = commands
        .spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                size: Size::new(Val::Percent(90.), Val::Percent(14.)),
                margin: UiRect::all(Val::Px(5.)),
                justify_content: JustifyContent::Start,
                ..default()
            },
            ..default()
        })
        .id();
    let top_new = commands
        .spawn(NodeBundle {
            background_color: Color::BLACK.with_a(0.5).into(),
            style: Style {
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Stretch,
                margin: UiRect::all(Val::Px(5.)),
                size: Size::new(Val::Percent(23.), Val::Percent(100.)),
                ..default()
            },
            ..default()
        })
        .id();
    let new_rec = commands
        .spawn((
            ButtonBundle {
                image: asset_server.load("rec-add.png").into(),
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Px(-2.),
                        right: Val::Px(0.),
                        top: Val::Px(-2.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                ..default()
            },
            create_component,
            GenericButton,
        ))
        .with_children(|builder| {
            builder.spawn((
                get_tooltip(font.clone(), "New Rectangle".to_string(), 14.),
                Tooltip,
            ));
        })
        .id();
    let top_del = commands
        .spawn(NodeBundle {
            background_color: Color::BLACK.with_a(0.5).into(),
            style: Style {
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(5.)),
                align_self: AlignSelf::Stretch,
                size: Size::new(Val::Percent(23.), Val::Percent(100.)),
                ..default()
            },
            ..default()
        })
        .id();
    let del_rec = commands
        .spawn((
            ButtonBundle {
                image: asset_server.load("rec-del.png").into(),
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Px(-2.),
                        right: Val::Px(0.),
                        top: Val::Px(-2.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                ..default()
            },
            delete_component,
            GenericButton,
        ))
        .with_children(|builder| {
            builder.spawn((
                get_tooltip(font.clone(), "Delete Rectangle".to_string(), 14.),
                Tooltip,
            ));
        })
        .id();
    commands.entity(top_new).add_child(new_rec);
    commands.entity(top_del).add_child(del_rec);
    commands.entity(node).add_child(top_del);
    commands.entity(node).add_child(top_new);
    node
}
