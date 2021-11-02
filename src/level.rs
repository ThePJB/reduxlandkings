use std::collections::HashMap;
use crate::entity::*;
use crate::rect::*;
use rand::prelude::*;
use glam::*;

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
        let side_length = 40;

        let mut level = Level {
            entities: HashMap::new(),
            tiles: vec!(Tile::Wall; side_length*side_length),
            side_length,
            grid_size: 0.2,
        };

        let num_walkers = 40;
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

        let (player_walker_i, player_walker) = walkers.iter()
        .enumerate()
        .max_by_key(|(i, w1)| {
            let x = w1.pos.0 - side_length as i32/2;
            let y = w1.pos.1 - side_length as i32/2;
            x*x+y*y
        }).unwrap();

        let player_pos_x = player_walker.pos.0 as f32 * level.grid_size + level.grid_size as f32/2.0;
        let player_pos_y = player_walker.pos.1 as f32 * level.grid_size + level.grid_size as f32/2.0;

        level.entities.insert(0, Entity {
            kind: EntityKind::Player, 
            aabb: Rect::new(player_pos_x, player_pos_y, 0.05, 0.05),
            velocity: Vec2::new(0.0, 0.0),
        });

        level
    }
}
