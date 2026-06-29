use crate::ecs::{
    BoomerWorld, ENEMY, ENGINE_ENTITY, Enemy, EnemyState, EngineEntity, Phase, SpriteFrame,
};
use crate::systems::world::level::ARENA_HALF;
use crate::systems::world::textures::{MAT_IMP_ATTACK, MAT_IMP_HURT, MAT_IMP_IDLE};
use crate::systems::world::{audio, billboard, flash, game, player};
use nalgebra_glm::{Vec3, vec3};
use nightshade::prelude::*;

const ENEMY_HEALTH: f32 = 30.0;
const ENEMY_SPEED: f32 = 3.4;
const ENEMY_WIDTH: f32 = 1.5;
const ENEMY_HEIGHT: f32 = 1.9;
const ATTACK_RANGE: f32 = 1.9;
const ATTACK_DAMAGE: f32 = 9.0;
const ATTACK_COOLDOWN: f32 = 1.1;
const SEPARATION_DISTANCE: f32 = 1.4;
const HIT_FLASH_TIME: f32 = 0.12;
const DEATH_TIME: f32 = 0.5;

pub const HIT_RADIUS: f32 = 0.7;
pub const CENTER_HEIGHT: f32 = 1.0;

const WAVE_SIZES: [u32; 3] = [5, 7, 9];

pub fn start_first_wave(boomer_world: &mut BoomerWorld, world: &mut World) {
    boomer_world.resources.game.wave = 1;
    spawn_wave(boomer_world, world, WAVE_SIZES[0]);
}

fn spawn_wave(boomer_world: &mut BoomerWorld, world: &mut World, count: u32) {
    let base_angle = boomer_world.resources.game.wave as f32 * 0.7;
    for index in 0..count {
        let angle = base_angle + index as f32 * std::f32::consts::TAU / count as f32;
        let radius = 11.0 + (index % 3) as f32 * 2.0;
        let position = vec3(angle.cos() * radius, 0.0, angle.sin() * radius);
        spawn_enemy(boomer_world, world, position);
    }
}

fn spawn_enemy(boomer_world: &mut BoomerWorld, world: &mut World, position: Vec3) {
    let engine = billboard::spawn(
        world,
        MAT_IMP_IDLE,
        position,
        vec3(ENEMY_WIDTH, ENEMY_HEIGHT, 1.0),
    );
    let game_entity = boomer_world.spawn_entities(ENEMY | ENGINE_ENTITY, 1)[0];
    boomer_world.set_engine_entity(game_entity, EngineEntity(engine));
    boomer_world.set_enemy(
        game_entity,
        Enemy {
            position,
            health: ENEMY_HEALTH,
            state: EnemyState::Chase,
            attack_cooldown: 0.0,
            hit_flash: 0.0,
            death_timer: 0.0,
            sprite: SpriteFrame::Idle,
        },
    );
}

pub fn center(enemy: &Enemy) -> Vec3 {
    enemy.position + vec3(0.0, CENTER_HEIGHT, 0.0)
}

