use std::collections::HashMap;
use crate::entity::*;
use crate::rect::*;
use rand::prelude::*;
use crate::kmath::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Tile {
    Wall,
    Open,
}

pub struct Level {
    pub entities: HashMap<u32, Entity>, 
    pub tiles: Vec<Tile>,
    pub side_length: usize,
    pub grid_size: f32,
}

struct Walker {
    pos: (i32, i32),
    dir: i32,
    alive: bool,
}

impl Level {
    pub fn new() -> Level {
        let side_length = 20;

        let mut level = Level {
            entities: HashMap::new(),
            tiles: vec!(Tile::Wall; side_length*side_length),
            side_length,
            grid_size: 0.15,
        };

        let num_walkers = 20;
        let walk_iters = 20;
        let p_change_dir = 0.3;

        let mut walkers = Vec::new();
        let dirs = [(1, 0), (-1,0), (0, -1), (0, 1)];

        for _ in 0..num_walkers {
            walkers.push(Walker {
                pos: (side_length as i32/2, side_length as i32/2),
                dir: rand::thread_rng().gen_range(0..4),
                alive: true,
            });
        }

        level.tiles[side_length/2 * side_length + side_length/2 as usize] = Tile::Open;
        for _ in 0..walk_iters {
            for w in walkers.iter_mut() {
                if !w.alive {
                    continue;
                }

                // maybe change direction
                if rand::thread_rng().gen_range(0.0..1.0) < p_change_dir {
                    let mut idx = rand::thread_rng().gen_range(0..3);
                    if idx >= w.dir {
                        idx += 1;
                    }
                    w.dir = idx;
                }

                // advance
                // kill instead of going off
                let dir = dirs[w.dir as usize];
                let candidate_pos = (w.pos.0 + dir.0, w.pos.1 + dir.1);
                if candidate_pos.0 <= 0 || candidate_pos.1 <= 0 || 
                        candidate_pos.0 >= side_length as i32-1 || 
                        candidate_pos.1 >= side_length as i32-1 {
                    w.alive = false;
                } else {
                    w.pos = candidate_pos;
                    level.tiles[w.pos.0 as usize * side_length + w.pos.1 as usize] = Tile::Open;
                }
            }
        }
        
        let mut walker_positions: Vec<(i32, i32)> = walkers.iter().map(|w| w.pos).collect::<Vec<(i32, i32)>>();
        walker_positions.sort();
        walker_positions.dedup();
        
        let player_pos = *walker_positions.iter().max_by_key(|(x, y)| {
            let xp = x - side_length as i32/2;
            let yp = y - side_length as i32/2;
            xp*xp+yp*yp
        }).unwrap();
        
        let player_pos_x = player_pos.0 as f32 * level.grid_size + level.grid_size as f32/2.0;
        let player_pos_y = player_pos.1 as f32 * level.grid_size + level.grid_size as f32/2.0;

        level.entities.insert(0, Entity::new(EntityKind::Player, Vec2::new(player_pos_x, player_pos_y)));

        walker_positions.retain(|pos| *pos != player_pos);

        for (x, y) in walker_positions {
            let walker_pos_x = x as f32 * level.grid_size + level.grid_size as f32/2.0;
            let walker_pos_y = y as f32 * level.grid_size + level.grid_size as f32/2.0;
            level.entities.insert(rand::thread_rng().gen(), Entity::new(EntityKind::WalkerShooter, Vec2::new(walker_pos_x, walker_pos_y)));
        }

        level
    }

    pub fn apply_command(&mut self, command: EntityCommand) {
        match command {
            EntityCommand::Move(id, vel) => {
                if let Some(ent) = self.entities.get_mut(&id) {
                    ent.velocity = vel;
                }},
            EntityCommand::Shoot(id, dir) => {
                if let Some(ent) = self.entities.get_mut(&id) {
                ent.want_shoot = true;
                ent.previous_shoot_dir = dir;
            }},
            EntityCommand::Unshoot(id) => {
                if let Some(ent) = self.entities.get_mut(&id) {
                ent.want_shoot = false;
            }},
        }
    }
}
