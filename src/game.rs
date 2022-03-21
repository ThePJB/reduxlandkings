use glam::{Mat4, Vec3};
use crate::kmath::*;
use glow::*;
use std::collections::VecDeque;
use rand::prelude::*;

use crate::level::*;
use crate::renderer::*;
use crate::rect::*;
use crate::entity::*;
use crate::collision_system::*;
use crate::gun::*;

#[derive(Debug)]
pub enum InputCommand {
    Look(Vec2),
    Shoot(Vec2),
    Unshoot,
    Move(Vec2),
    EatGun,

    Reset,
}

pub struct Game {
    seed: u32,
    t: f32,
    pub player_pos: Vec2,   // camera focus
    look: Vec2,             // screen space
    aspect_ratio: f32,

    level: Level,
    player_id: u32,
    collisions: Vec<CollisionEvent>,

    player_gun_fifo: VecDeque<Gun>,

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

fn draw_gun_icon(renderer: &mut Renderer, r: Rect, height: f32) {

    renderer.draw_rect(r, Vec3::new(0.0, 0.0, 0.0), height);

    let inner = r.dilate(-0.005);

    renderer.draw_rect(inner, Vec3::new(1.0, 1.0, 1.0), height + 1.0);

    renderer.draw_rect(inner.child(0.1, 0.2, 0.8, 0.3), Vec3::new(0.0, 0.0, 0.0), height + 2.0);
    renderer.draw_rect(inner.child(0.1, 0.2, 0.3, 0.5), Vec3::new(0.0, 0.0, 0.0), height + 2.0);
}

impl Game {
    // screen is 0..aspect ratio in x and 0..1 in y
    // ok well at least this works, fucking matrices
    pub fn screen_to_world(&self, p: Vec2) -> Vec2 {
        let screen_max = Vec2::new(self.aspect_ratio, 1.0);
        let look_weight = 0.2;
        return p + self.player_pos + look_weight*(self.look - 0.5*screen_max) - 0.5*screen_max;
    }

