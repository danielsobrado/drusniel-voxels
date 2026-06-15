use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use crate::camera::controller::{spawn_camera, player_camera_system};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (spawn_camera, lock_cursor_on_start))
            .add_systems(Update, player_camera_system);
    }
}

fn lock_cursor_on_start(mut windows: Query<(&mut Window, &mut CursorOptions)>) {
    if let Ok((_window, mut cursor_options)) = windows.single_mut() {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

