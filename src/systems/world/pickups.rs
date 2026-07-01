use crate::ecs::{
    BrimstoneWorld, ENGINE_ENTITY, EngineEntity, PICKUP, Pickup, PickupKind, WeaponKind,
};
use crate::systems::common::next_random;
use crate::systems::world::textures::{MAT_AMMO, MAT_KEYCARD, MAT_MEDKIT};
use crate::systems::world::{audio, billboard, fx, player};
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const INITIAL_HEALTH: [(f32, f32); 2] = [(8.0, -8.0), (-9.0, 9.0)];
const INITIAL_AMMO: [(f32, f32); 3] = [(9.0, 9.0), (-8.0, -8.0), (0.0, 13.0)];

pub fn spawn_initial(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    for (x, z) in INITIAL_HEALTH {
        spawn_pickup(brimstone_world, world, PickupKind::Health, vec3(x, 0.0, z));
    }
    for (x, z) in INITIAL_AMMO {
        spawn_pickup(brimstone_world, world, PickupKind::Ammo, vec3(x, 0.0, z));
    }
}

/// Push-forward economy: kills drop what the player is short on, so staying
/// aggressive sustains you and the drops pull you into the fight.
pub fn maybe_drop(brimstone_world: &mut BrimstoneWorld, world: &mut World, position: Vec3) {
    let health_fraction =
        brimstone_world.resources.stats.health / brimstone_world.resources.stats.max_health;
    let current = brimstone_world.resources.weapon.current;
    let ammo_fraction =
        brimstone_world.resources.weapon.ammo(current) as f32 / current.max_ammo().max(1) as f32;
    let roll = next_random(&mut brimstone_world.resources.game.random_state);

    let kind = if health_fraction < 0.5 && roll < 0.32 {
        Some(PickupKind::Health)
    } else if (ammo_fraction < 0.4 && roll < 0.55) || roll < tuning::AMMO_DROP_CHANCE {
        Some(PickupKind::Ammo)
    } else {
        None
    };

    if let Some(kind) = kind {
        spawn_pickup(brimstone_world, world, kind, vec3(position.x, 0.0, position.z));
    }
}

/// Drop the mission keycard at a fixed spot for `Keycard` objectives.
pub fn spawn_keycard(brimstone_world: &mut BrimstoneWorld, world: &mut World, position: Vec3) {
    spawn_pickup(
        brimstone_world,
        world,
        PickupKind::Keycard,
        vec3(position.x, 0.0, position.z),
    );
}

fn spawn_pickup(brimstone_world: &mut BrimstoneWorld, world: &mut World, kind: PickupKind, ground: Vec3) {
    let material = match kind {
        PickupKind::Health => MAT_MEDKIT,
        PickupKind::Ammo => MAT_AMMO,
        PickupKind::Keycard => MAT_KEYCARD,
    };
    let scale = if matches!(kind, PickupKind::Keycard) {
        vec3(1.3, 1.3, 1.0)
    } else {
        vec3(0.9, 0.9, 1.0)
    };
    let position = vec3(ground.x, tuning::PICKUP_HOVER, ground.z);
    let engine = billboard::spawn(world, material, position, scale);
    let game_entity = brimstone_world.spawn_entities(PICKUP | ENGINE_ENTITY, 1)[0];
    brimstone_world.set_engine_entity(game_entity, EngineEntity(engine));
    brimstone_world.set_pickup(
        game_entity,
        Pickup {
            position,
            kind,
            bob_phase: ground.x + ground.z,
        },
    );
}

pub fn update(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let player_position = player::position(brimstone_world, world);

    let snapshots: Vec<(Entity, Entity, Pickup)> = brimstone_world
        .query_entities(PICKUP | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine = brimstone_world.get_engine_entity(game_entity)?.0;
            let pickup = *brimstone_world.get_pickup(game_entity)?;
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
                brimstone_world.resources.stats.health < brimstone_world.resources.stats.max_health
            }
            PickupKind::Ammo => {
                let weapon = &brimstone_world.resources.weapon;
                WeaponKind::ALL
                    .iter()
                    .any(|&kind| weapon.ammo(kind) < kind.max_ammo())
            }
            PickupKind::Keycard => !brimstone_world.resources.game.has_key,
        };

        if close && wanted {
            collected.push((game_entity, engine, updated.position, updated.kind));
        } else if let Some(slot) = brimstone_world.get_pickup_mut(game_entity) {
            *slot = updated;
        }
    }

    for (game_entity, engine, position, kind) in collected {
        match kind {
            PickupKind::Health => {
                let max = brimstone_world.resources.stats.max_health;
                brimstone_world.resources.stats.health =
                    (brimstone_world.resources.stats.health + tuning::HEALTH_PICKUP_AMOUNT).min(max);
            }
            PickupKind::Ammo => {
                let weapon = &mut brimstone_world.resources.weapon;
                for kind in WeaponKind::ALL {
                    weapon.add_ammo(kind, kind.pickup_ammo());
                }
            }
            PickupKind::Keycard => {
                brimstone_world.resources.game.has_key = true;
            }
        }
        let color = match kind {
            PickupKind::Health => vec3(0.4, 1.0, 0.5),
            PickupKind::Ammo => vec3(1.0, 0.85, 0.3),
            PickupKind::Keycard => vec3(1.0, 0.85, 0.2),
        };
        fx::hit(brimstone_world, world, position, color);
        audio::play(brimstone_world, world, audio::PICKUP, 0.7);
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity: engine });
        brimstone_world.despawn_entities(&[game_entity]);
    }
}

pub fn despawn_all(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let engines: Vec<Entity> = brimstone_world
        .query_entities(PICKUP | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            brimstone_world
                .get_engine_entity(game_entity)
                .map(|link| link.0)
        })
        .collect();
    for engine in engines {
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity: engine });
    }
    let game_entities: Vec<Entity> = brimstone_world.query_entities(PICKUP).collect();
    if !game_entities.is_empty() {
        brimstone_world.despawn_entities(&game_entities);
    }
}