    pub fn new(aspect_ratio: f32) -> Game {

        let mut game = Game {
            seed: 0,
            // level: Level::new(Entity::new(EntityKind::Player, Vec2::new(0.0, 0.0))),
            level: Level::new_dla(Entity::new(EntityKind::Player, Vec2::new(0.0, 0.0)), 0),
            look: Vec2::new(0.0, 0.0),
            player_id: 0,
            collisions: Vec::new(),
            t: 0.0,
            player_pos: Vec2::new(0.0, 0.0),
            aspect_ratio,
            player_gun_fifo: VecDeque::new(),
        };

        game.player_gun_fifo.push_back(generate_gun(3));
        game.player_gun_fifo.push_back(generate_gun(3));
        game.player_gun_fifo.push_back(generate_gun(3));
        
        game
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
                entity.think(*entity_id, &self.level, &mut commands, self.t);
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
                    entity.gun.make_bullets(&mut new_bullets, entity.aabb.centroid(), entity.previous_shoot_dir, *entity_key, entity.team);
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

        // handle pickups
        for col in self.collisions.iter() {
            let gun = if col.subject == self.player_id {
                match col.object {
                    CollisionObject::Entity(id) => {
                        if let Some(entity) = self.level.entities.get(&id) {
                            if entity.kind == EntityKind::GunPickup {
                                Some(entity.gun.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            } else {
                None
            };

            match gun {
                Some(pickup_gun) => {
                    match col.object {
                        CollisionObject::Entity(id) => {
                            if let Some(entity) = self.level.entities.get_mut(&id) {
                                entity.health = 0.0
                            }
                        },
                    _ => {},
                    }

                    self.player_gun_fifo.push_back(pickup_gun);
                },
                None => {},
            }
        }

        self.level.entities.retain(|_, ent| ent.health > 0.0);

        for col in self.collisions.iter().filter(|col| col.subject == self.player_id) {
            //println!("Player collision: {:?} {:?}", col.object, col.penetration);
        }

        apply_movement(&mut self.level.entities, &self.collisions, dt);

        // calculate enemies remaining
        let remaining_enemies = self.level.entities.iter().map(|(_, e)| e.kind).filter(|ek| *ek != EntityKind::Bullet && *ek != EntityKind::Player && *ek != EntityKind::GunPickup).count();
        //println!("Remaining enemies: {}", remaining_enemies);

        if remaining_enemies == 0 {
            // println!("you win!");
            // if let Some(player) = self.level.entities.get(&self.player_id) {
            //     self.level = Level::new_dla(player.clone());
            // }
        }
    }


    pub fn draw(&self, renderer: &mut Renderer) {
        let floor_height = 10.0;
        let wall_height = 100.0;
        let wall_front_height = 101.0;
        let wall_overhang_height = 500.0;
        let wall_underhang_height = 150.0;
        let entity_height = 200.0;

        let hud_back_height = 800.0;
        let hud_front_height = 900.0;

        let overhang_amount = 0.15;
        let underhang_amount = 0.15;
        let wall_front_amount = 0.5;    // to fix jank corner case will need to be the same as overhang, will look ok once characters are taller or tiles are smaller


        {   // Level
            let scale = 1.2;
            let look_strength = 0.2;
    
            let dims = scale * Vec2::new(self.aspect_ratio, 1.0);
            let look_vec = (scale*self.look) - dims/2.0;
            let center = self.player_pos + look_strength * look_vec;
    
            renderer.top_left = center - dims/2.0;
            renderer.bot_right = center + dims/2.0;
    
            for i in 0..self.level.side_length {
                for j in 0..self.level.side_length {
                    let tile_rect = Rect::new(
                        i as f32 * self.level.grid_size, 
                        j as f32 * self.level.grid_size, 
                        self.level.grid_size,
                        self.level.grid_size);
    
                    let tile = self.level.tiles[i*self.level.side_length + j];

                    let edge_colour = Vec3::new(0.1, 0.1, 0.3);
                    if tile.walkable {
                        renderer.draw_rect(tile_rect, self.level.floor_colour, floor_height)
                    } else {
                        renderer.draw_rect(tile_rect, self.level.wall_colour, wall_height);
                    }
                    if tile.overhang {
                        renderer.draw_rect(tile_rect.child(0.0, 1.0 - overhang_amount, 1.0, overhang_amount), self.level.wall_colour, wall_overhang_height);
                    }
                    if tile.underhang {
                        renderer.draw_rect(tile_rect.child(0.0, 0.0, 1.0, underhang_amount), edge_colour, wall_underhang_height);
                    }
                    if tile.edge {
                        renderer.draw_rect(tile_rect.child(0.0, 1.0 - wall_front_amount, 1.0, wall_front_amount), edge_colour, wall_front_height);
                    }
                }
            }
    
            for (_, ent) in self.level.entities.iter() {
                let ent_rect = Rect::new(
                    ent.aabb.x,
                    ent.aabb.y,
                    ent.aabb.w,
                    ent.aabb.h,
                );


                match ent.kind {
                    EntityKind::Player => renderer.draw_rect(ent_rect, Vec3::new(1.0, 1.0, 1.0), entity_height),
                    EntityKind::WalkerShooter => renderer.draw_rect(ent_rect, Vec3::new(1.0, 0.0, 0.0), entity_height),
                    EntityKind::RunnerGunner => renderer.draw_rect(ent_rect, Vec3::new(0.0, 0.0, 1.0), entity_height),
                    EntityKind::Chungus => renderer.draw_rect(ent_rect, Vec3::new(0.0, 0.0, 0.5), entity_height),
                    EntityKind::Bullet => renderer.draw_rect(ent_rect, Vec3::new(1.0, 1.0, 0.0), entity_height),
                    EntityKind::GunPickup => draw_gun_icon(renderer, ent_rect, entity_height),
                };
            }
        }

        {   // Minimap
            renderer.top_left = Vec2::new(0.0, 0.0);
            renderer.bot_right = Vec2::new(self.aspect_ratio, 1.0);

            let mm_border = Rect::new(0.0, 0.7, 0.3, 0.3).dilate(-0.02);
            renderer.draw_rect(mm_border, Vec3::new(0.0, 0.0, 0.0), hud_back_height);
            let mm_rect = mm_border.dilate(-0.01);

            let level_w = self.level.grid_size * self.level.side_length as f32;

            for i in 0..self.level.side_length {
                for j in 0..self.level.side_length {
                    let tile_rect = Rect::new(
                        i as f32 * self.level.grid_size / level_w * mm_rect.w + mm_rect.x, 
                        j as f32 * self.level.grid_size / level_w * mm_rect.h + mm_rect.y, 
                        self.level.grid_size / level_w * mm_rect.w,
                        self.level.grid_size / level_w * mm_rect.h);
    
                    let tile = self.level.tiles[i*self.level.side_length + j];
    
                    renderer.draw_rect(
                        tile_rect,
                        
                        if tile.walkable {
                            Vec3::new(0.8, 0.8, 0.4)
                        } else {
                            Vec3::new(0.2, 0.2, 0.4)
                        },
                        hud_front_height);
                }
            }

            let player_rect = Rect::new_centered(
                self.player_pos.x / level_w * mm_rect.w + mm_rect.x,
                self.player_pos.y / level_w * mm_rect.w + mm_rect.y,
                0.01, 0.01);
            renderer.draw_rect(player_rect, Vec3::new(1.0, 1.0, 1.0), hud_front_height + 1.0);
        }

        {   // HP bar
            let hp_percentage = if let Some(player) = self.level.entities.get(&self.player_id) {
                player.health / player.max_health
            } else { 
                0.0 
            };

            let hp_border = Rect::new(0.0, 0.65, 0.3, 0.08).dilate(-0.02);
            renderer.draw_rect(hp_border, Vec3::new(0.0, 0.0, 0.0), hud_back_height);
            let mut hp_bar = hp_border.dilate(-0.01);
            hp_bar.w *= hp_percentage;
            
            renderer.draw_rect(hp_bar, Vec3::new(1.0, 0.0, 0.0), hud_front_height);
        }

        {   // Gun gui
            // current
            draw_gun_icon(renderer, Rect::new(0.02, 0.02, 0.06, 0.06), hud_front_height);

            let ammo_percentage = if let Some(player) = self.level.entities.get(&self.player_id) {
                player.gun.state.ammo as f32 / player.gun.max_ammo as f32
            } else { 
                0.0 
            };

            let ammo_border = Rect::new(0.1, 0.03, 0.15, 0.04);
            renderer.draw_rect(ammo_border, Vec3::new(0.0, 0.0, 0.0), hud_back_height);
            let mut ammo_bar = ammo_border.dilate(-0.01);
            ammo_bar.w *= ammo_percentage;
            
            renderer.draw_rect(ammo_bar, Vec3::new(1.0, 1.0, 0.0), hud_front_height);

            let mut ypos = 0.08;
            for gun in self.player_gun_fifo.iter() {
                ypos += 0.01; // padding
                draw_gun_icon(renderer, Rect::new(0.02, ypos, 0.04, 0.04), hud_front_height);
                ypos += 0.04; // padding
            }
        }
    }

    pub fn reset_level(&mut self) {
        self.seed += 1;
        let player = if let Some(player) = self.level.entities.get(&self.player_id) {
            player.clone()
        } else {
            Entity::new(EntityKind::Player, Vec2::new(0.0, 0.0))
        };
        self.level = Level::new_dla(player, self.seed);
    }

    pub fn apply_command(&mut self, cmd: InputCommand) {
        match cmd {
            InputCommand::Look(p) => {
                self.look = p
            },
            InputCommand::Shoot(normalized_pos) => {
                let shoot_pos_world = self.screen_to_world(normalized_pos);
                let dir = (shoot_pos_world - self.player_pos).normalize();

                self.level.apply_command(EntityCommand::Shoot(self.player_id, dir));
            },
            InputCommand::Unshoot => {
                self.level.apply_command(EntityCommand::Unshoot(self.player_id));
            },
            InputCommand::Move(dir) => {
                self.level.apply_command(EntityCommand::Move(self.player_id, dir));
            },
            InputCommand::Reset => {
                self.reset_level();
            },
            InputCommand::EatGun => {
                if let Some(player) = self.level.entities.get_mut(&self.player_id) {
                    if let Some(next_gun) = self.player_gun_fifo.pop_front() {
                        player.gun = next_gun;
                        player.health += 1.0;
                        println!("New gun: {:?}", player.gun.gun_traits);
                        if player.health > player.max_health {
                            player.health = player.max_health;
                        }
                    }
                }
            }
        }
    }
}
