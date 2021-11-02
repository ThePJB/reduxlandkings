use crate::rect::*;
use glam::*;

pub enum EntityKind {
    Player,
}

pub enum EntityCommand {
    Move(u32, f32, f32),
    Shoot(u32, f32, f32),
}

pub struct Entity {
    pub kind: EntityKind,
    pub aabb: Rect,
    pub velocity: Vec2,

}
