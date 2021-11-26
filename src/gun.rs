use crate::kmath::*;
use rand::prelude::*;
use crate::entity::*;
use crate::rect::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum GunTrait {
    Manual,       // Semiauto, +accuracy
    Rapidfire,    // - cooldown - damage
    SprayNPray,   // - cooldown - accuracy
    Hungry,       // - ammo - cooldown
    DoubleBurst,  // burst 2 - burst cooldown
    HeavyBurst,   // + burst size - burst cooldown

    Shotgun,      // 5 shot - damage
    Sawnoff,      // + quanity + spread + randomspread
    
    TripleShot,   // triple shot - ammo

    Marksman,     // + speed + damage + cooldown + accuracy
}

fn trait_ok(gt: GunTrait, other_traits: &Vec<GunTrait>) -> bool {
    if gt == GunTrait::HeavyBurst {
        return other_traits.contains(&GunTrait::DoubleBurst)
    }
    if gt == GunTrait::Sawnoff {
        return other_traits.contains(&GunTrait::Shotgun)
    }
    true
}

fn random_gun_trait() -> GunTrait {
    match rand::thread_rng().gen_range(0..=9) {
        0 => GunTrait::Manual,
        1 => GunTrait::Rapidfire,
        2 => GunTrait::SprayNPray,
        3 => GunTrait::Hungry,
        4 => GunTrait::DoubleBurst,
        5 => GunTrait::HeavyBurst,
        6 => GunTrait::Shotgun,
        7 => GunTrait::Sawnoff,
        8 => GunTrait::TripleShot,
        9 => GunTrait::Marksman,
        _ => panic!("unreachable"),
    }
}



#[derive(Debug, Clone, Copy)]
pub struct GunState {
    pub last_shot: f32,
    pub last_burst: f32,
    pub burst_count: i32,
    pub repeat: bool,
    pub compelled: bool,
    pub ammo: i32,
}

