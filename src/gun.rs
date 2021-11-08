#[derive(Debug)]
pub struct Gun {
    pub damage: f32,
    pub cooldown: f32,
    pub last_shot: f32,
    pub bullet_speed: f32,
    // maybe a bool automatic
}

impl Gun {
    pub fn new() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 0.5,
            last_shot: -1000.0,
            bullet_speed: 2.0,
        }
    }
    pub fn new_machinegun() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 0.05,
            last_shot: -1000.0,
            bullet_speed: 1.5,
        }
    }

    // returns whether it does a shoot this frame
    // instead of the bool it could return the bullet entity, whatever
    pub fn update(&mut self, squeeze: bool, dt: f32, t: f32) -> bool {
        if squeeze {
            if t - self.last_shot > self.cooldown {
                self.last_shot = t;
                return true;
            }
        }
        return false;
    }
}