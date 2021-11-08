use crate::kmath::*;
use rand::prelude::*;
use crate::entity::*;

#[derive(Debug)]
pub struct GunState {
    pub last_shot: f32,
    pub repeat: bool,
}

impl GunState {
    pub fn new() -> GunState {
        GunState {
            repeat: false,
            last_shot: -10000.0,
        }
    }
}

#[derive(Debug)]
pub struct Gun {
    pub damage: f32,
    pub cooldown: f32,
    pub bullet_speed: f32,
    pub random_spread: f32,
    
    pub bullets_per_shot: i32,
    pub spread: f32,
    pub auto: bool,



    pub state: GunState,

}

impl Gun {
    pub fn new() -> Gun {
        Gun {
            auto: true,
            damage: 1.0,
            cooldown: 0.5,
            bullet_speed: 2.0,
            random_spread: 0.01,
            
            bullets_per_shot: 1,
            spread: 0.0,

            state: GunState::new(),
        }
    }
    pub fn new_machinegun() -> Gun {
        Gun {
            auto: true,
            damage: 1.0,
            cooldown: 0.05,
            bullet_speed: 1.5,
            random_spread: 0.01,
            bullets_per_shot: 3,
            spread: 0.5,

            state: GunState::new(),
        }
    }

    pub fn new_shotgun() -> Gun {
        Gun {
            auto: false,
            damage: 1.0,
            cooldown: 1.0,
            bullet_speed: 1.3,
            random_spread: 0.1,
            bullets_per_shot: 5,
            spread: 0.8,

            state: GunState::new(),
        }
    }

    // returns whether it does a shoot this frame
    // instead of the bool it could return the bullet entity, whatever
    pub fn update(&mut self, squeeze: bool, dt: f32, t: f32) -> bool {
        if squeeze {
            if !self.auto && self.state.repeat {
                return false;
            }
            self.state.repeat = true;

            if t - self.state.last_shot > self.cooldown {
                self.state.last_shot = t;
                return true;
            }
        } else {
            self.state.repeat = false;
        }
        return false;
    }

    pub fn make_bullets(&self, bullets: &mut Vec<Entity>, pos: Vec2, dir: Vec2, owner: u32) {
        for i in 0..self.bullets_per_shot {
            let idx = i - self.bullets_per_shot/2; // for 1: 0 for 3: -1 etc
            let spread_float = self.spread * idx as f32 / self.bullets_per_shot as f32;
            let spread_dir = dir.rotate(spread_float);

            let adjusted_dir = spread_dir.spread(self.random_spread);

            bullets.push(Entity::new(EntityKind::Bullet, pos)
                            .with_velocity(adjusted_dir * self.bullet_speed)
                            .with_owner(owner)
                            .with_damage(self.damage));
        }
    }
}