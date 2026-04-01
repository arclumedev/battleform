use bevy::prelude::*;

use bf_game::game::GamePlugin;
use bf_game::local_match::LocalEngine;
use bf_game::{BevyGameState, EntityMap};
use bf_types::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Battleform".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.10, 0.18)))
        .init_resource::<BevyGameState>()
        .init_resource::<EntityMap>()
        .add_plugins(GamePlugin)
        .add_systems(Startup, start_local_match)
        .run();
}

/// Start a bot-vs-bot local match on startup.
fn start_local_match(mut engine: ResMut<LocalEngine>) {
    let config = MatchConfig {
        map_preset: MapPreset::Duel,
        players: vec![
            PlayerConfig {
                slot: 0,
                kind: PlayerKind::Bot,
                name: "Bot Alpha".to_string(),
            },
            PlayerConfig {
                slot: 1,
                kind: PlayerKind::Bot,
                name: "Bot Beta".to_string(),
            },
        ],
        max_ticks: 2000,
        tick_rate_ms: 100,
    };

    engine.start_match(config);
    bevy::log::info!("[game] Local match started: Bot Alpha vs Bot Beta");
}
