use crate::rect::*;
use crate::gun::*;
use crate::level::*;
use crate::kmath::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntityKind {
    Player,
    WalkerShooter,
    RunnerGunner,
    Bullet,

}

pub enum EntityCommand {
    Move(u32, Vec2),
    Shoot(u32, Vec2),
    Unshoot(u32),
}

#[derive(Debug)]
pub struct Entity {
    pub kind: EntityKind,
    pub aabb: Rect,
    pub velocity: Vec2,
    pub speed: f32,

    pub gun: Gun,
    pub want_shoot: bool,
    pub previous_shoot_dir: Vec2,

    pub owner: u32,
    pub health: f32,
    
    pub damage: f32,

}

impl Entity {
    pub fn new(kind: EntityKind, pos: Vec2) -> Entity {
        let entity_scale = 0.8;
        let side_length = match kind {
            EntityKind::Player => 0.05 * entity_scale,
            EntityKind::WalkerShooter |
            EntityKind::RunnerGunner => 0.05 * entity_scale,
            EntityKind::Bullet => 0.02 * entity_scale,
        };
        let gun = match kind {
            EntityKind::Player => {Gun::new_burstrifle()},
            EntityKind::WalkerShooter => {Gun {
                bullet_speed: 0.5,
                damage: 0.5,
                cooldown: 1.0,
                random_spread: 0.05,
                bullets_per_shot: 1,
                spread: 0.0,
                action: Action::Auto,

                state: GunState::new(),
            }}
            EntityKind::RunnerGunner => {Gun {
                bullet_speed: 0.5,
                damage: 0.5,
                cooldown: 0.05,
                random_spread: 0.2,
                bullets_per_shot: 1,
                spread: 0.0,
                action: Action::Burst(5, 1.5),

                state: GunState::new(),
            }}
            _ => {Gun::new()}
        };
        let speed = match kind {
            EntityKind::Player => 0.6,
            EntityKind::WalkerShooter => 0.2,
            EntityKind::RunnerGunner => 0.3,
            _ => 0.0,
        };

        Entity {
            aabb: Rect::new(pos.x - side_length/2.0, pos.y - side_length/2.0, side_length, side_length),
            kind: kind,
            velocity: Vec2::new(0.0, 0.0),
            gun: gun,
            want_shoot: false,
            previous_shoot_dir: Vec2::new(1.0, 0.0),
            owner: 123123, // sentinel
            health: 4.0,
            damage: 0.0,
            speed: speed,
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

    pub fn with_damage(mut self, damage: f32) -> Entity {
        self.damage = damage;
        self
    }

    pub fn think(&self, self_id: u32, level: &Level, commands: &mut Vec<EntityCommand>) {
        match self.kind {
            EntityKind::WalkerShooter => {
                for (target_id, target) in level.entities.iter().filter(|(_, e)| e.kind == EntityKind::Player) {
                    let this_pos = self.aabb.centroid();
                    let target_pos = target.aabb.centroid();
                    let dvec = target_pos - this_pos;
                    if dvec.magnitude() < 1.0 && dvec.magnitude() > 0.4 && level.raycast(this_pos, target_pos).is_none() {
                        commands.push(EntityCommand::Move(self_id, dvec.normalize()));
                    } else {
                        commands.push(EntityCommand::Move(self_id, Vec2::new(0.0, 0.0)));
                    }
                    if dvec.magnitude() < 0.5 && level.raycast(this_pos, target_pos).is_none() {
                        commands.push(EntityCommand::Shoot(self_id, dvec.normalize()));
                    } else {
                        commands.push(EntityCommand::Unshoot(self_id));
                    }
                }
            },
            EntityKind::RunnerGunner => {
                for (target_id, target) in level.entities.iter().filter(|(_, e)| e.kind == EntityKind::Player) {
                    let this_pos = self.aabb.centroid();
                    let target_pos = target.aabb.centroid();
                    let dvec = target_pos - this_pos;

                    // Moving
                    if dvec.magnitude() < 1.0 && level.raycast(this_pos, target_pos).is_none() {
                        commands.push(EntityCommand::Move(self_id, match dvec.magnitude() {
                            m if m > 0.25 => dvec.normalize(),
                            m if m > 0.2 => Vec2::new(0.0, 0.0),
                            _ => -dvec.normalize(),
                        }));
                    } else {
                        commands.push(EntityCommand::Move(self_id, Vec2::new(0.0, 0.0)));
                    }

                    // Shooting
                    if dvec.magnitude() < 0.3 && level.raycast(this_pos, target_pos).is_none() {
                        commands.push(EntityCommand::Shoot(self_id, dvec.normalize()));
                    } else {
                        commands.push(EntityCommand::Unshoot(self_id));
                    }
                }
            },
            _ => {},
        };
    }
}