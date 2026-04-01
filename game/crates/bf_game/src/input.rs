use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, _app: &mut App) {
        // Input systems will be added here:
        // - Mouse click unit/building selection
        // - Right-click command issuing
        // - Keyboard shortcuts
        // Camera input (WASD, scroll, drag) is in CameraPlugin
    }
}
