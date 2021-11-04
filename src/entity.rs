use crate::rect::*;
use crate::gun::*;
use glam::*;

#[derive(Debug)]
pub enum EntityKind {
    Player,
    WalkerShooter,
    Bullet,

}

pub enum EntityCommand {
    Move(u32, f32, f32),
    Shoot(u32, f32, f32),
    Unshoot(u32),
}

#[derive(Debug)]
pub struct Entity {
    pub kind: EntityKind,
    pub aabb: Rect,
    pub velocity: Vec2,

    pub gun: Gun,
    pub want_shoot: bool,
    pub previous_shoot_dir: Vec2,

    pub owner: u32,

}

impl Entity {
    pub fn new(kind: EntityKind, pos: Vec2) -> Entity {
        let side_length = match kind {
            EntityKind::Player => 0.05,
            EntityKind::WalkerShooter => 0.05,
            EntityKind::Bullet => 0.02,
        };

        Entity {
            aabb: Rect::new(pos.x - side_length/2.0, pos.y - side_length/2.0, side_length, side_length),
            kind: kind,
            velocity: Vec2::new(0.0, 0.0),
            gun: Gun::new(),
            want_shoot: false,
            previous_shoot_dir: Vec2::new(1.0, 0.0),
            owner: 123123, // sentinel
        }
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Entity {
        self.velocity = velocity;
        self
    }

    pub fn with_owner(mut self, owner: u32) -> Entity {
        self.owner = owner;
        self
    }
}