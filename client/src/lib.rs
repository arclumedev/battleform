use bevy::prelude::*;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

mod renderer;
mod state;

use state::*;

// --- Cross-boundary state (JS pushes data in, Bevy systems drain it) ---

static PENDING_DIFFS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
static PENDING_SNAPSHOTS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

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
        .insert_resource(ClearColor(Color::srgb(0.08, 0.10, 0.18)))
        .init_resource::<GameStateView>()
        .init_resource::<EntityMap>()
        .add_systems(Startup, renderer::setup_camera)
        .add_systems(
            Update,
            (
                drain_pending_updates,
                renderer::sync_entities,
                renderer::camera_controls,
            )
                .chain(),
        )
        .run();
}

// --- Systems ---

fn drain_pending_updates(mut state: ResMut<GameStateView>) {
    if let Ok(mut snapshots) = PENDING_SNAPSHOTS.lock() {
        for data in snapshots.drain(..) {
            match rmp_serde::from_slice::<GameStateView>(&data) {
                Ok(new_state) => {
                    web_sys::console::log_1(
                        &format!(
                            "[wasm] Snapshot: {}x{}, {} terrain rows, {} units, {} buildings",
                            new_state.map_width,
                            new_state.map_height,
                            new_state.terrain.len(),
                            new_state.units.len(),
                            new_state.buildings.len()
                        )
                        .into(),
                    );
                    *state = new_state;
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!("[wasm] MsgPack failed: {:?}", e).into(),
                    );
                    match serde_json::from_slice::<GameStateView>(&data) {
                        Ok(new_state) => {
                            *state = new_state;
                        }
                        Err(e2) => {
                            web_sys::console::error_1(
                                &format!("[wasm] Both failed: {:?} / {:?}", e, e2).into(),
                            );
                        }
                    }
                }
            }
        }
    }

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
