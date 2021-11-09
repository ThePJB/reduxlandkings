use crate::kmath::*;
use rand::prelude::*;
use crate::entity::*;

#[derive(Debug)]
pub struct GunState {
    pub last_shot: f32,
    pub last_burst: f32,
    pub burst_count: i32,
    pub repeat: bool,
    pub compelled: bool,
}

impl GunState {
    pub fn new() -> GunState {
        GunState {
            repeat: false,
            compelled: false,
            burst_count: 0,
            last_shot: -10000.0,
            last_burst: -10000.0,
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
    
    pub action: Action,
    
    pub state: GunState,
    
}

#[derive(Debug)]
pub enum Action {
    Semi,
    Burst(i32, f32),    // semi burst auto burst?
    Auto,
}

impl Gun {
    pub fn new() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 0.5,
            bullet_speed: 2.0,
            random_spread: 0.01,
            
            bullets_per_shot: 1,
            spread: 0.0,
            action: Action::Semi,

            state: GunState::new(),
        }
    }
    pub fn new_machinegun() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 0.05,
            bullet_speed: 1.5,
            random_spread: 0.01,
            bullets_per_shot: 3,
            spread: 0.5,
            action: Action::Auto,

            state: GunState::new(),
        }
    }
    pub fn new_burstrifle() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 0.02,
            bullet_speed: 1.5,
            random_spread: 0.01,
            bullets_per_shot: 1,
            spread: 0.0,
            action: Action::Burst(3, 0.33),

            state: GunState::new(),
        }
    }

    pub fn new_shotgun() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 0.7,
            bullet_speed: 1.3,
            random_spread: 0.1,
            bullets_per_shot: 5,
            spread: 0.7,
            action: Action::Semi,

            state: GunState::new(),
        }
    }

    // will the gun shoot this frame?
    pub fn will_shoot(&self, squeeze: bool, t: f32) -> bool {
        // no shoot due to cooldown
        if t - self.state.last_shot < self.cooldown {
            return false;
        }

        match self.action {
            Action::Semi => {
                squeeze && !self.state.repeat
            },
            Action::Burst(max, burst_cooldown) => {
                if self.state.burst_count >= max || self.state.burst_count == 0 {
                    squeeze && t - self.state.last_burst > burst_cooldown
                } else {
                    true
                }
            },
            Action::Auto => {
                squeeze
            },
        }
    }

    pub fn update(&mut self, squeeze: bool, did_shoot: bool, t: f32) {
        if !squeeze {
            self.state.repeat = false;
        }

        if squeeze && did_shoot {
            self.state.repeat = true;
        }

        if did_shoot {
            self.state.last_shot = t;

            match self.action {
                Action::Semi => {},
                Action::Burst(max, burst_cooldown) => {
                    // reset burst?
                    if self.state.burst_count >= max {
                        self.state.burst_count = 0;
                    }

                    // exceed burst?
                    self.state.burst_count += 1;
                    if self.state.burst_count >= max {
                        self.state.last_burst = t;
                    }

                },
                Action::Auto => {},
            }
        }
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