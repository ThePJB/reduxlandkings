use crate::rect::*;
use crate::gun::*;
use crate::level::*;
use crate::kmath::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntityKind {
    Player,
    WalkerShooter,
    RunnerGunner,
    Chungus,
    Bullet,
    
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntityTeam {
    Player,
    Enemy,
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
    pub team: EntityTeam,

    pub health: f32,
    pub max_health: f32,
    
    pub damage: f32,

}

impl Entity {
    pub fn new(kind: EntityKind, pos: Vec2) -> Entity {
        let side_length = match kind {
            EntityKind::Player => 0.05,
            EntityKind::WalkerShooter |
            EntityKind::RunnerGunner => 0.05,
            EntityKind::Chungus => 0.1,
            EntityKind::Bullet => 0.02,
        };
        let gun = match kind {
            EntityKind::Player => {Gun::new_burstrifle()},
            EntityKind::WalkerShooter => {Gun {
                bullet_speed: 0.5,
                damage: 0.5,
                cooldown: 1.1,
                random_spread: 0.05,
                bullets_per_shot: 1,
                spread: 0.0,
                action: Action::Auto,

                state: GunState::new(),
            }}
            EntityKind::RunnerGunner => {Gun {
                bullet_speed: 0.45,
                damage: 0.5,
                cooldown: 0.05,
                random_spread: 0.2,
                bullets_per_shot: 1,
                spread: 0.0,
                action: Action::Burst(5, 1.5),

                state: GunState::new(),
            }}
            EntityKind::Chungus => {Gun {
                bullet_speed: 0.35,
                damage: 0.5,
                cooldown: 0.1,
                random_spread: 0.3,
                bullets_per_shot: 5,
                spread: 0.5,
                action: Action::Burst(4, 4.0),

                state: GunState::new(),
            }}
            _ => {Gun::new()}
        };
        let speed = match kind {
            EntityKind::Player => 0.6,
            EntityKind::WalkerShooter => 0.2,
            EntityKind::RunnerGunner => 0.3,
            EntityKind::Chungus => 0.15,
            _ => 0.0,
        };
        let team = match kind {
            EntityKind::Player => EntityTeam::Player,
            _ => EntityTeam::Enemy,
        };
        let health = match kind {
            EntityKind::Chungus => 8.0,
            EntityKind::Player => 4.0,
            EntityKind::RunnerGunner |
            EntityKind::WalkerShooter => 4.0,
            _ => 1.0,
        };

        Entity {
            aabb: Rect::new(pos.x - side_length/2.0, pos.y - side_length/2.0, side_length, side_length),
            kind: kind,
            velocity: Vec2::new(0.0, 0.0),
            gun: gun,
            want_shoot: false,
            previous_shoot_dir: Vec2::new(1.0, 0.0),
            owner: 123123, // sentinel
            health: health,
            max_health: health,
            damage: 0.0,
            speed: speed,
            team: team,
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

    pub fn with_team(mut self, team: EntityTeam) -> Entity {
        self.team = team;
        self
    }

    pub fn with_damage(mut self, damage: f32) -> Entity {
        self.damage = damage;
        self
    }

    pub fn think(&self, self_id: u32, level: &Level, commands: &mut Vec<EntityCommand>, t: f32) {
        match self.kind {
            EntityKind::Chungus |
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

                    let coward_radius = if self.gun.on_burst_cooldown(t) {
                        0.4
                    } else {
                        0.25
                    };
                    let neutral_radius = 0.05;

                    // Moving
                    if dvec.magnitude() < 1.0 && level.raycast(this_pos, target_pos).is_none() {
                        commands.push(EntityCommand::Move(self_id, match dvec.magnitude() {
                            m if m > coward_radius + neutral_radius => dvec.normalize(),
                            m if m > coward_radius => Vec2::new(0.0, 0.0),
                            _ => -dvec.normalize(),
                        }));
                    } else {
                        commands.push(EntityCommand::Move(self_id, Vec2::new(0.0, 0.0)));
                    }

                    // Shooting
                    if dvec.magnitude() < coward_radius + neutral_radius && dvec.magnitude() > coward_radius  && level.raycast(this_pos, target_pos).is_none() {
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