pub fn damage(
    boomer_world: &mut BoomerWorld,
    world: &mut World,
    game_entity: Entity,
    amount: f32,
    point: Vec3,
) {
    let Some(enemy) = boomer_world.get_enemy(game_entity) else {
        return;
    };
    if enemy.state == EnemyState::Dying {
        return;
    }
    let mut updated = *enemy;
    updated.health -= amount;
    updated.hit_flash = HIT_FLASH_TIME;
    let dead = updated.health <= 0.0;
    if dead {
        updated.state = EnemyState::Dying;
        updated.death_timer = DEATH_TIME;
    }
    boomer_world.set_enemy(game_entity, updated);

    flash::spark(boomer_world, world, point);
    if dead {
        audio::play(boomer_world, world, audio::IMP_DEATH, 1.0);
    } else {
        audio::play(boomer_world, world, audio::IMP_HURT, 0.8);
    }
}

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let player_position = player::position(boomer_world, world);
    let bound = ARENA_HALF - 1.2;

    let mut snapshots: Vec<(Entity, Entity, Enemy)> = boomer_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine = boomer_world.get_engine_entity(game_entity)?.0;
            let enemy = *boomer_world.get_enemy(game_entity)?;
            Some((game_entity, engine, enemy))
        })
        .collect();

    let mut attacks = 0u32;
    for (_, _, enemy) in snapshots.iter_mut() {
        if enemy.state == EnemyState::Dying {
            enemy.death_timer -= delta;
            continue;
        }
        enemy.hit_flash -= delta;
        enemy.attack_cooldown -= delta;

        let mut to_player = player_position - enemy.position;
        to_player.y = 0.0;
        let distance = to_player.norm();
        if distance > ATTACK_RANGE {
            enemy.state = EnemyState::Chase;
            if distance > 1e-3 {
                enemy.position += to_player / distance * ENEMY_SPEED * delta;
            }
        } else {
            enemy.state = EnemyState::Attack;
            if enemy.attack_cooldown <= 0.0 {
                enemy.attack_cooldown = ATTACK_COOLDOWN;
                attacks += 1;
            }
        }
    }

    separate(&mut snapshots, bound);

    for _ in 0..attacks {
        game::damage_player(boomer_world, world, ATTACK_DAMAGE);
    }

    for (game_entity, engine, enemy) in &snapshots {
        if enemy.state == EnemyState::Dying {
            let fraction = (enemy.death_timer / DEATH_TIME).max(0.0);
            if let Some(transform) = world.core.get_local_transform_mut(*engine) {
                transform.scale = vec3(ENEMY_WIDTH, ENEMY_HEIGHT * fraction, 1.0);
            }
            world
                .core
                .set_local_transform_dirty(*engine, LocalTransformDirty);
        } else {
            let frame = if enemy.hit_flash > 0.0 {
                SpriteFrame::Hurt
            } else if enemy.state == EnemyState::Attack {
                SpriteFrame::Attack
            } else {
                SpriteFrame::Idle
            };
            let material = match frame {
                SpriteFrame::Idle => MAT_IMP_IDLE,
                SpriteFrame::Attack => MAT_IMP_ATTACK,
                SpriteFrame::Hurt => MAT_IMP_HURT,
            };
            let mut next = *enemy;
            if next.sprite != frame {
                next.sprite = frame;
                billboard::set_material(world, *engine, material);
            }
            if let Some(slot) = boomer_world.get_enemy_mut(*game_entity) {
                *slot = next;
            }
            continue;
        }
        if let Some(slot) = boomer_world.get_enemy_mut(*game_entity) {
            *slot = *enemy;
        }
    }

    remove_dead(boomer_world, world);

    if boomer_world.query_entities(ENEMY).next().is_none()
        && matches!(boomer_world.resources.game.phase, Phase::Playing)
    {
        advance_wave(boomer_world, world);
    }
}

fn separate(snapshots: &mut [(Entity, Entity, Enemy)], bound: f32) {
    let count = snapshots.len();
    for first in 0..count {
        if snapshots[first].2.state == EnemyState::Dying {
            continue;
        }
        for second in (first + 1)..count {
            if snapshots[second].2.state == EnemyState::Dying {
                continue;
            }
            let mut offset = snapshots[first].2.position - snapshots[second].2.position;
            offset.y = 0.0;
            let distance = offset.norm();
            if distance > 1e-3 && distance < SEPARATION_DISTANCE {
                let push = offset / distance * (SEPARATION_DISTANCE - distance) * 0.5;
                snapshots[first].2.position += push;
                snapshots[second].2.position -= push;
            }
        }
    }
    for (_, _, enemy) in snapshots.iter_mut() {
        enemy.position.x = enemy.position.x.clamp(-bound, bound);
        enemy.position.z = enemy.position.z.clamp(-bound, bound);
        enemy.position.y = 0.0;
    }
}

fn remove_dead(boomer_world: &mut BoomerWorld, world: &mut World) {
    let dead: Vec<(Entity, Entity)> = boomer_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let enemy = boomer_world.get_enemy(game_entity)?;
            if enemy.state == EnemyState::Dying && enemy.death_timer <= 0.0 {
                let engine = boomer_world.get_engine_entity(game_entity)?.0;
                Some((game_entity, engine))
            } else {
                None
            }
        })
        .collect();
    for (game_entity, engine) in dead {
        despawn_recursive_immediate(world, engine);
        boomer_world.despawn_entities(&[game_entity]);
    }
}

fn advance_wave(boomer_world: &mut BoomerWorld, world: &mut World) {
    if (boomer_world.resources.game.wave as usize) < WAVE_SIZES.len() {
        boomer_world.resources.game.wave += 1;
        let count = WAVE_SIZES[boomer_world.resources.game.wave as usize - 1];
        spawn_wave(boomer_world, world, count);
    } else {
        boomer_world.resources.game.phase = Phase::Won;
    }
}

pub fn despawn_all(boomer_world: &mut BoomerWorld, world: &mut World) {
    let engines: Vec<Entity> = boomer_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            boomer_world
                .get_engine_entity(game_entity)
                .map(|link| link.0)
        })
        .collect();
    for engine in engines {
        despawn_recursive_immediate(world, engine);
    }
    let game_entities: Vec<Entity> = boomer_world.query_entities(ENEMY).collect();
    if !game_entities.is_empty() {
        boomer_world.despawn_entities(&game_entities);
    }
}

pub fn alive_count(boomer_world: &BoomerWorld) -> usize {
    boomer_world
        .query_entities(ENEMY)
        .filter(|game_entity| {
            boomer_world
                .get_enemy(*game_entity)
                .map(|enemy| enemy.state != EnemyState::Dying)
                .unwrap_or(false)
        })
        .count()
}
