use crate::ecs::{BoomerWorld, ENGINE_ENTITY, EngineEntity, PICKUP, Pickup, PickupKind};
use crate::systems::common::next_random;
use crate::systems::world::textures::{MAT_AMMO, MAT_MEDKIT};
use crate::systems::world::{audio, billboard, fx, player};
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const INITIAL_HEALTH: [(f32, f32); 2] = [(8.0, -8.0), (-9.0, 9.0)];
const INITIAL_AMMO: [(f32, f32); 3] = [(9.0, 9.0), (-8.0, -8.0), (0.0, 13.0)];

pub fn spawn_initial(boomer_world: &mut BoomerWorld, world: &mut World) {
    for (x, z) in INITIAL_HEALTH {
        spawn_pickup(boomer_world, world, PickupKind::Health, vec3(x, 0.0, z));
    }
    for (x, z) in INITIAL_AMMO {
        spawn_pickup(boomer_world, world, PickupKind::Ammo, vec3(x, 0.0, z));
    }
}

/// Push-forward economy: kills drop what the player is short on, so staying
/// aggressive sustains you and the drops pull you into the fight.
pub fn maybe_drop(boomer_world: &mut BoomerWorld, world: &mut World, position: Vec3) {
    let health_fraction =
        boomer_world.resources.stats.health / boomer_world.resources.stats.max_health;
    let ammo_fraction =
        boomer_world.resources.weapon.ammo as f32 / boomer_world.resources.weapon.max_ammo as f32;
    let roll = next_random(&mut boomer_world.resources.game.random_state);

    let kind = if health_fraction < 0.5 && roll < 0.32 {
        Some(PickupKind::Health)
    } else if (ammo_fraction < 0.4 && roll < 0.55) || roll < tuning::AMMO_DROP_CHANCE {
        Some(PickupKind::Ammo)
    } else {
        None
    };

    if let Some(kind) = kind {
        spawn_pickup(boomer_world, world, kind, vec3(position.x, 0.0, position.z));
    }
}

fn spawn_pickup(boomer_world: &mut BoomerWorld, world: &mut World, kind: PickupKind, ground: Vec3) {
    let material = match kind {
        PickupKind::Health => MAT_MEDKIT,
        PickupKind::Ammo => MAT_AMMO,
    };
    let position = vec3(ground.x, tuning::PICKUP_HOVER, ground.z);
    let engine = billboard::spawn(world, material, position, vec3(0.9, 0.9, 1.0));
    let game_entity = boomer_world.spawn_entities(PICKUP | ENGINE_ENTITY, 1)[0];
    boomer_world.set_engine_entity(game_entity, EngineEntity(engine));
    boomer_world.set_pickup(
        game_entity,
        Pickup {
            position,
            kind,
            bob_phase: ground.x + ground.z,
        },
    );
}

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let player_position = player::position(boomer_world, world);

    let snapshots: Vec<(Entity, Entity, Pickup)> = boomer_world
        .query_entities(PICKUP | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine = boomer_world.get_engine_entity(game_entity)?.0;
            let pickup = *boomer_world.get_pickup(game_entity)?;
            Some((game_entity, engine, pickup))
        })
        .collect();

    let mut collected: Vec<(Entity, Entity, Vec3, PickupKind)> = Vec::new();
    for (game_entity, engine, pickup) in snapshots {
        let mut updated = pickup;
        updated.bob_phase += delta * tuning::PICKUP_BOB_SPEED;
        updated.position.y =
            tuning::PICKUP_HOVER + updated.bob_phase.sin() * tuning::PICKUP_BOB_HEIGHT;

        let mut offset = player_position - updated.position;
        offset.y = 0.0;
        let close = offset.norm() < tuning::PICKUP_RADIUS;
        let wanted = match updated.kind {
            PickupKind::Health => {
                boomer_world.resources.stats.health < boomer_world.resources.stats.max_health
            }
            PickupKind::Ammo => {
                boomer_world.resources.weapon.ammo < boomer_world.resources.weapon.max_ammo
            }
        };

        if close && wanted {
            collected.push((game_entity, engine, updated.position, updated.kind));
        } else if let Some(slot) = boomer_world.get_pickup_mut(game_entity) {
            *slot = updated;
        }
    }

    for (game_entity, engine, position, kind) in collected {
        match kind {
            PickupKind::Health => {
                let max = boomer_world.resources.stats.max_health;
                boomer_world.resources.stats.health =
                    (boomer_world.resources.stats.health + tuning::HEALTH_PICKUP_AMOUNT).min(max);
            }
            PickupKind::Ammo => {
                let max = boomer_world.resources.weapon.max_ammo;
                boomer_world.resources.weapon.ammo =
                    (boomer_world.resources.weapon.ammo + tuning::AMMO_PICKUP_AMOUNT).min(max);
            }
        }
        let color = match kind {
            PickupKind::Health => vec3(0.4, 1.0, 0.5),
            PickupKind::Ammo => vec3(1.0, 0.85, 0.3),
        };
        fx::hit(boomer_world, world, position, color);
        audio::play(boomer_world, world, audio::PICKUP, 0.7);
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity: engine });
        boomer_world.despawn_entities(&[game_entity]);
    }
}

pub fn despawn_all(boomer_world: &mut BoomerWorld, world: &mut World) {
    let engines: Vec<Entity> = boomer_world
        .query_entities(PICKUP | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            boomer_world
                .get_engine_entity(game_entity)
                .map(|link| link.0)
        })
        .collect();
    for engine in engines {
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity: engine });
    }
    let game_entities: Vec<Entity> = boomer_world.query_entities(PICKUP).collect();
    if !game_entities.is_empty() {
        boomer_world.despawn_entities(&game_entities);
    }
}
