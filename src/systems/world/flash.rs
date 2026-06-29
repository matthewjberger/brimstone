use crate::ecs::{BoomerWorld, Flash};
use crate::systems::world::billboard;
use crate::systems::world::textures::{MAT_MUZZLE, MAT_SPARK};
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

pub fn muzzle(boomer_world: &mut BoomerWorld, world: &mut World, point: Vec3) {
    spawn(boomer_world, world, MAT_MUZZLE, point, 0.8, 0.07);
}

pub fn spark(boomer_world: &mut BoomerWorld, world: &mut World, point: Vec3) {
    spawn(boomer_world, world, MAT_SPARK, point, 0.55, 0.16);
}

fn spawn(
    boomer_world: &mut BoomerWorld,
    world: &mut World,
    material: &str,
    point: Vec3,
    scale: f32,
    lifetime: f32,
) {
    let position = point - vec3(0.0, scale * 0.5, 0.0);
    let entity = billboard::spawn(world, material, position, vec3(scale, scale, scale));
    boomer_world.resources.flashes.items.push(Flash {
        entity,
        position,
        timer: lifetime,
        lifetime,
        base_scale: scale,
    });
}

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let mut index = 0;
    while index < boomer_world.resources.flashes.items.len() {
        boomer_world.resources.flashes.items[index].timer -= delta;
        if boomer_world.resources.flashes.items[index].timer <= 0.0 {
            let flash = boomer_world.resources.flashes.items.swap_remove(index);
            queue_ecs_command(
                world,
                EcsCommand::DespawnRecursive {
                    entity: flash.entity,
                },
            );
            continue;
        }
        let item = &boomer_world.resources.flashes.items[index];
        let fraction = item.timer / item.lifetime;
        let scale = item.base_scale * (0.5 + fraction * 0.7);
        let entity = item.entity;
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.scale = vec3(scale, scale, scale);
        }
        world
            .core
            .set_local_transform_dirty(entity, LocalTransformDirty);
        index += 1;
    }
}
