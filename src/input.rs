use crate::{CameraMode, Config, args::Args};
use bevy::{platform::collections::HashMap, prelude::*, window::WindowCloseRequested};
use bevy_ggrs::{LocalInputs, LocalPlayers};

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_FIRE: u8 = 1 << 4;

const KEYS_UP: [KeyCode; 2] = [KeyCode::KeyW, KeyCode::ArrowUp];
const KEYS_DOWN: [KeyCode; 2] = [KeyCode::KeyS, KeyCode::ArrowDown];
const KEYS_LEFT: [KeyCode; 2] = [KeyCode::KeyA, KeyCode::ArrowLeft];
const KEYS_RIGHT: [KeyCode; 2] = [KeyCode::KeyD, KeyCode::ArrowRight];
const KEYS_FIRE: [KeyCode; 2] = [KeyCode::ControlLeft, KeyCode::Enter];

pub fn read_local_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>,
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut input = 0u8;

        if keys.pressed(KEYS_UP[*handle]) {
            input |= INPUT_UP;
        }
        if keys.pressed(KEYS_DOWN[*handle]) {
            input |= INPUT_DOWN;
        }
        if keys.pressed(KEYS_LEFT[*handle]) {
            input |= INPUT_LEFT
        }
        if keys.pressed(KEYS_RIGHT[*handle]) {
            input |= INPUT_RIGHT;
        }
        if keys.pressed(KEYS_FIRE[*handle]) {
            input |= INPUT_FIRE;
        }

        local_inputs.insert(*handle, input);
    }

    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}

pub fn read_unsynced_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<Entity, With<Window>>,
    args: Res<Args>,
    mut camera_mode: ResMut<CameraMode>,
    mut messages: MessageWriter<WindowCloseRequested>,
) {
    if keys.all_pressed([KeyCode::ControlLeft, KeyCode::KeyQ]) {
        // request closing all windows
        for window in windows {
            messages.write(WindowCloseRequested { window });
        }
    }

    if args.debug && keys.just_pressed(KeyCode::Tab) {
        // toggle camera mode
        *camera_mode = camera_mode.next();
    }
}

pub fn direction(input: u8) -> Vec2 {
    let mut dir = Vec2::ZERO;
    if input & INPUT_UP != 0 {
        dir.y += 1.;
    }
    if input & INPUT_DOWN != 0 {
        dir.y -= 1.;
    }
    if input & INPUT_LEFT != 0 {
        dir.x -= 1.;
    }
    if input & INPUT_RIGHT != 0 {
        dir.x += 1.;
    }
    dir.normalize_or_zero()
}

pub fn fire(input: u8) -> bool {
    input & INPUT_FIRE != 0
}
