use glam::Vec3;
use std::collections::HashMap;
use crate::entity::*;
use crate::rect::*;
use rand::prelude::*;
use crate::kmath::*;
use crate::map_gen::*;
use crate::map_fragment::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Tile {
    pub walkable: bool,
    pub overhang: bool,
    pub underhang: bool,
    pub edge: bool,
}

#[derive(Debug)]
pub struct LevelGenParams {
    side_length: i32,
    num_walkers: i32,
    walk_iters: i32,
    p_change_dir: f32,
}

impl LevelGenParams {
    pub fn new() -> LevelGenParams {
        LevelGenParams {
            side_length: 20,
            num_walkers: 20,
            walk_iters: 20,
            p_change_dir: 0.3,
        }
    }
    pub fn new_rand() -> LevelGenParams {
        let mut rng = rand::thread_rng();
        LevelGenParams {
            side_length: rng.gen_range(15..50),
            num_walkers: rng.gen_range(15..50),
            walk_iters: rng.gen_range(5..40),
            p_change_dir: rng.gen_range(0.1..0.5),
        }
    }
}

pub struct Level {
    pub entities: HashMap<u32, Entity>, 
    pub tiles: Vec<Tile>,
    pub side_length: usize,
    pub grid_size: f32,

    pub floor_colour: Vec3,
    pub wall_colour: Vec3,
}

struct Walker {
    pos: (i32, i32),
    dir: i32,
    alive: bool,
}

