use bevy::prelude::*;

use bf_engine::GameEngine;
use bf_types::*;
use crate::game::GameSystems;
use crate::BevyGameState;

/// Plugin for offline/local play with an embedded engine.
pub struct LocalMatchPlugin;

impl Plugin for LocalMatchPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LocalEngine::default())
            .insert_resource(Time::<Fixed>::from_seconds(0.1)) // 10 ticks/sec
            .add_systems(FixedUpdate, tick_engine.in_set(GameSystems::Simulation))
            .add_systems(
                Update,
                sync_engine_to_view.in_set(GameSystems::EntitySync),
            );
    }
}

/// Bevy resource wrapping the game engine for local play.
#[derive(Resource, Default)]
pub struct LocalEngine(pub Option<GameEngine>);

impl LocalEngine {
    /// Start a new local match with the given config.
    pub fn start_match(&mut self, config: MatchConfig) {
        self.0 = Some(GameEngine::new(config));
    }
}

/// FixedUpdate system: tick the engine at 100ms intervals.
fn tick_engine(mut engine: ResMut<LocalEngine>) {
    if let Some(ref mut e) = engine.0 {
        if !e.is_finished() {
            e.tick();
        }
    }
}

/// Update system: copy engine state to the renderer's BevyGameState resource.
fn sync_engine_to_view(engine: Res<LocalEngine>, mut view: ResMut<BevyGameState>) {
    if let Some(ref e) = engine.0 {
        view.0 = e.full_snapshot();
    }
}
