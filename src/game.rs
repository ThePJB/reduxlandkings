use glam::*;
use glow::*;
use std::collections::HashMap;
use crate::level::*;
use crate::renderer::*;
use crate::rect::*;

#[derive(Debug)]
pub enum InputCommand {
    Look(Vec2),
    Shoot(Vec2),

    // movt how
    // press schema - for when theres a lot of shit
    // held schema - actually movt is complicated, just calculate
    // mouse stuff - also just calculate
    Move(Vec2),

    Reset,
}

pub struct Camera {
    pub projection: Mat4,
    pub view: Mat4,
    pub inverse_projection: Mat4,
    pub inverse_view: Mat4,
}

pub struct Game {
    pub camera: Camera,
    level: Level,
    look: Vec2,
    player_id: u32,

}

impl Game {
    pub fn new(aspect_ratio: f32) -> Game {
        let ortho = Mat4::orthographic_lh(
            0.0, aspect_ratio, 0.0, 1.0, 0.0, 1.0);
            // left right bottom top near far?
    
        let view = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
    
        println!("ortho: {}", ortho);
        println!("view: {}", view);

        Game {
            level: Level::new(),
            look: Vec2::new(0.0, 0.0),
            camera: Camera {
                projection: ortho,
                view: view,
                inverse_projection: ortho.inverse(),
                inverse_view: view.inverse(),
            },
            player_id: 0,
        }
    }

    pub fn update(&mut self, aspect_ratio: f32, dt: f64) {
        for (_, entity) in self.level.entities.iter_mut() {
            entity.aabb.x += entity.velocity.x * dt as f32;
            entity.aabb.y += entity.velocity.y * dt as f32;
        }

        // move camera
        if let Some(player) = self.level.entities.get(&self.player_id) {
            let player_pos = player.aabb.centroid();
            

            let look_strength = 0.2;
            let look_translation_x = self.look.x - aspect_ratio/2.0;
            let look_translation_y = 0.5 - self.look.y;

            self.camera.view = Mat4::from_translation(Vec3::new(
                -player_pos.x + aspect_ratio/2.0 - look_strength*look_translation_x, 
                -player_pos.y + 0.5 - look_strength*look_translation_y, 
                0.0));
            self.camera.inverse_view = self.camera.view.inverse();
        }
        // update projection mat (eg if aspect ratio changes)
        self.camera.projection = Mat4::orthographic_lh(0.0, aspect_ratio, 0.0, 1.0, 0.0, 1.0);
        self.camera.inverse_projection = self.camera.projection.inverse();
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        for i in 0..self.level.side_length {
            for j in 0..self.level.side_length {
                let tile_rect = Rect::new(
                    i as f32 * self.level.grid_size, 
                    j as f32 * self.level.grid_size, 
                    self.level.grid_size,
                    self.level.grid_size);

                let tile_type = self.level.tiles[i*self.level.side_length + j];

                renderer.draw_rect(
                    tile_rect,
                    if tile_type == Tile::Open {
                        Vec3::new(0.8, 0.8, 0.4)
                    } else {
                        Vec3::new(0.2, 0.2, 0.4)
                    },
                    1.0);
            }
        }

        for (_, ent) in self.level.entities.iter() {
            let ent_rect = Rect::new(
                ent.aabb.x,
                ent.aabb.y,
                ent.aabb.w,
                ent.aabb.h,
            );

            renderer.draw_rect(
                ent_rect,
                Vec3::new(1.0, 1.0, 1.0),
                0.5,
            );
        }
    }

    pub fn apply_command(&mut self, cmd: InputCommand) {
        match cmd {
            InputCommand::Look(p) => {
                self.look = p
            },
            InputCommand::Shoot(p) => {},
            InputCommand::Move(p) => {
                let player_speed = 0.5;
                self.level.entities.get_mut(&self.player_id).unwrap().velocity = p * player_speed;
            },
            InputCommand::Reset => {},
        }
    }
}
