use crate::kmath::*;

pub fn gen_dla(w: usize, h: usize, n: u32, mut seed: u32) -> Vec<bool> {
    let mut level = vec![false; w*h];
    level[w/2 * h + h/2] = true;

    for i in 0..n {
        let mut px = (khash(seed) % w as u32) as i32;
        let mut py = (khash(seed + 1) % h as u32) as i32;
        let (dx, dy) = [(-1, 0), (1, 0), (0, -1), (0, 1)][(khash(seed + 2) % 4) as usize];

        'outer: loop {
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                break 'outer;
            }

            for (ndx, ndy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = px + ndx;
                let ny = py + ndy;

                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }

                if level[nx as usize * h + ny as usize] {
                    level[px as usize * h + py as usize] = true;
                    break 'outer;
                }
            }

            px += dx;
            py += dy;
        }

        seed = khash(seed + 2305982305);
    }

    level
}

pub fn gen_dla_rim(w: usize, h: usize, n: u32, mut seed: u32) -> Vec<bool> {
    let mut level = vec![false; w*h];
    level[w/2 * h + h/2] = true;

    for i in 0..n {
        let (mut px, mut py, dx, dy) = [
            ((w - 1) as i32, (khash(seed + 1) % h as u32) as i32, -1, 0), 
            (0 as i32, (khash(seed + 1) % h as u32) as i32, 1, 0), 
            ((khash(seed + 1) % w as u32) as i32, (h - 1) as i32, -1, 0), 
            ((khash(seed + 1) % h as u32) as i32, 0, 1, 0), 
        ][(khash(seed + 2) % 4) as usize];

        'outer: loop {
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                break 'outer;
            }

            for (ndx, ndy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = px + ndx;
                let ny = py + ndy;

                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }

                if level[nx as usize * h + ny as usize] {
                    level[px as usize * h + py as usize] = true;
                    break 'outer;
                }
            }

            px += dx;
            py += dy;
        }

        seed = khash(seed + 2305982305);
    }

    level
}

pub fn gen_ca(w: usize, h: usize, density: f32, iters: u32, seed: u32) -> Vec<bool> {

    let mut level: Vec<bool> = (0..w*h).map(|s| krand(seed * 1231412 + s as u32) < density).collect();


    for n in 0..iters {
        let mut pong = vec![false; w*h];
        for i in 0..w {
            for j in 0..h {
                let neigh_count = [(-1i32, -1i32), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)].iter()
                    .map(|(dx, dy)| (i as i32 + dx, j as i32 + dy))
                    .filter(|(x, y)| *x > 0 && *y > 0 && *x < w as i32 && *y < h as i32)
                    .filter(|(x, y)| level[*x as usize * h + *y as usize])
                    .count();

                let walkable = neigh_count >= 1 && neigh_count < 5;


                pong[i as usize * w + j] = walkable;
            }
        }

        level = pong;
        for i in 0..w {
            for j in 0..h {
                if j == 0 || i == 0 || i == w-1 || j == h - 1 {
                    level[i*w+j] = false;
                }
            }
        }
    }

    level
}