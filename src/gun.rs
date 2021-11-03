#[derive(Debug)]
pub struct Gun {
    pub damage: f32,
    pub cooldown: f32,
    pub last_shot: f32,
}

impl Gun {
    pub fn new() -> Gun {
        Gun {
            damage: 1.0,
            cooldown: 1.0,
            last_shot: 0.0,
        }
    }

    // returns whether it does a shoot this frame
    // instead of the bool it could return the bullet entity, whatever
    pub fn update(&mut self, squeeze: bool, dt: f32, t: f32) -> bool {
        self.last_shot = t;
        squeeze
    }
}