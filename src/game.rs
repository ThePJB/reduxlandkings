use glam::{Mat4, Vec3};
use crate::kmath::*;
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
    player_pos: Vec2,   // camera focus
    look: Vec2,         // screen space
    aspect_ratio: f32,

    level: Level,
    player_id: u32,
    collisions: Vec<CollisionEvent>,

}


#[test]
fn test_screen_to_world() {
    let mut game = Game::new(1.0);
    game.player_pos = Vec2::new(5.0, 5.0);
    game.look = Vec2::new(0.5, 0.5); // cursor in middle should cancel out look
    let p = Vec2::new(0.0, 0.0); // top left
    assert_eq!(game.screen_to_world(p), Vec2::new(4.5, 4.5));
    
    game.look = Vec2::new(0.0, 0.0);
    assert_eq!(game.screen_to_world(p), Vec2::new(4.4, 4.4));
}


impl Game {
    // screen is 0..aspect ratio in x and 0..1 in y
    // ok well at least this works, fucking matrices
    pub fn screen_to_world(&self, p: Vec2) -> Vec2 {
        let screen_max = Vec2::new(self.aspect_ratio, 1.0);
        let look_weight = 0.2;
        return p + self.player_pos + look_weight*(self.look - 0.5*screen_max) - 0.5*screen_max;
    }

    // screen is 0..aspect ratio in x and 0..1 in y
    pub fn world_to_screen(&self, p: Vec2) -> Vec2 {
        Vec2::new(0.0, 0.0)
    }

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
            player_pos: Vec2::new(0.0, 0.0),
            aspect_ratio,
        }
    }

    pub fn update(&mut self, aspect_ratio: f32, dt: f32) {
        self.t += dt;
        if let Some(player) = self.level.entities.get(&self.player_id) {
            self.player_pos = player.aabb.centroid();
        }
        self.aspect_ratio = aspect_ratio;

        self.collisions.clear();

        {   // AI time
            let mut commands = Vec::new();
            for (entity_id, entity) in self.level.entities.iter() {
                entity.think(*entity_id, &self.level, &mut commands);
            }
            
            for command in commands {
                self.level.apply_command(command);
            }
        }

        {   // Shooting
            let mut new_bullets = Vec::new();

            for (entity_key, entity) in self.level.entities.iter_mut() {
                let will_shoot = entity.gun.will_shoot(entity.want_shoot, self.t);
                entity.gun.update(entity.want_shoot, will_shoot, self.t);
                if will_shoot {
                    entity.gun.make_bullets(&mut new_bullets, entity.aabb.centroid(), entity.previous_shoot_dir, *entity_key);
                }
            }

            for new_bullet in new_bullets {
                self.level.entities.insert(rand::thread_rng().gen(), new_bullet);
            }
        }



        collide_entity_entity(&self.level.entities, &mut self.collisions, dt);
        collide_entity_terrain(&self.level.entities, &self.level.tiles, self.level.grid_size, 
            self.level.side_length as i32, &mut self.collisions, dt);

        // handle bullet collisions
        for col in self.collisions.iter() {
            let damage = if let Some(subject) = self.level.entities.get_mut(&col.subject) {
                if subject.kind == EntityKind::Bullet {
                    subject.health = 0.0;
                    Some(subject.damage)
                } else {
                    None
                }
            } else {
                None
            };

            match damage {
                Some(damage_amount) => {match col.object {
                    CollisionObject::Entity(id) => {
                        if let Some(object) = self.level.entities.get_mut(&id) {
                            object.health -= damage_amount;
                        }
                    },
                    _ => {},
                }},
                None => {},
            }
        }

        self.level.entities.retain(|_, ent| ent.health > 0.0);

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

        // calculate enemies remaining
        let remaining_enemies = self.level.entities.iter().map(|(_, e)| e.kind).filter(|ek| *ek != EntityKind::Bullet && *ek != EntityKind::Player).count();
        //println!("Remaining enemies: {}", remaining_enemies);

        if remaining_enemies == 0 {
            println!("you win!");
            self.level = Level::new();
        }
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
                    EntityKind::RunnerGunner => Vec3::new(0.0, 0.0, 1.0),
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
                let shoot_pos_world = self.screen_to_world(normalized_pos);
                let mut dir = shoot_pos_world - self.player_pos;
                dir.y = -dir.y;
                dir = dir.normalize();

                self.level.apply_command(EntityCommand::Shoot(self.player_id, dir));

/*
                // calculate play pos on screen, or look pos in world
                let look_tform = self.camera.inverse_view.transform_point3a(Vec3A::new(normalized_pos.x, normalized_pos.y, 0.0));
                let look_world_pos = Vec2::new(look_tform.x, look_tform.y) + self.look;
                println!("click world pos: {:?}", look_world_pos);
                if let Some(player) = self.level.entities.get(&self.player_id) {
                    let mut dir = (look_world_pos - player.aabb.centroid()).normalize();
                    dir.y = -dir.y;
                    // look theres some transform spaghetti here for sure but it works
                    self.level.apply_command(EntityCommand::Shoot(self.player_id, dir.x, dir.y));
                }
                */
            },
            InputCommand::Unshoot => {
                self.level.apply_command(EntityCommand::Unshoot(self.player_id));
            },
            InputCommand::Move(dir) => {
                self.level.apply_command(EntityCommand::Move(self.player_id, dir));
            },
            InputCommand::Reset => {
                self.level = Level::new();
            },
        }
    }
}
