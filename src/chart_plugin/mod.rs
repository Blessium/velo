use bevy::{
    prelude::*,
    text::BreakLineOn,
    window::{PrimaryWindow, WindowResized},
};
use serde::{Deserialize, Serialize};

use crate::canvas::arrow::components::{ArrowConnect, ArrowConnectPos, ArrowType};
use crate::canvas::arrow::events::{CreateArrow, RedrawArrow};
use crate::resources::AppState;
use crate::resources::LoadRequest;
use crate::resources::StaticState;
use crate::utils::ReflectableUuid;
use std::time::Duration;
use uuid::Uuid;
#[path = "ui_helpers/ui_helpers.rs"]
pub mod ui_helpers;
pub use ui_helpers::*;
#[path = "systems/save.rs"]
mod save_systems;
use save_systems::*;
#[path = "systems/load.rs"]
mod load_systems;
use load_systems::*;
#[path = "systems/keyboard.rs"]
mod keyboard_systems;
use keyboard_systems::*;
#[path = "systems/modal.rs"]
mod modal;
use modal::*;
#[path = "systems/init_layout/init_layout.rs"]
mod init_layout;
use init_layout::*;
#[path = "systems/resize.rs"]
mod resize;
use resize::*;
#[path = "systems/button_handlers.rs"]
mod button_handlers;
use button_handlers::*;
#[path = "systems/tabs.rs"]
mod tabs;
use tabs::*;
#[path = "systems/doc_list.rs"]
mod doc_list;
use doc_list::*;

pub struct ChartPlugin;

pub struct AddRect {
    pub node: JsonNode,
    pub image: Option<UiImage>,
}

pub struct HighlightEvent;

#[derive(Serialize, Deserialize)]
pub enum NodeType {
    Rect,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TextPos {
    Center,
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

#[derive(Serialize, Deserialize)]
pub struct JsonNodeText {
    pub text: String,
    pub pos: TextPos,
}

#[derive(Serialize, Deserialize)]
pub struct JsonNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub left: Val,
    pub bottom: Val,
    pub width: Val,
    pub height: Val,
    pub text: JsonNodeText,
    pub bg_color: Color,
    pub tags: Vec<String>,
    pub z_index: i32,
}

pub const MAX_CHECKPOINTS: i32 = 7;
pub const MAX_SAVED_DOCS_IN_MEMORY: i32 = 7;

#[derive(Resource, Default)]
pub struct UiState {
    pub modal_id: Option<ReflectableUuid>,
    pub arrow_type: ArrowType,
    pub entity_to_edit: Option<ReflectableUuid>,
    pub tab_to_edit: Option<ReflectableUuid>,
    pub doc_to_edit: Option<ReflectableUuid>,
    pub hold_entity: Option<ReflectableUuid>,
    pub entity_to_resize: Option<(ReflectableUuid, ResizeMarker)>,
    pub arrow_to_draw_start: Option<ArrowConnect>,
}

#[derive(Resource)]
pub struct BlinkTimer {
    timer: Timer,
}
impl Plugin for ChartPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>();
        app.init_resource::<StaticState>();
        app.init_resource::<AppState>();

        app.register_type::<VeloNode>();
        app.register_type::<EditableText>();
        app.register_type::<ArrowConnect>();
        app.register_type::<ResizeMarker>();
        app.register_type::<ReflectableUuid>();
        app.register_type_data::<ReflectableUuid, ReflectSerialize>();
        app.register_type_data::<ReflectableUuid, ReflectDeserialize>();
        app.register_type::<ArrowConnectPos>();

        app.register_type::<BreakLineOn>();

        app.add_event::<AddRect>();
        app.add_event::<CreateArrow>();
        app.add_event::<RedrawArrow>();
        app.add_event::<HighlightEvent>();

        app.add_startup_system(init_layout);

        app.add_systems((
            rec_button_handlers,
            update_rectangle_position,
            create_new_rectangle,
            resize_entity_start,
            resize_entity_end,
            set_focused_entity,
            cancel_modal,
            modal_keyboard_input_system,
            confirm_modal,
            resize_notificator,
        ));

        app.add_systems(
            (save_json, remove_save_request)
                .chain()
                .distributive_run_if(should_save),
        );

        app.add_systems(
            (load_json, remove_load_request)
                .chain()
                .distributive_run_if(should_load),
        );

        app.add_systems((
            change_color_pallete,
            change_arrow_type,
            change_text_pos,
            add_tab_handler,
            delete_tab_handler,
            rename_tab_handler,
            text_manipulation,
            mouse_scroll_list,
            list_item_click,
            new_doc_handler,
            rename_doc_handler,
            delete_doc_handler,
            save_doc_handler,
            keyboard_input_system,
        ));

        app.add_systems((
            button_generic_handler,
            selected_tab_handler,
            higlight_event_handler,
        ));
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_timestamp() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_timestamp() -> f64 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_millis() as f64
}

