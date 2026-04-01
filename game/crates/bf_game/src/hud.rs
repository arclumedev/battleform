use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, _app: &mut App) {
        // HUD systems will be added here:
        // - Tooltip overlay
        // - Tick counter
        // - Player resource display
        // - Unit selection indicators
    }
}
