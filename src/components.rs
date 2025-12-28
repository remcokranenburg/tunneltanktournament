use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub id: usize,
}

#[derive(Component)]
pub struct CameraPosition {
    pub pos: UVec2,
}

#[derive(Component)]
pub struct Bullet {
    pub owner_id: usize,
}

#[derive(Component, Clone, Copy)]
pub struct BulletReady(pub bool);

#[derive(Component, Clone, Copy)]
pub struct MoveDir(pub Vec2);
