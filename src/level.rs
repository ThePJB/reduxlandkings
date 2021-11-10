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

            let entity_kinds = vec!(EntityKind::WalkerShooter, EntityKind::RunnerGunner, EntityKind::Chungus);

            level.entities.insert(rand::thread_rng().gen(), Entity::new(
                entity_kinds[rand::thread_rng().gen_range(0..entity_kinds.len())], 
                Vec2::new(walker_pos_x, walker_pos_y)));
        }

        level
    }

    pub fn apply_command(&mut self, command: EntityCommand) {
        match command {
            EntityCommand::Move(id, dir) => {
                if let Some(ent) = self.entities.get_mut(&id) {
                    ent.velocity = ent.speed * dir;
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

    pub fn raycast(&self, ray_origin: Vec2, ray_destination: Vec2) -> Option<Vec2> {
        let round_up = |u: f32, side_length: f32| {
            (u/side_length).ceil() * side_length
        };
        let round_down = |u: f32, side_length: f32| {
            (u/side_length).floor() * side_length
        };
        let bound = |u, sign: i32, side_length| {
            if sign >= 0 {
                let ru = round_up(u, side_length);
                if ru == u { side_length } else {ru - u}
            } else {
                let ru = round_down(u, side_length);
                if ru == u { side_length } else {u - ru}
            }
        };

        let mut grid_x = (ray_origin.x / self.grid_size) as i32;
        let mut grid_y = (ray_origin.y / self.grid_size) as i32;

        let grid_dest_x = (ray_destination.x / self.grid_size) as i32;
        let grid_dest_y = (ray_destination.y / self.grid_size) as i32;

        let delta_vec = ray_destination - ray_origin;
        let ray_dir = delta_vec.normalize();

        // increment these
        let mut actual_march_x: f32 = ray_origin.x;
        let mut actual_march_y: f32 = ray_origin.y;

        let sign_x = if delta_vec.x > 0.0 { 1 } else { -1 };
        let sign_y = if delta_vec.y > 0.0 { 1 } else { -1 };

        // cycle through these
        let side_length = self.grid_size; // should just be elems
        let mut next_tile_in_x: f32 = bound(actual_march_x, sign_x, side_length);
        let mut next_tile_in_y: f32 = bound(actual_march_y, sign_y, side_length);

        let mut n = 0;
        loop {
            if n > 9999 { 
                panic!("raycast infinite loop");
                println!("bailing");
                return None; 
            }
            n += 1;
            // might be a bit inefficient, checking same thing repeatedly, dont care its more readable rn
            // check to terminate (wall strike)
            if self.tiles[(grid_x*self.side_length as i32 + grid_y) as usize] == Tile::Wall {
                return Some(Vec2::new(actual_march_x, actual_march_y));
            }

            if grid_x == grid_dest_x && grid_y == grid_dest_y {
                return None;
            }

            let x_distance = bound(actual_march_x, sign_x, side_length);
            let y_distance = bound(actual_march_y, sign_y, side_length);

            let x_want = (x_distance / ray_dir.x).abs();
            let y_want = (y_distance / ray_dir.y).abs();
            
            let (x_to_march, y_to_march) = // this msut be wrong
                if x_want <= y_want {
                    let x_to_march = x_distance;
                    let y_to_march = ray_dir.div_scalar(ray_dir.x).mul_scalar(x_distance).y;
                    (x_to_march.abs(), y_to_march.abs())
                } else {
                    let y_to_march = y_distance;
                    let x_to_march = ray_dir.div_scalar(ray_dir.y).mul_scalar(y_distance).x;
                    (x_to_march.abs(), y_to_march.abs())
                };

            // march the ray
            actual_march_x += x_to_march * sign_x as f32;
            actual_march_y += y_to_march * sign_y as f32;

            // calculate grid update
            next_tile_in_x -= x_to_march;
            if next_tile_in_x <= 0.0 {
                next_tile_in_x += side_length;
                grid_x += sign_x;
            }
            next_tile_in_y -= y_to_march;
            if next_tile_in_y <= 0.0 {
                next_tile_in_y += side_length;
                grid_y += sign_y;
            }
        }
    }
}
