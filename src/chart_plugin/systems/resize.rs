use super::{
    ui_helpers::{EditableText, ResizeMarker},
    RedrawArrow, VeloNode, VeloNodeContainer,
};
use crate::UiState;
use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

pub fn resize_entity_start(
    mut interaction_query: Query<
        (&Interaction, &Parent, &ResizeMarker),
        (Changed<Interaction>, With<ResizeMarker>),
    >,
    mut button_query: Query<&VeloNode, With<VeloNode>>,
    mut state: ResMut<UiState>,
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
            Interaction::None => {
                primary_window.cursor.icon = CursorIcon::Default;
            }
        }
    }
}

pub fn resize_entity_end(
    mut mouse_motion_events: EventReader<MouseMotion>,
    state: Res<UiState>,
    mut rectangle_query: Query<(&VeloNodeContainer, &mut Style), With<VeloNodeContainer>>,
    mut text_input_query: Query<
        (&EditableText, &mut Style),
        (With<EditableText>, Without<VeloNodeContainer>),
    >,
    mut events: EventWriter<RedrawArrow>,
) {
    for event in mouse_motion_events.iter() {
        if let Some((id, resize_marker)) = state.entity_to_resize {
            for (rectangle, mut button_style) in &mut rectangle_query {
                if id == rectangle.id {
                    events.send(RedrawArrow { id });
                    #[allow(unused)]
                    let mut delta = event.delta;
                    #[cfg(target_arch = "wasm32")]
                    {
                        // MouseMotion returns different values depending on platform
                        delta = Vec2::new(delta.x / 2., delta.y / 2.);
                    }
                    match resize_marker {
                        ResizeMarker::TopLeft => {
                            if let Val::Px(width) = button_style.size.width {
                                button_style.size.width = Val::Px(width - delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                button_style.size.height = Val::Px(height - delta.y);
                            }

                            if let Val::Px(x) = button_style.position.left {
                                button_style.position.left = Val::Px(x + delta.x);
                            }
                        }
                        ResizeMarker::TopRight => {
                            if let Val::Px(width) = button_style.size.width {
                                button_style.size.width = Val::Px(width + delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                button_style.size.height = Val::Px(height - delta.y);
                            }
                        }
                        ResizeMarker::BottomLeft => {
                            if let Val::Px(width) = button_style.size.width {
                                button_style.size.width = Val::Px(width - delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                button_style.size.height = Val::Px(height + delta.y);
                            }

                            if let Val::Px(x) = button_style.position.left {
                                button_style.position.left = Val::Px(x + delta.x);
                            }

                            if let Val::Px(y) = button_style.position.bottom {
                                button_style.position.bottom = Val::Px(y - delta.y);
                            }
                        }
                        ResizeMarker::BottomRight => {
                            if let Val::Px(width) = button_style.size.width {
                                button_style.size.width = Val::Px(width + delta.x);
                            }

                            if let Val::Px(height) = button_style.size.height {
                                button_style.size.height = Val::Px(height + delta.y);
                            }

                            if let Val::Px(y) = button_style.position.bottom {
                                button_style.position.bottom = Val::Px(y - delta.y);
                            }
                        }
                    }
                    for (text, mut text_style) in &mut text_input_query {
                        if text.id == id {
                            text_style.max_size.width = button_style.size.width;
                            text_style.max_size.height = button_style.size.height;
                        }
                    }
                }
            }
        }
    }
}