impl Level {
    pub fn new(mut player: Entity) -> Level {
        let params = LevelGenParams::new_rand();
        println!("Level gen params: {:?}", params);

        let start_tile = Tile {walkable: false, overhang: false, underhang: false, edge: false};

        let mut level = Level {
            entities: HashMap::new(),
            tiles: vec!(start_tile; (params.side_length*params.side_length) as usize),
            side_length: params.side_length as usize,
            grid_size: 0.2,
            floor_colour: Vec3::new(0.75, 0.75, 0.5),
            wall_colour: Vec3::new(0.2, 0.2, 0.4),

        };

        let mut walkers = Vec::new();
        let dirs = [(1, 0), (-1,0), (0, -1), (0, 1)];

        for _ in 0..params.num_walkers {
            walkers.push(Walker {
                pos: (params.side_length/2, params.side_length/2),
                dir: rand::thread_rng().gen_range(0..4),
                alive: true,
            });
        }

        level.tiles[(params.side_length/2 * params.side_length + params.side_length/2) as usize].walkable = true;
        for _ in 0..params.walk_iters {
            for w in walkers.iter_mut() {
                if !w.alive {
                    continue;
                }

                // maybe change direction
                if rand::thread_rng().gen_range(0.0..1.0) < params.p_change_dir {
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
                        candidate_pos.0 >= params.side_length-1 || 
                        candidate_pos.1 >= params.side_length-1 {
                    w.alive = false;
                } else {
                    w.pos = candidate_pos;
                    level.tiles[(w.pos.0 * params.side_length + w.pos.1) as usize].walkable = true;
                }
            }
        }
        
        let mut walker_positions: Vec<(i32, i32)> = walkers.iter().map(|w| w.pos).collect::<Vec<(i32, i32)>>();
        walker_positions.sort();
        walker_positions.dedup();
        
        let player_pos = *walker_positions.iter().max_by_key(|(x, y)| {
            let xp = x - params.side_length/2;
            let yp = y - params.side_length/2;
            xp*xp+yp*yp
        }).unwrap();
        
        let player_pos_x = player_pos.0 as f32 * level.grid_size + level.grid_size as f32/2.0;
        let player_pos_y = player_pos.1 as f32 * level.grid_size + level.grid_size as f32/2.0;

        player.aabb.x = player_pos_x - player.aabb.w/2.0;
        player.aabb.y = player_pos_y - player.aabb.h/2.0;

        level.entities.insert(0, player);

        walker_positions.retain(|pos| *pos != player_pos);

        for (x, y) in walker_positions {
            let walker_pos_x = x as f32 * level.grid_size + level.grid_size as f32/2.0;
            let walker_pos_y = y as f32 * level.grid_size + level.grid_size as f32/2.0;

            let entity_kinds = vec!(EntityKind::WalkerShooter, EntityKind::RunnerGunner, EntityKind::Chungus, EntityKind::GunPickup);

            level.entities.insert(rand::thread_rng().gen(), Entity::new(
                entity_kinds[rand::thread_rng().gen_range(0..entity_kinds.len())], 
                Vec2::new(walker_pos_x, walker_pos_y)));
        }

        // CA pass
        for i in 0..level.side_length as i32 {
            for j in 0..level.side_length as i32 {
                let maybe_tile_below = level.get_tile(i, j+1);
                let maybe_tile_above = level.get_tile(i, j-1);
                let mut this_tile = level.get_tile_mut(i, j).unwrap();

                if this_tile.walkable {
                    match maybe_tile_above {
                        Some(above) if !above.walkable => {this_tile.underhang = true},
                        _ => {},
                    }
                    match maybe_tile_below {
                        Some(below) if !below.walkable => {this_tile.overhang = true},
                        _ => {},
                    }
                } else {
                    match maybe_tile_below {
                        Some(below) if below.walkable => {this_tile.edge = true},
                        _ => {},
                    }
                }
            }
        }

        level
    }

    pub fn new_dla(mut player: Entity, seed: u32) -> Level {

        let w = 30;
        let h = 30;
        let (player_pos, basic_level) = gen_ca2(w, h, seed);
        /*
        let w = 30;
        let h = 30;

        // let basic_level = gen_dla(w, h, 600, seed);
        // let basic_level = gen_ca(w, h, 0.6, 10, seed);
        // let mut basic_level = MapFragment::new(w,h).dla(1000, seed);
        let dla = MapFragment::new(w,h).dla(1000,seed+2);
        let ca = MapFragment::new(w,h).scramble(0.55, seed).ca(10, seed+1);
        // let mut basic_level = dla.and_equals(&ca, 0, 0);
        let mut conv = dla.conv(&MapFragment::new_open(2,2));
        conv.and_equals(&ca, 0, 0);
        // let mut basic_level = ca.conv(&MapFragment::new_open(2,2)).and_equals(&dla, 0, 0);
        // well these ops are linear so

        let mut basic_level = MapFragment::new(w,h).dla(1000,seed+6);
        let ca2 = MapFragment::new(w,h).scramble(0.55, seed + 4).ca(10, seed+8);

        basic_level.blit(&ca2, w/3, 0);
        basic_level.blit(&conv, 2* w / 3, 0);

        let mut ppseed = seed + 12534129;
        let player_pos = loop {
            let rx = (khash(ppseed) % 10) as i32 - 5;
            let ry = (khash(ppseed + 1) % 10) as i32 - 5;

            let pp = (w/2 + rx, h/2 + ry);

            if basic_level.get(pp.0, pp.1) {
                break pp;
            }

            ppseed += 2;
        };

        let paths = basic_level.paths_from(player_pos.0, player_pos.1);
        for i in 0..basic_level.w {
            for j in 0..basic_level.h {
                if !paths.reachable(i, j) {
                    basic_level.set(i, j, false);
                }
            }
        }
        */

        let mut level = Level {
            entities: HashMap::new(),
            tiles: basic_level.walkable.iter().map(|b| Tile {walkable: *b, overhang: false, underhang: false, edge: false}).collect(),
            side_length: w as usize,
            grid_size: 0.2,
            floor_colour: Vec3::new(0.75, 0.75, 0.5),
            wall_colour: Vec3::new(0.2, 0.2, 0.4),

        };

        // place player

        // let player_pos = *walker_positions.iter().max_by_key(|(x, y)| {
        //     let xp = x - params.side_length/2;
        //     let yp = y - params.side_length/2;
        //     xp*xp+yp*yp
        // }).unwrap();
        
        let player_pos_x = player_pos.0 as f32 * level.grid_size + level.grid_size as f32/2.0;
        let player_pos_y = player_pos.1 as f32 * level.grid_size + level.grid_size as f32/2.0;

        player.aabb.x = player_pos_x - player.aabb.w/2.0;
        player.aabb.y = player_pos_y - player.aabb.h/2.0;

        level.entities.insert(0, player);

        // CA pass
        for i in 0..level.side_length as i32 {
            for j in 0..level.side_length as i32 {
                let maybe_tile_below = level.get_tile(i, j+1);
                let maybe_tile_above = level.get_tile(i, j-1);
                let mut this_tile = level.get_tile_mut(i, j).unwrap();

                if this_tile.walkable {
                    match maybe_tile_above {
                        Some(above) if !above.walkable => {this_tile.underhang = true},
                        _ => {},
                    }
                    match maybe_tile_below {
                        Some(below) if !below.walkable => {this_tile.overhang = true},
                        _ => {},
                    }
                } else {
                    match maybe_tile_below {
                        Some(below) if below.walkable => {this_tile.edge = true},
                        _ => {},
                    }
                }
            }
        }

        for i in 0..w {
            for j in 0..h {
                if basic_level.get(i as i32, j as i32) {
                    let s = seed * 213414 + i as u32 * 4123523 + j as u32 * 31234;
                    if krand(s) < 0.08 {
                        let px = i as f32 * level.grid_size + level.grid_size as f32/2.0;
                        let py = j as f32 * level.grid_size + level.grid_size as f32/2.0;
                        let entity_kinds = vec!(EntityKind::WalkerShooter, EntityKind::RunnerGunner, EntityKind::Chungus, EntityKind::GunPickup);
                        let e = Entity::new(entity_kinds[khash(s + 2) as usize % entity_kinds.len()], Vec2::new(px, py));
                        level.entities.insert(khash(s + 1), e);
                    }
                }
            }
        }

        level
    }

    pub fn get_tile(&self, i: i32, j: i32) -> Option<Tile> {
        if i >= 0 && i < self.side_length as i32 && j >= 0 && j < self.side_length as i32 {
            Some(self.tiles[i as usize*self.side_length + j as usize])
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, i: i32, j: i32) -> Option<&mut Tile> {
        if i >= 0 && i < self.side_length as i32 && j >= 0 && j < self.side_length as i32 {
            Some(&mut self.tiles[i as usize*self.side_length + j as usize])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, i: i32, j: i32, t: Tile) {
        self.tiles[i as usize*self.side_length + j as usize] = t
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

            // check to terminate (left map)
            if grid_x < 0 || grid_x >= self.side_length as i32 - 1 || grid_y < 0 || grid_y >= self.side_length as i32 - 1 {
                return Some(Vec2::new(actual_march_x, actual_march_y));
            }
            
            // check to terminate (wall strike)
            if !self.tiles[(grid_x*self.side_length as i32 + grid_y) as usize].walkable {
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

/*
#[test]
pub fn test_raycast() {
    let level = Level {
        entities: HashMap::new(),
        tiles: vec!(
            Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall,
        ),
        side_length: 8,
        grid_size: 1.0,
        floor_colour: Vec3::new(0.0, 0.0, 0.0),
        wall_colour: Vec3::new(0.0, 0.0, 0.0),
    };
    
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(6.9, 6.9)), None);
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(1.1, 6.9)), None);
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(6.9, 1.1)), None);
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(7.1, 1.1)), Some(Vec2::new(7.0, 1.1)));
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(7.1, 7.1)), Some(Vec2::new(7.0, 7.0)));
}

#[test]
pub fn test_raycast2() {
    let level = Level {
        entities: HashMap::new(),
        tiles: vec!(
            Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall, Tile::Wall, Tile::Wall,
            Tile::Wall, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Open, Tile::Wall,
            Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall, Tile::Wall,
        ),
        side_length: 8,
        grid_size: 1.0,
        floor_colour: Vec3::new(0.0, 0.0, 0.0),
        wall_colour: Vec3::new(0.0, 0.0, 0.0),
    };
    
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(6.9, 6.9)), Some(Vec2::new(5.0, 5.0)));
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(1.1, 6.9)), Some(Vec2::new(1.1, 3.0)));
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(6.9, 1.1)), None);
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(7.1, 1.1)), Some(Vec2::new(7.0, 1.1)));
    assert_eq!(level.raycast(Vec2::new(1.1, 1.1), Vec2::new(7.1, 7.1)), Some(Vec2::new(5.0, 5.0)));
}
*/