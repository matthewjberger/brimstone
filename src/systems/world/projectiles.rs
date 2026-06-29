use crate::ecs::{BoomerWorld, Projectile};
use crate::systems::world::textures::MAT_FIREBALL;
use crate::systems::world::{billboard, fx, game, player};
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::physics::resources::physics_world_cast_ray;
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const PLAYER_HIT_PADDING: f32 = 0.55;

pub fn spawn(boomer_world: &mut BoomerWorld, world: &mut World, origin: Vec3, target: Vec3) {
    let mut direction = target - origin;
    if direction.norm() < 1e-3 {
        direction = vec3(0.0, 0.0, 1.0);
    }
    let velocity = direction.normalize() * tuning::FIREBALL_SPEED;
    let entity = billboard::spawn(
        world,
        MAT_FIREBALL,
        origin,
        vec3(tuning::FIREBALL_SCALE, tuning::FIREBALL_SCALE, 1.0),
    );
    boomer_world.resources.projectiles.items.push(Projectile {
        entity,
        position: origin,
        velocity,
        lifetime: tuning::FIREBALL_LIFETIME,
        damage: tuning::FIREBALL_DAMAGE,
    });
}

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let player_entity = boomer_world.resources.player.player_entity;
    let player_center = player::position(boomer_world, world);

    let mut items = std::mem::take(&mut boomer_world.resources.projectiles.items);
    let mut removed: Vec<(Entity, Vec3, bool)> = Vec::new();

    items.retain_mut(|item| {
        item.lifetime -= delta;
        let start = item.position;
        let travel = item.velocity * delta;
        let step = travel.norm().max(1e-4);
        let direction = item.velocity / item.velocity.norm().max(1e-4);
        let next = start + travel;
        item.position = next;

        if item.lifetime <= 0.0 {
            removed.push((item.entity, next, false));
            return false;
        }

        let to_player = player_center - next;
        if to_player.norm() < tuning::FIREBALL_RADIUS + PLAYER_HIT_PADDING {
            removed.push((item.entity, next, true));
            return false;
        }

        if let Some(hit) = physics_world_cast_ray(
            &world.resources.physics,
            start,
            direction,
            step,
            player_entity,
        ) && hit.distance <= step
        {
            removed.push((item.entity, hit.point, false));
            return false;
        }

        true
    });

    boomer_world.resources.projectiles.items = items;

    for (entity, impact, damaged) in removed {
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity });
        fx::hit(boomer_world, world, impact, vec3(1.0, 0.5, 0.2));
        if damaged {
            game::damage_player(boomer_world, world, tuning::FIREBALL_DAMAGE);
        }
    }
}

pub fn despawn_all(boomer_world: &mut BoomerWorld, world: &mut World) {
    for projectile in boomer_world.resources.projectiles.items.drain(..) {
        queue_ecs_command(
            world,
            EcsCommand::DespawnRecursive {
                entity: projectile.entity,
            },
        );
    }
}