fn set_focused_entity(
    mut interaction_query: Query<(&Interaction, &VeloNode), (Changed<Interaction>, With<VeloNode>)>,
    mut state: ResMut<UiState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut holding_time: Local<(Duration, Option<ReflectableUuid>)>,
) {
    let mut window = windows.single_mut();
    for (interaction, rectangle) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                window.cursor.icon = CursorIcon::Text;
                *state = UiState::default();
                state.entity_to_edit = Some(rectangle.id);
                let now_ms = get_timestamp();
                *holding_time = (Duration::from_millis(now_ms as u64), Some(rectangle.id));
            }
            Interaction::Hovered => {
                if state.hold_entity.is_none() && state.entity_to_edit.is_none() {
                    window.cursor.icon = CursorIcon::Default;
                }
                if state.entity_to_edit.is_some() {
                    window.cursor.icon = CursorIcon::Text;
                }
            }
            Interaction::None => {
                window.cursor.icon = CursorIcon::Default;
            }
        }
    }

    if state.hold_entity.is_some() {
        window.cursor.icon = CursorIcon::Move;
    }

    let now_ms = get_timestamp();
    // 150ms delay before re-positioning the rectangle
    if state.hold_entity.is_none()
        && Duration::from_millis(now_ms as u64) - holding_time.0 > Duration::from_millis(150)
        && holding_time.1.is_some()
    {
        state.hold_entity = holding_time.1;
    }

    if buttons.just_released(MouseButton::Left) {
        *holding_time = (Duration::new(0, 0), None);
        state.hold_entity = None;
        state.entity_to_resize = None;
    }
}

fn update_rectangle_position(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut node_position: Query<(&mut Style, &VeloNodeContainer), With<VeloNodeContainer>>,
    state: Res<UiState>,
    mut query: Query<(&Style, &LeftPanel), Without<VeloNodeContainer>>,
    mut events: EventWriter<RedrawArrow>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = windows.single();
    for event in cursor_moved_events.iter() {
        for (mut style, top) in &mut node_position.iter_mut() {
            if Some(top.id) == state.hold_entity {
                let size = query.single_mut().0.size;
                if let (Val::Percent(x), Val::Px(element_width)) = (size.width, style.size.width) {
                    let width = (primary_window.width() * x) / 100.;
                    style.position.left = Val::Px(event.position.x - width - element_width / 2.);
                }
                if let Val::Px(element_height) = style.size.height {
                    style.position.bottom = Val::Px(event.position.y - element_height / 2.);
                }
                events.send(RedrawArrow { id: top.id });
            }
        }
    }
}

fn create_new_rectangle(
    mut commands: Commands,
    mut events: EventReader<AddRect>,
    state: ResMut<StaticState>,
    mut ui_state: ResMut<UiState>,
) {
    for event in events.iter() {
        let font = state.font.as_ref().unwrap().clone();
        *ui_state = UiState::default();
        ui_state.entity_to_edit = Some(ReflectableUuid(event.node.id));
        let entity = spawn_node(
            &mut commands,
            NodeMeta {
                font,
                size: (event.node.width, event.node.height),
                id: ReflectableUuid(event.node.id),
                image: event.image.clone(),
                text: event.node.text.text.clone(),
                bg_color: event.node.bg_color,
                position: (event.node.left, event.node.bottom),
                text_pos: event.node.text.pos.clone(),
                tags: event.node.tags.clone(),
                z_index: event.node.z_index,
            },
        );
        commands.entity(state.main_panel.unwrap()).add_child(entity);
    }
}

fn resize_notificator(mut commands: Commands, resize_event: Res<Events<WindowResized>>) {
    let mut reader = resize_event.get_reader();
    for _ in reader.iter(&resize_event) {
        commands.insert_resource(LoadRequest {
            doc_id: None,
            drop_last_checkpoint: false,
        });
    }
}

fn higlight_event_handler(
    mut app_state: ResMut<AppState>,
    mut docs: Query<(&mut BackgroundColor, &DocListItemContainer), With<DocListItemContainer>>,
    mut tabs: Query<
        (&mut BackgroundColor, &TabContainer),
        (With<TabContainer>, Without<DocListItemContainer>),
    >,
) {
    let current_doc = app_state.current_document.unwrap();

    for (mut bg_color, i) in &mut docs.iter_mut() {
        if current_doc == i.id {
            bg_color.0 = Color::ALICE_BLUE;
        } else {
            bg_color.0 = Color::rgba(0.8, 0.8, 0.8, 0.5)
        }
    }

    if let Some(doc) = app_state.docs.get_mut(&current_doc) {
        let tab = doc.tabs.iter().find(|x| x.is_active).unwrap();
        for (mut bg_color, i) in &mut tabs.iter_mut() {
            if tab.id == i.id {
                bg_color.0 = Color::ALICE_BLUE;
            } else {
                bg_color.0 = Color::rgba(0.8, 0.8, 0.8, 0.5)
            }
        }
    }
}