impl GunState {
    pub fn new(ammo: i32) -> GunState {
        GunState {
            repeat: false,
            compelled: false,
            burst_count: 0,
            last_shot: -10000.0,
            last_burst: -10000.0,
            ammo: ammo,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gun {
    pub damage: f32,
    pub cooldown: f32,
    pub bullet_speed: f32,
    pub random_spread: f32,
    pub bullet_size: f32,
    
    pub bullets_per_shot: i32,

    pub max_ammo: i32,

    pub spread: f32,
    
    pub action: Action,
    
    pub state: GunState,

    pub gun_traits: Vec<GunTrait>,
    
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Semi,
    Burst(i32, f32),    // semi burst auto burst?
    Auto,
}

impl Gun {
    pub fn new_default() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 0.7,
            bullet_speed: 1.0,
            random_spread: 0.05,
            max_ammo: 50,
            bullet_size: 0.02,
            
            bullets_per_shot: 1,
            spread: 0.5,
            action: Action::Auto,
            state: GunState::new(30),
            gun_traits: Vec::new(),
        }
    }

    pub fn apply_trait(&mut self, gun_trait: GunTrait) {
        self.gun_traits.push(gun_trait);
        match gun_trait {
            GunTrait::Manual => {
                self.action = Action::Semi;
                self.random_spread *= 0.8;
                self.cooldown *= 1.1;
            },
            GunTrait::Rapidfire => {
                self.cooldown *= 0.4;
                self.damage *= 0.7;
                self.max_ammo *= 2;
                self.state.ammo *= 2;
            },
            GunTrait::Shotgun => {
                self.damage *= 0.7;
                self.bullets_per_shot += 4;
                self.spread *= 1.5;
                self.random_spread *= 2.0;
            },
            GunTrait::SprayNPray => {
                self.cooldown *= 0.5;
                self.random_spread *= 2.0;
            },
            GunTrait::Hungry => {
                self.state.ammo /= 2;
                self.max_ammo /= 2;
                self.cooldown *= 0.8;
                self.damage *= 1.5;
            },
            GunTrait::DoubleBurst => {
                self.action = Action::Burst(3, self.cooldown);
                self.cooldown *= 0.1;
                self.damage *= 0.7;
            },
            GunTrait::HeavyBurst => {
                match self.action {
                    Action::Burst(amount, cooldown) => {
                        self.action = Action::Burst(amount * 2, cooldown * 2.0);
                    }
                    _ => {panic!("unreachable")},
                };
            },
            GunTrait::TripleShot => {
                self.bullets_per_shot += 2;
                self.state.ammo /= 2;
                self.max_ammo /= 2;
            },
            GunTrait::Sawnoff => {
                self.random_spread *= 2.0;
                self.bullets_per_shot += 2;
                self.spread *= 2.0;
            },
            GunTrait::Marksman => {
                self.damage *= 1.5;
                self.cooldown *= 1.5;
                self.bullet_speed *= 2.0;
                self.random_spread *= 0.5;
            },
        }
    }

    pub fn new(damage: f32, cooldown: f32, bullet_speed: f32, random_spread: f32, ammo: i32) -> Gun {
        Gun {
            damage,
            cooldown,
            bullet_speed,
            random_spread,
            max_ammo: ammo,
            bullet_size: 0.02,
            
            bullets_per_shot: 1,
            spread: 0.0,
            action: Action::Auto,

            state: GunState::new(ammo),
            gun_traits: Vec::new(),
        }
    }
    pub fn with_multishot(mut self, count: i32, arc: f32) -> Gun {
        self.bullets_per_shot = count;
        self.spread = arc;
        self
    }
    pub fn with_burst(mut self, count: i32, cooldown: f32) -> Gun {
        self.action = Action::Burst(count, cooldown);
        self
    }
    pub fn with_semi_auto(mut self) -> Gun {
        self.action = Action::Semi;
        self
    }


    pub fn new_machinegun() -> Gun {
        Gun::new(1.0, 0.05, 1.5, 0.01, 100)
            .with_multishot(3, 0.5)
    }
    pub fn new_burstrifle() -> Gun {
        Gun::new(1.0, 0.02, 1.5, 0.01, 100)
        .with_burst(3, 0.33)
    }
    
    pub fn new_shotgun() -> Gun {
        Gun::new(1.0, 0.7, 1.3, 0.1, 15)
        .with_multishot(5, 0.5)
        .with_semi_auto()
    }

    pub fn new_minigun() -> Gun {
        Gun::new(0.2, 0.01, 1.5, 0.07, 500)
        //.with_burst(40, 2.0)
    }

    pub fn on_cooldown(&self, t: f32) -> bool {
        t - self.state.last_shot < self.cooldown
    }

    pub fn on_burst_cooldown(&self, t: f32) -> bool {
        match self.action {
            Action::Burst(max, burst_cooldown) => {(self.state.burst_count >= max || self.state.burst_count == 0)
                && t - self.state.last_burst > burst_cooldown},
            _ => false,
        }
        
    }

    // will the gun shoot this frame?
    pub fn will_shoot(&self, squeeze: bool, t: f32) -> bool {
        // no shoot due to cooldown
        if self.on_cooldown(t) {
            return false;
        }

        if self.state.ammo <= 0 {
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

            self.state.ammo -= 1;

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

    pub fn make_bullets(&self, bullets: &mut Vec<Entity>, pos: Vec2, dir: Vec2, owner_id: u32, owner_team: EntityTeam) {
        for i in 0..self.bullets_per_shot {
            let idx = i - self.bullets_per_shot/2; // for 1: 0 for 3: -1 etc
            let spread_float = self.spread * idx as f32 / self.bullets_per_shot as f32;
            let spread_dir = dir.rotate(spread_float);

            let adjusted_dir = spread_dir.spread(self.random_spread);

            bullets.push(Entity {
                kind: EntityKind::Bullet,
                aabb: Rect::new_centered(pos.x, pos.y, self.bullet_size, self.bullet_size),
                velocity: adjusted_dir * self.bullet_speed,
                owner: owner_id,
                team: owner_team,
                damage: self.damage,
                gun: Gun::new_default(),
                want_shoot: false,
                health: 1.0,
                max_health: 1.0,
                previous_shoot_dir: Vec2::new(0.0, 0.0),
                speed: 0.0,
            });

            bullets.push(Entity::new(EntityKind::Bullet, pos)
                            .with_velocity(adjusted_dir * self.bullet_speed)
                            .with_owner(owner_id)
                            .with_team(owner_team)
                            .with_damage(self.damage));
        }
    }
}

pub fn generate_gun(num_traits: i32) -> Gun {
    let mut traits = Vec::new();

    for i in 0..num_traits {
        loop {
            let gt = random_gun_trait();
            if trait_ok(gt, &traits) {
                traits.push(gt);
                break;
            }
        }
    }
    traits.sort();

    let mut g = Gun::new_default();
    for gt in traits {
        g.apply_trait(gt);
    }

    return g;
}