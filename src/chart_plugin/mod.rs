#[cfg(not(target_arch = "wasm32"))]
use arboard::*;
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use bevy_prototype_lyon::{prelude::*, shapes};
#[cfg(not(target_arch = "wasm32"))]
use image::*;
use std::convert::TryInto;
#[path = "structs.rs"]
mod structs;
pub use structs::*;
#[path = "ui_helpers.rs"]
mod ui_helpers;
pub use ui_helpers::*;

pub struct ChartPlugin;

impl Plugin for ChartPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppState>();

        app.add_event::<AddRect>();

        app.add_startup_system(init_layout);

        app.add_systems((
            update_rectangle_pos,
            update_text_on_typing,
            create_new_rectangle,
            create_entity_event,
            resize_entity_start,
            resize_entity_end,
            connect_rectangles,
        ));

        #[cfg(not(target_arch = "wasm32"))]
        app.add_system(insert_image_from_clipboard);

        app.add_system(set_focused_entity);
    }
}

fn init_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/SourceCodePro-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 18.0,
        color: Color::BLACK,
    };
    commands
        .spawn((
            ButtonBundle {
                z_index: ZIndex::Global(1),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Px(10.),
                        top: Val::Px(10.),
                        ..Default::default()
                    },
                    size: Size::new(Val::Px(100.), Val::Px(100.)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            CreateRectButton,
        ))
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section("NEW RECT", text_style.clone()).with_style(Style {
                    position_type: PositionType::Relative,
                    ..default()
                }),
            ));
        });
}

fn connect_rectangles(
    mut commands: Commands,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ArrowConnectMarker>)>,
    mut state: ResMut<AppState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    for interaction in interaction_query.iter_mut() {
        if let Interaction::Clicked = interaction {
            match state.line_to_draw_start {
                Some(start) => {
                    if let Some(end) = window.cursor_position() {
                        let shape = shapes::Line(
                            camera
                                .viewport_to_world_2d(camera_transform, start)
                                .unwrap(),
                            camera.viewport_to_world_2d(camera_transform, end).unwrap(),
                        );
                        eprint!("end: {:?}", end);
                        commands.spawn((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shape),
                                ..default()
                            },
                            Stroke::new(Color::BLACK, 2.0),
                        ));

                        state.line_to_draw_start = None;
                    }
                }
                None => {
                    if let Some(pos) = window.cursor_position() {
                        eprint!("start: {:?}", pos);
                        state.line_to_draw_start = Some(pos);
                    }
                }
            }
        }
    }
}

