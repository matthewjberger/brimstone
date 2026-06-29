use crate::ecs::{BoomerWorld, ENGINE_ENTITY, EngineEntity, PICKUP, Pickup, PickupKind};
use crate::systems::world::textures::{MAT_AMMO, MAT_MEDKIT};
use crate::systems::world::{audio, billboard, player};
use nalgebra_glm::vec3;
use nightshade::prelude::*;

const PICKUP_RADIUS: f32 = 1.4;
const HEALTH_AMOUNT: f32 = 25.0;
const AMMO_AMOUNT: u32 = 12;
const HOVER_HEIGHT: f32 = 0.6;
const BOB_HEIGHT: f32 = 0.16;
const BOB_SPEED: f32 = 2.5;

const HEALTH_SPOTS: [(f32, f32); 2] = [(6.0, -6.0), (-7.0, 7.0)];
const AMMO_SPOTS: [(f32, f32); 3] = [(7.0, 7.0), (-6.0, -6.0), (0.0, 12.0)];

pub fn spawn_all(boomer_world: &mut BoomerWorld, world: &mut World) {
    for (x, z) in HEALTH_SPOTS {
        spawn_pickup(boomer_world, world, PickupKind::Health, x, z);
    }
    for (x, z) in AMMO_SPOTS {
        spawn_pickup(boomer_world, world, PickupKind::Ammo, x, z);
    }
}

fn spawn_pickup(
    boomer_world: &mut BoomerWorld,
    world: &mut World,
    kind: PickupKind,
    x: f32,
    z: f32,
) {
    let material = match kind {
        PickupKind::Health => MAT_MEDKIT,
        PickupKind::Ammo => MAT_AMMO,
    };
    let position = vec3(x, HOVER_HEIGHT, z);
    let engine = billboard::spawn(world, material, position, vec3(0.9, 0.9, 1.0));
    let game_entity = boomer_world.spawn_entities(PICKUP | ENGINE_ENTITY, 1)[0];
    boomer_world.set_engine_entity(game_entity, EngineEntity(engine));
    boomer_world.set_pickup(
        game_entity,
        Pickup {
            position,
            kind,
            bob_phase: x + z,
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

    let mut removed: Vec<(Entity, Entity)> = Vec::new();
    for (game_entity, engine, pickup) in snapshots {
        let mut updated = pickup;
        updated.bob_phase += delta * BOB_SPEED;
        updated.position.y = HOVER_HEIGHT + updated.bob_phase.sin() * BOB_HEIGHT;

        let mut offset = player_position - updated.position;
        offset.y = 0.0;
        let close = offset.norm() < PICKUP_RADIUS;
        let wanted = match updated.kind {
            PickupKind::Health => {
                boomer_world.resources.stats.health < boomer_world.resources.stats.max_health
            }
            PickupKind::Ammo => {
                boomer_world.resources.weapon.ammo < boomer_world.resources.weapon.max_ammo
            }
        };

        if close && wanted {
            match updated.kind {
                PickupKind::Health => {
                    let max = boomer_world.resources.stats.max_health;
                    boomer_world.resources.stats.health =
                        (boomer_world.resources.stats.health + HEALTH_AMOUNT).min(max);
                }
                PickupKind::Ammo => {
                    let max = boomer_world.resources.weapon.max_ammo;
                    boomer_world.resources.weapon.ammo =
                        (boomer_world.resources.weapon.ammo + AMMO_AMOUNT).min(max);
                }
            }
            audio::play(boomer_world, world, audio::PICKUP, 0.8);
            removed.push((game_entity, engine));
        } else if let Some(slot) = boomer_world.get_pickup_mut(game_entity) {
            *slot = updated;
        }
    }

    for (game_entity, engine) in removed {
        despawn_recursive_immediate(world, engine);
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
        despawn_recursive_immediate(world, engine);
    }
    let game_entities: Vec<Entity> = boomer_world.query_entities(PICKUP).collect();
    if !game_entities.is_empty() {
        boomer_world.despawn_entities(&game_entities);
    }
}
