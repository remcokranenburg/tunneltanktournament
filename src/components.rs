use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub id: usize,
}

#[derive(Component)]
pub struct CameraPosition {
    pub pos: UVec2,
}
