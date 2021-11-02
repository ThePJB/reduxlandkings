use glam::*;
use std::collections::HashMap;
use crate::rect::*;
use crate::level::*;
use crate::entity::*;

#[derive(Debug, Clone, Copy)]
pub enum CollisionObject {
    Entity(u32),
    Terrain(i32, i32),
}

#[derive(Debug)]
pub struct CollisionEvent {
    pub subject: u32,
    pub object: CollisionObject,
    pub penetration: Vec2,
    pub subject_rect: Rect, // with bound factored in, but what if we filter things
}

pub fn collide_entity_terrain(
        entities: &HashMap<u32, Entity>, 
        terrain: &Vec<Tile>, 
        terrain_grid_size: f32,
        terrain_side_length: i32,
        collisions: &mut Vec<CollisionEvent>, 
        dt: f32) {

    for (subject_key, subject) in entities {

        let subject_rect_desired = subject.aabb.translate(subject.velocity * dt);

        let terrain_idx = subject.aabb.centroid() / terrain_grid_size;

        for i in -1..=1 {
            let tx = terrain_idx.x as i32 + i;
            if tx < 0 || tx >= terrain_side_length {continue};
            for j in -1..=1 {
                let ty = terrain_idx.y as i32 + j;
                if ty < 0 || ty >= terrain_side_length {continue};

                if let Some(tile) = terrain.get((tx * terrain_side_length + ty) as usize) {
                    if *tile == Tile::Wall {
                        let tile_rect = Rect::new(tx as f32 * terrain_grid_size, ty as f32 * terrain_grid_size, terrain_grid_size, terrain_grid_size);
                        if rect_intersection(subject_rect_desired, tile_rect) {

                            // depends sign of penetration
                            let penetration = least_penetration(subject_rect_desired, tile_rect);

                            collisions.push(CollisionEvent {
                                subject: *subject_key,
                                object: CollisionObject::Terrain(tx, ty),
                                penetration,
                                subject_rect: subject_rect_desired.translate(penetration),
                            })
                        }
                    }
                }
            }
        }
    }
}

pub fn collide_entity_entity(
        entities: &HashMap<u32, Entity>, 
        collisions: &mut Vec<CollisionEvent>, 
        dt: f32) {

    for (subject_key, subject) in entities {

        let subject_rect_desired = subject.aabb.translate(subject.velocity * dt);
        
        for (object_key, object) in entities {
            if subject_key == object_key {continue};
            
            let object_rect_desired = object.aabb.translate(object.velocity * dt);

            if rect_intersection(subject_rect_desired, object_rect_desired) {
                let penetration = least_penetration(subject_rect_desired, object_rect_desired);

                // so the final rects are desired - penetration * velocity split:
                // equal velocity: 50/50
                // all one: 100%/0
                // a/(a+b)
                // nah its not that easy because they could have different starting distance
                // just calculate a shit bound, it probably won't matter

                // again maybe its -penetration on the translate

                collisions.push(CollisionEvent {
                    subject: *subject_key,
                    object: CollisionObject::Entity(*object_key),
                    penetration,
                    subject_rect: subject_rect_desired.translate(penetration),
                })
            }
        }
    }
}

// then see how we go doing movement bounds
// maybe storing pen  vec instead of other shit is fine

fn movement_bounds(subject_key: u32, collisions: &Vec<CollisionEvent>) -> (f32, f32, f32, f32) {
    let max_dx = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.penetration.x < 0.0)
        .map(|col| col.subject_rect.left())
        .fold(f32::INFINITY, |a, b| a.min(b));  // feel like this should be max

    let max_dy = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.penetration.y > 0.0)  // hopefully coordinate system not cooked
        .map(|col| col.subject_rect.top())
        .fold(f32::INFINITY, |a, b| a.min(b));
        
    let min_dx = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.penetration.x > 0.0)
        .map(|col| col.subject_rect.right())
        .fold(-f32::INFINITY, |a, b| a.max(b));

    let min_dy = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.penetration.y < 0.0)
        .map(|col| col.subject_rect.bot())
        .fold(-f32::INFINITY, |a, b| a.max(b));

    return (min_dx, max_dx, min_dy, max_dy);
}

fn clamp(val: f32, min: f32, max: f32) -> f32 {
    match val {
        val if val <= min => min,
        val if val >= max => max,
        _ => val
    }
}

pub fn apply_movement(entities: &mut HashMap<u32, Entity>, collisions: &Vec<CollisionEvent>, dt: f32) {
    for (entity_key, entity) in entities.iter_mut() {
        let (min_x, max_x, min_y, max_y) = movement_bounds(*entity_key, collisions);
        let x_movt = clamp(entity.velocity.x * dt, min_x, max_x);
        let y_movt = clamp(entity.velocity.y * dt, min_y, max_y);

        entity.aabb.x += x_movt;
        entity.aabb.y += y_movt;
    }
}

// todo test a bit
// deffo some mistakos

//probably a less stupid way to do bounds like filter for rects and get rightmost etc
// putting rect in event is misleading maybe bound is better, that fn i deleted