use bevy::prelude::*;

use crate::camera::CameraPlugin;
use crate::map::MapPlugin;
use crate::units::UnitsPlugin;
use crate::hud::HudPlugin;
use crate::input::InputPlugin;

/// Root plugin composing all per-domain sub-plugins.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        app.add_plugins((
            CameraPlugin,
            MapPlugin,
            UnitsPlugin,
            HudPlugin,
            InputPlugin,
        ));

        // Mode-specific plugins
        #[cfg(feature = "native")]
        app.add_plugins(crate::local_match::LocalMatchPlugin);
    }
}

/// Top-level application state.
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    PostGame,
}

/// System ordering sets for the game loop.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystems {
    /// FixedUpdate: engine tick, state sync
    Simulation,
    /// Update: mouse/keyboard handling
    Input,
    /// Update: spawn/update/despawn ECS entities from game state
    EntitySync,
    /// Update: camera, visual effects, HUD
    Render,
}
