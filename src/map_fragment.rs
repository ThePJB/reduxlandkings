use ordered_float::OrderedFloat;

use crate::kmath::*;
use crate::priority_queue::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MapFragment {
    pub w: i32,
    pub h: i32,
    pub walkable: Vec<bool>,
}

impl MapFragment {
    pub fn new(w: i32, h: i32) -> MapFragment {
        MapFragment {
            w,
            h,
            walkable: vec![false; (w*h) as usize],
        }
    }
    pub fn new_open(w: i32, h: i32) -> MapFragment {
        MapFragment {
            w,
            h,
            walkable: vec![true; (w*h) as usize],
        }
    }

    // xor= etc
    // conv, dilate

    pub fn blit(&mut self, other: &MapFragment, x: i32, y: i32) {
        println!("blit x{} y{} w{} h{} otherw{} otherh{}", x, y, self.w, self.h, other.w, other.h);
        for i in x.max(0)..(x+other.w).min(self.w) {
            for j in y.max(0)..(y+other.h).min(self.h) {
                self.set(i, j, other.get((i-x).max(0).min(self.w-1), (j-y).max(0).min(self.h-1)));
            }
        }
    }
    pub fn conv(mut self, other: &MapFragment) -> MapFragment {
        let mut pong = self.clone();

        let conv_xmin = -(other.w/2);
        let conv_ymin = -(other.h/2);

        println!("conv xmin {} ymin {}", conv_xmin, conv_ymin);

        for i in conv_xmin..(self.w - other.w/2) {
            for j in conv_ymin..(self.h - other.h/2) {
                let kernel_center_x = i + other.w/2;
                let kernel_center_y = j + other.h/2;
                println!("kernel center: {} {}", kernel_center_x, kernel_center_y);
                if self.get(kernel_center_x, kernel_center_y) {
                    pong.blit(other, i, j);
                }
            }
        }
        pong
    }
    pub fn and_equals(&mut self, other: &MapFragment, x: i32, y: i32) {
        for i in x.min(0)..(x+other.w).max(x + self.w) {
            for j in y.min(0)..(y+other.h).max(y + self.h) {
                self.set(i, j, self.get(i,j) && other.get(i-x, j-y));
            }
        }
    }
    pub fn or_equals(mut self, other: &MapFragment, x: i32, y: i32) {
        for i in x.min(0)..(x+other.w).max(x + self.w) {
            for j in y.min(0)..(y+other.h).max(y + self.h) {
                self.set(i, j, self.get(i,j) || other.get(i-x, j-y));
            }
        }
    }

    pub fn get(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.w || y >= self.h { panic!("get {},{} out of bounds (w/h {},{})", x, y , self.w, self.h)};
        self.walkable[x as usize * self.h as usize + y as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, value: bool) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h { panic!("set out of bounds")};
        self.walkable[x as usize * self.h as usize + y as usize] = value;
    }

    pub fn scramble(mut self, density: f32, seed: u32) -> MapFragment {
        self.walkable = self.walkable.iter().enumerate().map(|(i, _)| krand(seed + i as u32) < density).collect();
        self
    }

    pub fn dla(mut self, iters: u32, mut seed: u32) -> MapFragment {
        self.set(self.w/2, self.h/2, true);
        for i in 0..iters {
            let mut px = (khash(seed) % self.w as u32) as i32;
            let mut py = (khash(seed + 1) % self.h as u32) as i32;
            let (dx, dy) = [(-1, 0), (1, 0), (0, -1), (0, 1)][(khash(seed + 2) % 4) as usize];
                
            'outer: loop {
                if px < 0 || py < 0 || px >= self.w as i32 || py >= self.h as i32 {
                    break 'outer;
                }

                for (ndx, ndy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = px + ndx;
                    let ny = py + ndy;

                    if nx < 0 || ny < 0 || nx >= self.w as i32 || ny >= self.h as i32 { continue; }

                    if self.get(nx, ny) {
                        self.set(px, py, true);
                        break 'outer;
                    }
                }

                px += dx;
                py += dy;
            }
            seed = khash(seed + 2305982305);
        }

        self
    }

    pub fn dla_rim(mut self, iters: u32, seed: u32) -> MapFragment{

        self
    }

    pub fn ca(mut self, iters: u32, seed: u32) -> MapFragment{
        for n in 0..iters {
            let mut pong = self.clone();
            for i in 0..self.w {
                for j in 0..self.h {
                    let neigh_count = [(-1i32, -1i32), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)].iter()
                        .map(|(dx, dy)| (i + dx, j + dy))
                        .filter(|(x, y)| *x > 0 && *y > 0 && *x < self.w && *y < self.h)
                        .filter(|(x, y)| self.get(*x, *y))
                        .count();
    
                    pong.set(i,j, neigh_count >= 1 && neigh_count < 5);
                }
            }
            self = pong;
        }

        self
    }

    // drunk walk

    // path stuff
    pub fn paths_from(&self, x: i32, y: i32) -> MapPaths {
        // djikstra
        let mut pq = PriorityQueue::new();
        let mut table = HashMap::new();
        table.insert((x, y), (x, y, 0.0f32));
        pq.set(0.0, (x, y));

        while let Some((dist, (x, y))) = pq.remove_min_with_priority() {
            // idea: flip with the weights for more variety
            for (dx, dy, w) in [(-1, 0, 1.0), (0, -1, 1.0), (0, 1, 1.0), (1, 0, 1.0)] {
                let nx = x + dx;
                let ny = y + dy;

                if nx > 0 && nx < self.w && ny > 0 && ny < self.h && self.get(nx, ny) {
                    let &(prev_x, prev_y, old_dist) = table.get(&(nx, ny)).unwrap_or(&(-1, -1, std::f32::INFINITY));
                    
                    let new_dist = dist + w;
                    if new_dist < old_dist {
                        // decrease key?
                        pq.set(new_dist, (nx, ny));
                        table.insert((nx, ny), (x, y, new_dist));
                    }
                }
                // if its undiscovered put it in the queue
                // i think use of pq rules out needing to go backz
            }

        }

        MapPaths{prev: table}
    }    
}

pub struct MapPaths {
    prev: HashMap<(i32, i32), (i32, i32, f32)>,
}

impl MapPaths {
    pub fn furthest(&self) -> (i32, i32) {
        self.prev.iter().max_by_key(|((x, y), (px, py, d))| OrderedFloat(*d)).map(|((x, y), (px, py, d))| (*x, *y)).unwrap()
    }

    pub fn dist(&self, x: i32, y: i32) -> f32 {
        self.prev.get(&(x, y)).unwrap().2
    }

    pub fn path(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(i32, i32)> {
        let mut v = Vec::new();
        let mut p = (x2, y2);
        while p != (x1, y1) {
            let prev = self.prev.get(&p).map(|(x, y, w)| (*x, *y)).unwrap();
            v.push(prev);
            p = prev;
        }
        v.reverse();
        v
    }

    pub fn tiles_within_dist(&self, dist: f32) -> Vec<(i32, i32)> {
        self.prev.iter().filter_map(|((x, y), (px, py, d))| {
            if *d < dist {
                Some((*x, *y))
            } else {
                None
            }
        }).collect()
    }

    pub fn reachable(&self, x: i32, y: i32) -> bool {
        self.prev.get(&(x,y)).is_some()
    }
}

// impl map paths: 
// maybe get critical paths
// sus if distance < amount
// get furthest lol

// now this could get cleaned up further, we could have a hot paths function for example