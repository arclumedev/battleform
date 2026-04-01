use bevy::prelude::*;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

mod renderer;
mod state;

use state::*;

// --- Cross-boundary state (JS pushes data in, Bevy systems drain it) ---

static PENDING_DIFFS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
static PENDING_SNAPSHOTS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

// Camera snapshot for JS tooltip queries
pub struct CameraSnapshot {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub win_w: f32,
    pub win_h: f32,
}

impl Default for CameraSnapshot {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, scale: 1.0, win_w: 800.0, win_h: 600.0 }
    }
}

pub static CAMERA_STATE: Mutex<CameraSnapshot> = Mutex::new(CameraSnapshot {
    x: 0.0, y: 0.0, scale: 1.0, win_w: 800.0, win_h: 600.0,
});

// Game state snapshot for JS tooltip queries
static GAME_STATE_SNAPSHOT: Mutex<Option<GameStateView>> = Mutex::new(None);

#[wasm_bindgen]
pub fn push_state_diff(data: &[u8]) {
    if let Ok(mut pending) = PENDING_DIFFS.lock() {
        pending.push(data.to_vec());
    }
}

#[wasm_bindgen]
pub fn push_full_state(data: &[u8]) {
    if let Ok(mut pending) = PENDING_SNAPSHOTS.lock() {
        pending.push(data.to_vec());
    }
}

// --- Bevy Resources ---

#[derive(Resource, Default)]
pub struct EntityMap {
    pub units: std::collections::HashMap<String, Entity>,
    pub buildings: std::collections::HashMap<String, Entity>,
    pub resources: std::collections::HashMap<String, Entity>,
    pub fog_tiles: Vec<Vec<Entity>>,
    pub terrain_spawned: bool,
}

// --- Entry point ---

#[wasm_bindgen]
pub fn start() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#glcanvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<GameStateView>()
        .init_resource::<EntityMap>()
        .add_systems(Startup, renderer::setup_camera)
        .add_systems(
            Update,
            (
                drain_pending_updates,
                renderer::sync_entities,
                renderer::camera_controls,
                renderer::export_camera_state,
                snapshot_state_for_js,
            )
                .chain(),
        )
        .run();
}

// --- Systems ---

fn drain_pending_updates(mut state: ResMut<GameStateView>) {
    // Process full snapshots first
    if let Ok(mut snapshots) = PENDING_SNAPSHOTS.lock() {
        for data in snapshots.drain(..) {
            match rmp_serde::from_slice::<GameStateView>(&data) {
                Ok(new_state) => {
                    web_sys::console::log_1(&format!(
                        "[wasm] Snapshot: {}x{}, {} terrain rows, {} units, {} buildings",
                        new_state.map_width, new_state.map_height,
                        new_state.terrain.len(), new_state.units.len(), new_state.buildings.len()
                    ).into());
                    *state = new_state;
                }
                Err(e) => {
                    web_sys::console::warn_1(&format!("[wasm] MsgPack failed: {:?}", e).into());
                    match serde_json::from_slice::<GameStateView>(&data) {
                        Ok(new_state) => {
                            web_sys::console::log_1(&format!(
                                "[wasm] JSON snapshot: {}x{}", new_state.map_width, new_state.map_height
                            ).into());
                            *state = new_state;
                        }
                        Err(e2) => {
                            web_sys::console::error_1(&format!(
                                "[wasm] Both failed. MsgPack: {:?}, JSON: {:?}", e, e2
                            ).into());
                        }
                    }
                }
            }
        }
    }

    // Then apply diffs
    if let Ok(mut diffs) = PENDING_DIFFS.lock() {
        for data in diffs.drain(..) {
            if let Ok(diff) = rmp_serde::from_slice::<StateDiff>(&data) {
                state.apply(diff);
            } else if let Ok(diff) = serde_json::from_slice::<StateDiff>(&data) {
                state.apply(diff);
            }
        }
    }
}

fn snapshot_state_for_js(state: Res<GameStateView>) {
    if let Ok(mut snap) = GAME_STATE_SNAPSHOT.lock() {
        *snap = Some(state.clone());
    }
}

/// Called from JS on mousemove. Takes screen-space cursor position,
/// converts to hex coords using the latest camera state, and returns
/// tooltip text (empty string = nothing to show).
#[wasm_bindgen]
pub fn get_tile_info(cursor_x: f32, cursor_y: f32) -> String {
    let cam = match CAMERA_STATE.lock() {
        Ok(c) => CameraSnapshot {
            x: c.x, y: c.y, scale: c.scale, win_w: c.win_w, win_h: c.win_h,
        },
        Err(_) => return String::new(),
    };

    let state = match GAME_STATE_SNAPSHOT.lock() {
        Ok(s) => match s.as_ref() {
            Some(s) => s.clone(),
            None => return String::new(),
        },
        Err(_) => return String::new(),
    };

    // Screen to world
    let screen_center_x = cam.win_w / 2.0;
    let screen_center_y = cam.win_h / 2.0;
    let world_x = cam.x + (cursor_x - screen_center_x) * cam.scale;
    let world_y = cam.y - (cursor_y - screen_center_y) * cam.scale;

    let (col, row) = renderer::pixel_to_hex_pub(world_x, world_y);

    if col < 0 || row < 0 || col >= state.map_width as i32 || row >= state.map_height as i32 {
        return String::new();
    }

    let lines = renderer::tile_info_at(&state, col, row);
    lines.join("\n")
}
