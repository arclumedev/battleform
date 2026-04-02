use bevy::prelude::*;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod camera;
pub mod game;
pub mod hud;
pub mod input;
#[cfg(feature = "native")]
pub mod local_match;
pub mod map;
pub mod render_utils;
pub mod state;
pub mod units;

use state::*;

// --- Bevy Resource wrapper for GameStateView ---
// bf_types doesn't depend on Bevy, so we wrap it to add the Resource derive.

#[derive(Resource, Default)]
pub struct BevyGameState(pub GameStateView);

impl Deref for BevyGameState {
    type Target = GameStateView;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BevyGameState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// --- Cross-boundary state (JS pushes data in, Bevy systems drain it) ---

static PENDING_DIFFS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
static PENDING_SNAPSHOTS: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn push_state_diff(data: &[u8]) {
    if let Ok(mut pending) = PENDING_DIFFS.lock() {
        pending.push(data.to_vec());
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
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

// --- Logging helpers ---

macro_rules! log {
    ($($arg:tt)*) => {
        #[cfg(feature = "wasm")]
        web_sys::console::log_1(&format!($($arg)*).into());
        #[cfg(not(feature = "wasm"))]
        bevy::log::info!($($arg)*);
    };
}

#[allow(unused_macros)]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "wasm")]
        web_sys::console::warn_1(&format!($($arg)*).into());
        #[cfg(not(feature = "wasm"))]
        bevy::log::warn!($($arg)*);
    };
}

#[allow(unused_macros)]
macro_rules! error_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "wasm")]
        web_sys::console::error_1(&format!($($arg)*).into());
        #[cfg(not(feature = "wasm"))]
        bevy::log::error!($($arg)*);
    };
}

#[allow(unused_imports)]
pub(crate) use error_log;
pub(crate) use log;
#[allow(unused_imports)]
pub(crate) use warn_log;

// --- WASM Entry point ---

#[cfg(feature = "wasm")]
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
        .init_resource::<BevyGameState>()
        .init_resource::<EntityMap>()
        .add_plugins(game::GamePlugin)
        .add_systems(Update, drain_pending_updates.before(game::GameSystems::EntitySync))
        .run();
}

// --- Systems ---

#[cfg(feature = "wasm")]
fn drain_pending_updates(mut state: ResMut<BevyGameState>) {
    if let Ok(mut snapshots) = PENDING_SNAPSHOTS.lock() {
        for data in snapshots.drain(..) {
            match rmp_serde::from_slice::<GameStateView>(&data) {
                Ok(new_state) => {
                    log!(
                        "[game] Snapshot: {}x{}, {} terrain rows, {} units, {} buildings",
                        new_state.map_width,
                        new_state.map_height,
                        new_state.terrain.len(),
                        new_state.units.len(),
                        new_state.buildings.len()
                    );
                    state.0 = new_state;
                }
                Err(e) => {
                    warn_log!("[game] MsgPack failed: {:?}", e);
                    match serde_json::from_slice::<GameStateView>(&data) {
                        Ok(new_state) => {
                            state.0 = new_state;
                        }
                        Err(e2) => {
                            error_log!("[game] Both failed: {:?} / {:?}", e, e2);
                        }
                    }
                }
            }
        }
    }

    if let Ok(mut diffs) = PENDING_DIFFS.lock() {
        for data in diffs.drain(..) {
            if let Ok(diff) = rmp_serde::from_slice::<StateDiff>(&data) {
                state.0.apply(diff);
            } else if let Ok(diff) = serde_json::from_slice::<StateDiff>(&data) {
                state.0.apply(diff);
            }
        }
    }
}
