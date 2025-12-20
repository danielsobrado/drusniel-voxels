use crate::camera::controller::{apply_taa_capabilities, player_camera_system, spawn_camera};
use crate::rendering::capabilities::{GraphicsCapabilities, GraphicsDetectionSet};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_camera, lock_cursor_on_start)
                .chain()
                .after(GraphicsDetectionSet),
        )
        .add_systems(
            Update,
            (
                player_camera_system,
                apply_taa_capabilities.run_if(resource_changed::<GraphicsCapabilities>()),
            ),
        );
    }
}

fn lock_cursor_on_start(mut windows: Query<(&mut Window, &mut CursorOptions)>) {
    if let Ok((_window, mut cursor_options)) = windows.single_mut() {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}
