use crate::rect::*;
use crate::gun::*;
use glam::*;

#[derive(Debug)]
pub enum EntityKind {
    Player,
    WalkerShooter,

}

pub enum EntityCommand {
    Move(u32, f32, f32),
    Shoot(u32, f32, f32),
}

#[derive(Debug)]
pub struct Entity {
    pub kind: EntityKind,
    pub aabb: Rect,
    pub velocity: Vec2,
    pub gun: Gun,

}

impl Entity {
    pub fn new(kind: EntityKind, pos: Vec2) -> Entity {
        let side_length = match kind {
            EntityKind::Player => 0.05,
            EntityKind::WalkerShooter => 0.05,
        };

        Entity {
            aabb: Rect::new(pos.x - side_length/2.0, pos.y - side_length/2.0, side_length, side_length),
            kind: kind,
            velocity: Vec2::new(0.0, 0.0),
            gun: Gun::new(),
        }
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Entity {
        self.velocity = velocity;
        self
    }
}