fn create_entity_event(
    mut events: EventWriter<AddRect>,
    interaction_query: Query<
        (&Interaction, &CreateRectButton),
        (Changed<Interaction>, With<CreateRectButton>),
    >,
) {
    for (interaction, _) in &interaction_query {
        match *interaction {
            Interaction::Clicked => {
                events.send(AddRect);
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn set_focused_entity(
    mut interaction_query: Query<
        (&Interaction, &Rectangle),
        (Changed<Interaction>, With<Rectangle>),
    >,
    mut state: ResMut<AppState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
) {
    let mut window = windows.single_mut();
    for (interaction, rectangle) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.hold_entity = Some(rectangle.id);
            }
            Interaction::Hovered => {
                window.cursor.icon = CursorIcon::Text;
                state.entity_to_edit = Some(rectangle.id);
                if state.hold_entity.is_none() {
                    window.cursor.icon = CursorIcon::Move;
                }
            }
            Interaction::None => {
                state.entity_to_edit = None;
                window.cursor.icon = CursorIcon::Default;
            }
        }
    }
    if buttons.just_released(MouseButton::Left) {
        state.hold_entity = None;
        state.entity_to_resize = None;
    }
}

fn resize_entity_end(
    mut mouse_motion_events: EventReader<MouseMotion>,
    state: Res<AppState>,
    mut top_query: Query<(&Rectangle, &mut Style), With<Rectangle>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let primary_window = windows.single_mut();
    for event in mouse_motion_events.iter() {
        if let Some((id, resize_marker)) = state.entity_to_resize {
            for (rectangle, mut button_style) in &mut top_query {
                if id == rectangle.id {
                    let delta = event.delta;
                    match resize_marker {
                        ResizeMarker::TopLeft => {
                            if let Val::Px(width) = button_style.size.width {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.width = Val::Px(width - scale_factor * delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.height = Val::Px(height - scale_factor * delta.y);
                            }
                        }
                        ResizeMarker::TopRight => {
                            if let Val::Px(width) = button_style.size.width {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.width = Val::Px(width + scale_factor * delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.height = Val::Px(height - scale_factor * delta.y);
                            }
                        }
                        ResizeMarker::BottomLeft => {
                            if let Val::Px(width) = button_style.size.width {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.width = Val::Px(width - scale_factor * delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.height = Val::Px(height + scale_factor * delta.y);
                            }
                        }
                        ResizeMarker::BottomRight => {
                            if let Val::Px(width) = button_style.size.width {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.width = Val::Px(width + scale_factor * delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                let scale_factor = primary_window.resolution.scale_factor() as f32;
                                button_style.size.height = Val::Px(height + scale_factor * delta.y);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn resize_entity_start(
    mut interaction_query: Query<
        (&Interaction, &Parent, &ResizeMarker),
        (Changed<Interaction>, With<ResizeMarker>),
    >,
    mut button_query: Query<&Rectangle, With<Rectangle>>,
    mut state: ResMut<AppState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = windows.single_mut();
    for (interaction, parent, resize_marker) in &mut interaction_query {
        let rectangle = button_query.get_mut(parent.get()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                state.entity_to_resize = Some((rectangle.id, *resize_marker));
            }
            Interaction::Hovered => match *resize_marker {
                ResizeMarker::TopLeft => {
                    primary_window.cursor.icon = CursorIcon::NwseResize;
                }
                ResizeMarker::TopRight => {
                    primary_window.cursor.icon = CursorIcon::NeswResize;
                }
                ResizeMarker::BottomLeft => {
                    primary_window.cursor.icon = CursorIcon::NeswResize;
                }
                ResizeMarker::BottomRight => {
                    primary_window.cursor.icon = CursorIcon::NwseResize;
                }
            },
            Interaction::None => {}
        }
    }
}

fn update_rectangle_pos(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut sprite_position: Query<(&mut Style, &Top)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    state: ResMut<AppState>,
) {
    let (camera, camera_transform) = camera_q.single();
    for event in cursor_moved_events.iter() {
        for (mut style, top) in &mut sprite_position.iter_mut() {
            if Some(top.id) == state.hold_entity {
                if let Some(world_position) =
                    camera.viewport_to_world_2d(camera_transform, event.position)
                {
                    style.position.left = Val::Px(world_position.x);
                    style.position.bottom = Val::Px(world_position.y);
                }
            }
        }
    }
}

fn update_text_on_typing(
    mut char_evr: EventReader<ReceivedCharacter>,
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&mut Text, &EditableText), With<EditableText>>,
    state: Res<AppState>,
) {
    for (mut text, editable_text) in &mut query.iter_mut() {
        if Some(editable_text.id) == state.entity_to_edit {
            if keys.just_pressed(KeyCode::Back) {
                let mut str = text.sections[0].value.clone();
                str.pop();
                text.sections[0].value = str;
            } else {
                for ev in char_evr.iter() {
                    text.sections[0].value = format!("{}{}", text.sections[0].value, ev.char);
                }
            }
        }
    }
}

fn create_new_rectangle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<AddRect>,
    mut state: ResMut<AppState>,
) {
    for _ in events.iter() {
        state.entity_counter += 1;
        let font = asset_server.load("fonts/SourceCodePro-Regular.ttf");
        let text_style = TextStyle {
            font,
            font_size: 18.0,
            color: Color::BLACK,
        };
        let box_size = Vec2::new(100.0, 100.0);
        // Rectangle
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            ..Default::default()
                        },
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                Top {
                    id: state.entity_counter,
                },
            ))
            .with_children(|builder| {
                builder
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(box_size.x), Val::Px(box_size.y)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Rectangle {
                            id: state.entity_counter,
                        },
                    ))
                    .with_children(|builder| {
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(50.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(0.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.9, 0.9, 1.0).into(),
                                ..default()
                            },
                            ArrowConnectMarker::Top,
                        ));
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(0.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(50.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.9, 0.9, 1.0).into(),
                                ..default()
                            },
                            ArrowConnectMarker::Left,
                        ));
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(50.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(100.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.9, 0.9, 1.0).into(),
                                ..default()
                            },
                            ArrowConnectMarker::Bottom,
                        ));
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(100.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(50.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.9, 0.9, 1.0).into(),
                                ..default()
                            },
                            ArrowConnectMarker::Right,
                        ));
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(0.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(0.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                ..default()
                            },
                            ResizeMarker::TopLeft,
                        ));
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(100.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(0.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                ..default()
                            },
                            ResizeMarker::TopRight,
                        ));
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(100.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(100.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                ..default()
                            },
                            ResizeMarker::BottomRight,
                        ));
                        builder.spawn((
                            ButtonBundle {
                                style: get_marker_style(UiRect {
                                    left: Val::Percent(0.),
                                    right: Val::Percent(0.),
                                    top: Val::Percent(100.),
                                    bottom: Val::Percent(0.),
                                }),
                                background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                ..default()
                            },
                            ResizeMarker::BottomLeft,
                        ));
                        builder.spawn((
                            TextBundle::from_section("", text_style.clone())
                                .with_style(Style {
                                    position_type: PositionType::Relative,
                                    ..default()
                                })
                                .with_text_alignment(TextAlignment::Center),
                            EditableText {
                                id: state.entity_counter,
                            },
                        ));
                    });
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn insert_image_from_clipboard(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<AppState>,
) {
    let mut clipboard = Clipboard::new().unwrap();
    match clipboard.get_image() {
        Ok(image) => {
            state.entity_counter += 1;
            clipboard.clear().unwrap();
            let image: RgbaImage = ImageBuffer::from_raw(
                image.width.try_into().unwrap(),
                image.height.try_into().unwrap(),
                image.bytes.into_owned(),
            )
            .unwrap();
            let size: Extent3d = Extent3d {
                width: image.width(),
                height: image.height(),
                ..Default::default()
            };
            let image = Image::new(
                size,
                TextureDimension::D2,
                image.to_vec(),
                TextureFormat::Rgba8UnormSrgb,
            );
            let image = images.add(image);
            commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    },
                    Top {
                        id: state.entity_counter,
                    },
                ))
                .with_children(|builder| {
                    builder
                        .spawn((
                            ButtonBundle {
                                image: image.into(),
                                style: Style {
                                    size: Size::new(
                                        Val::Px(size.width as f32),
                                        Val::Px(size.height as f32),
                                    ),
                                    // horizontally center child text
                                    justify_content: JustifyContent::Center,
                                    // vertically center child text
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            Rectangle {
                                id: state.entity_counter,
                            },
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                ButtonBundle {
                                    style: get_marker_style(UiRect {
                                        left: Val::Percent(0.),
                                        right: Val::Percent(0.),
                                        top: Val::Percent(0.),
                                        bottom: Val::Percent(0.),
                                    }),
                                    background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                    ..default()
                                },
                                ResizeMarker::TopLeft,
                            ));
                            builder.spawn((
                                ButtonBundle {
                                    style: get_marker_style(UiRect {
                                        left: Val::Percent(100.),
                                        right: Val::Percent(0.),
                                        top: Val::Percent(0.),
                                        bottom: Val::Percent(0.),
                                    }),
                                    background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                    ..default()
                                },
                                ResizeMarker::TopRight,
                            ));
                            builder.spawn((
                                ButtonBundle {
                                    style: get_marker_style(UiRect {
                                        left: Val::Percent(100.),
                                        right: Val::Percent(0.),
                                        top: Val::Percent(100.),
                                        bottom: Val::Percent(0.),
                                    }),
                                    background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                    ..default()
                                },
                                ResizeMarker::BottomRight,
                            ));
                            builder.spawn((
                                ButtonBundle {
                                    style: get_marker_style(UiRect {
                                        left: Val::Percent(0.),
                                        right: Val::Percent(0.),
                                        top: Val::Percent(100.),
                                        bottom: Val::Percent(0.),
                                    }),
                                    background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                                    ..default()
                                },
                                ResizeMarker::BottomLeft,
                            ));
                        });
                });
        }
        Err(_) => {}
    }
}
