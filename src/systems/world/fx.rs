use crate::ecs::BrimstoneWorld;
use nalgebra_glm::{Vec3, Vec4, vec3, vec4};
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command, spawn_light_entity};
use nightshade::prelude::*;

const BURST_TTL: f32 = 1.6;
const EXPLOSION_TTL: f32 = 3.8;
const TRACER_TTL: f32 = 0.05;

fn track(brimstone_world: &mut BrimstoneWorld, entity: Entity, ttl: f32) {
    brimstone_world.resources.transient.items.push((entity, ttl));
}

fn gradient(color: Vec3) -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
            (0.25, vec4(color.x, color.y, color.z, 1.0)),
            (0.7, vec4(color.x * 0.7, color.y * 0.7, color.z * 0.7, 0.7)),
            (1.0, vec4(color.x * 0.3, color.y * 0.3, color.z * 0.3, 0.0)),
        ],
    }
}

/// Hot core → weapon-tinted flame → smoky fade, for the muzzle flame flash.
fn muzzle_gradient(color: Vec3) -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
            (
                0.3,
                vec4(
                    (color.x + 1.4) * 0.5,
                    (color.y + 1.4) * 0.5,
                    (color.z + 1.4) * 0.5,
                    1.0,
                ),
            ),
            (0.7, vec4(color.x, color.y, color.z, 0.75)),
            (1.0, vec4(color.x * 0.3, color.y * 0.3, color.z * 0.3, 0.0)),
        ],
    }
}

pub fn muzzle(
    brimstone_world: &mut BrimstoneWorld,
    world: &mut World,
    position: Vec3,
    forward: Vec3,
    color: Vec3,
) {
    // Flame flash (thruster-style fire), tinted to the weapon.
    let flame = ParticleEmitter {
        emitter_type: EmitterType::Fire,
        shape: EmitterShape::Point,
        position,
        direction: forward,
        spawn_rate: 0.0,
        burst_count: 20,
        particle_lifetime_min: 0.05,
        particle_lifetime_max: 0.2,
        initial_velocity_min: 3.0,
        initial_velocity_max: 11.0,
        velocity_spread: 0.4,
        gravity: Vec3::zeros(),
        drag: 0.35,
        size_start: 0.32,
        size_end: 0.02,
        color_gradient: muzzle_gradient(color),
        emissive_strength: 9.0,
        one_shot: true,
        ..Default::default()
    };
    let flame_entity = spawn_entities(world, NAME | PARTICLE_EMITTER, 1)[0];
    world.core.set_particle_emitter(flame_entity, flame);
    track(brimstone_world, flame_entity, 0.4);

    // Fast white sparks off the muzzle.
    let sparks = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Point,
        position,
        direction: forward,
        spawn_rate: 0.0,
        burst_count: 14,
        particle_lifetime_min: 0.04,
        particle_lifetime_max: 0.14,
        initial_velocity_min: 6.0,
        initial_velocity_max: 15.0,
        velocity_spread: 0.5,
        gravity: Vec3::zeros(),
        drag: 0.2,
        size_start: 0.1,
        size_end: 0.01,
        color_gradient: ColorGradient::flash_burst(),
        emissive_strength: 10.0,
        one_shot: true,
        ..Default::default()
    };
    let spark_entity = spawn_entities(world, NAME | PARTICLE_EMITTER, 1)[0];
    world.core.set_particle_emitter(spark_entity, sparks);
    track(brimstone_world, spark_entity, 0.4);

    // A brief flash of light that lights up the scene for a couple of frames.
    let light = spawn_light_entity(world, position, "MuzzleFlash");
    world.core.set_light(
        light,
        Light {
            light_type: LightType::Point,
            color: vec3(color.x.min(1.0), color.y.min(1.0), color.z.min(1.0)),
            intensity: 18.0,
            range: 7.0,
            ..Default::default()
        },
    );
    track(brimstone_world, light, 0.06);
}

pub fn hit(brimstone_world: &mut BrimstoneWorld, world: &mut World, position: Vec3, color: Vec3) {
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.18 },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 0.0,
        burst_count: 20,
        particle_lifetime_min: 0.18,
        particle_lifetime_max: 0.55,
        initial_velocity_min: 3.0,
        initial_velocity_max: 9.0,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::new(0.0, -10.0, 0.0),
        drag: 0.1,
        size_start: 0.11,
        size_end: 0.01,
        color_gradient: gradient(color),
        emissive_strength: 7.0,
        one_shot: true,
        ..Default::default()
    };
    let entity = spawn_entities(world, NAME | PARTICLE_EMITTER, 1)[0];
    world.core.set_particle_emitter(entity, emitter);
    track(brimstone_world, entity, BURST_TTL);
}

pub fn death(
    brimstone_world: &mut BrimstoneWorld,
    world: &mut World,
    position: Vec3,
    color: Vec3,
    count: u32,
) {
    let entity = spawn_entities(world, NAME | PARTICLE_EMITTER, 1)[0];
    world.core.set_particle_emitter(
        entity,
        ParticleEmitter::firework_explosion(position, color, count),
    );
    track(brimstone_world, entity, EXPLOSION_TTL);
}

pub fn tracer(
    brimstone_world: &mut BrimstoneWorld,
    world: &mut World,
    start: Vec3,
    end: Vec3,
    color: Vec4,
) {
    let entity = spawn_entities(world, NAME | LINES | VISIBILITY | GLOBAL_TRANSFORM, 1)[0];
    world
        .core
        .set_lines(entity, Lines::new(vec![Line { start, end, color }]));
    world
        .core
        .set_visibility(entity, Visibility { visible: true });
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    track(brimstone_world, entity, TRACER_TTL);
}

/// A short-lived procedural lightning arc between two points (the tesla cannon).
/// The engine's vfx system regenerates its jagged path into the entity's lines
/// each frame; we just despawn it after a brief life.
pub fn lightning(
    brimstone_world: &mut BrimstoneWorld,
    world: &mut World,
    start: Vec3,
    end: Vec3,
    color: Vec3,
    ttl: f32,
) {
    let entity = spawn_entities(
        world,
        NAME | LINES | LIGHTNING_BOLT | VISIBILITY | GLOBAL_TRANSFORM,
        1,
    )[0];
    world.core.set_lines(entity, Lines::new(Vec::new()));
    world
        .core
        .set_visibility(entity, Visibility { visible: true });
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_lightning_bolt(
        entity,
        LightningBolt {
            start,
            end,
            color,
            intensity: 6.0,
            jaggedness: 0.55,
            branch_count: 3,
            branch_spread: 0.7,
            regen_interval: 0.025,
            seed: entity.id as u32,
            ..Default::default()
        },
    );
    track(brimstone_world, entity, ttl);
}

pub fn tick(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let mut index = 0;
    while index < brimstone_world.resources.transient.items.len() {
        brimstone_world.resources.transient.items[index].1 -= delta;
        if brimstone_world.resources.transient.items[index].1 <= 0.0 {
            let (entity, _) = brimstone_world.resources.transient.items.swap_remove(index);
            queue_ecs_command(world, EcsCommand::DespawnRecursive { entity });
        } else {
            index += 1;
        }
    }
}
