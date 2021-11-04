use glam::*;
use glow::*;
use std::collections::HashMap;
use rand::prelude::*;

use crate::level::*;
use crate::renderer::*;
use crate::rect::*;
use crate::entity::*;
use crate::collision_system::*;

#[derive(Debug)]
pub enum InputCommand {
    Look(Vec2),
    Shoot(Vec2),
    Unshoot,

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
    t: f32,
    pub camera: Camera,
    level: Level,
    look: Vec2,
    player_id: u32,
    collisions: Vec<CollisionEvent>,

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
            collisions: Vec::new(),
            t: 0.0,
        }
    }

    pub fn update(&mut self, aspect_ratio: f32, dt: f32) {
        self.t += dt;

        self.collisions.clear();

        {   // Shooting
            let mut new_bullets = Vec::new();

            for (entity_key, entity) in self.level.entities.iter_mut() {
                if entity.gun.update(entity.want_shoot, dt, self.t) {
                    new_bullets.push(Entity::new(EntityKind::Bullet, entity.aabb.centroid())
                        .with_velocity(entity.previous_shoot_dir * entity.gun.bullet_speed)
                        .with_owner(*entity_key));
                }
            }

            for new_bullet in new_bullets {
                self.level.entities.insert(rand::thread_rng().gen(), new_bullet);
            }
        }

        collide_entity_entity(&self.level.entities, &mut self.collisions, dt);
        collide_entity_terrain(&self.level.entities, &self.level.tiles, self.level.grid_size, 
            self.level.side_length as i32, &mut self.collisions, dt);

        for col in self.collisions.iter().filter(|col| col.subject == self.player_id) {
            println!("Player collision: {:?} {:?}", col.object, col.penetration);
        }

        apply_movement(&mut self.level.entities, &self.collisions, dt);

        // move camera
        if let Some(player) = self.level.entities.get(&self.player_id) {
            let player_pos = player.aabb.centroid();
            
            // look is fucking up the camera matrix

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

                renderer.draw_rect(tile_rect, Vec3::new(0.9, 0.9, 0.9), 1.0);
                renderer.draw_rect(
                    tile_rect.dilate(-0.003),
                    if tile_type == Tile::Open {
                        Vec3::new(0.8, 0.8, 0.4)
                    } else {
                        Vec3::new(0.2, 0.2, 0.4)
                    },
                    0.6);
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
                match ent.kind {
                    EntityKind::Player => Vec3::new(1.0, 1.0, 1.0),
                    EntityKind::WalkerShooter => Vec3::new(1.0, 0.0, 0.0),
                    EntityKind::Bullet => Vec3::new(1.0, 1.0, 0.0),
                },
                0.5,
            );
        }
    }

    pub fn apply_command(&mut self, cmd: InputCommand) {
        match cmd {
            InputCommand::Look(p) => {
                self.look = p
            },
            InputCommand::Shoot(normalized_pos) => {
                // calculate play pos on screen, or look pos in world
                let look_tform = self.camera.inverse_view.transform_point3a(Vec3A::new(normalized_pos.x, normalized_pos.y, 0.0));
                let look_world_pos = Vec2::new(look_tform.x, look_tform.y);
                println!("click world pos: {:?}", look_world_pos);
                if let Some(player) = self.level.entities.get(&self.player_id) {
                    let mut dir = (look_world_pos - player.aabb.centroid()).normalize();
                    dir.y = -dir.y;
                    // look theres some transform spaghetti here for sure but it works
                    self.level.apply_command(EntityCommand::Shoot(self.player_id, dir.x, dir.y));
                }
            },
            InputCommand::Unshoot => {
                self.level.apply_command(EntityCommand::Unshoot(self.player_id));
            },
            InputCommand::Move(p) => {
                let player_speed = 0.5;
                self.level.entities.get_mut(&self.player_id).unwrap().velocity = p * player_speed;
            },
            InputCommand::Reset => {},
        }
    }
}
