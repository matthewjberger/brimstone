use crate::ecs::{BoomerWorld, ENEMY, ENGINE_ENTITY, Enemy, EnemyKind, EnemyState, EngineEntity};
use crate::systems::common::random_range;
use crate::systems::world::textures::{
    MAT_CASTER_HURT, MAT_CASTER_IDLE, MAT_IMP_HURT, MAT_IMP_IDLE, MAT_SWARM_HURT, MAT_SWARM_IDLE,
};
use crate::systems::world::{audio, billboard, fx, game, pickups, player, projectiles};
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::physics::resources::physics_world_cast_ray;
use nightshade::prelude::*;

const CASTER_WINDUP: f32 = 0.45;
const PROBE_HEIGHT: f32 = 0.6;
const PROBE_DISTANCE: f32 = 1.9;

struct Stats {
    health: f32,
    speed: f32,
    width: f32,
    height: f32,
    attack_range: f32,
    attack_damage: f32,
    attack_cooldown: f32,
    score: u32,
    idle: &'static str,
    hurt: &'static str,
    color: Vec3,
}

fn stats(kind: EnemyKind) -> Stats {
    match kind {
        EnemyKind::Imp => Stats {
            health: tuning::IMP_HEALTH,
            speed: tuning::IMP_SPEED,
            width: tuning::IMP_WIDTH,
            height: tuning::IMP_HEIGHT,
            attack_range: tuning::IMP_ATTACK_RANGE,
            attack_damage: tuning::IMP_DAMAGE,
            attack_cooldown: tuning::IMP_ATTACK_COOLDOWN,
            score: tuning::IMP_SCORE,
            idle: MAT_IMP_IDLE,
            hurt: MAT_IMP_HURT,
            color: vec3(1.0, 0.3, 0.2),
        },
        EnemyKind::Swarmer => Stats {
            health: tuning::SWARM_HEALTH,
            speed: tuning::SWARM_SPEED,
            width: tuning::SWARM_WIDTH,
            height: tuning::SWARM_HEIGHT,
            attack_range: tuning::SWARM_ATTACK_RANGE,
            attack_damage: tuning::SWARM_DAMAGE,
            attack_cooldown: tuning::SWARM_ATTACK_COOLDOWN,
            score: tuning::SWARM_SCORE,
            idle: MAT_SWARM_IDLE,
            hurt: MAT_SWARM_HURT,
            color: vec3(0.4, 1.0, 0.4),
        },
        EnemyKind::Caster => Stats {
            health: tuning::CASTER_HEALTH,
            speed: tuning::CASTER_SPEED,
            width: tuning::CASTER_WIDTH,
            height: tuning::CASTER_HEIGHT,
            attack_range: 0.0,
            attack_damage: 0.0,
            attack_cooldown: 0.0,
            score: tuning::CASTER_SCORE,
            idle: MAT_CASTER_IDLE,
            hurt: MAT_CASTER_HURT,
            color: vec3(0.75, 0.4, 1.0),
        },
    }
}

pub fn center(enemy: &Enemy) -> Vec3 {
    enemy.position + vec3(0.0, tuning::ENEMY_CENTER_HEIGHT, 0.0)
}

pub fn spawn(boomer_world: &mut BoomerWorld, world: &mut World, kind: EnemyKind, position: Vec3) {
    let s = stats(kind);
    let engine = billboard::spawn(world, s.idle, position, vec3(s.width, s.height, 1.0));
    let strafe_roll = random_range(&mut boomer_world.resources.game.random_state, 0.0, 1.0);
    let fire_jitter = random_range(&mut boomer_world.resources.game.random_state, 0.4, 1.0)
        * tuning::CASTER_FIRE_COOLDOWN;
    let game_entity = boomer_world.spawn_entities(ENEMY | ENGINE_ENTITY, 1)[0];
    boomer_world.set_engine_entity(game_entity, EngineEntity(engine));
    boomer_world.set_enemy(
        game_entity,
        Enemy {
            kind,
            position,
            velocity: Vec3::zeros(),
            health: s.health,
            state: EnemyState::Chase,
            attack_cooldown: 0.0,
            fire_cooldown: fire_jitter,
            windup: 0.0,
            hit_flash: 0.0,
            death_timer: 0.0,
            showing_hurt: false,
            strafe_dir: if strafe_roll < 0.5 { -1.0 } else { 1.0 },
        },
    );
    fx::hit(boomer_world, world, position + vec3(0.0, 1.0, 0.0), s.color);
}

pub fn damage(
    boomer_world: &mut BoomerWorld,
    world: &mut World,
    game_entity: Entity,
    amount: f32,
    point: Vec3,
    knockback: Vec3,
) {
    let Some(enemy) = boomer_world.get_enemy(game_entity) else {
        return;
    };
    if enemy.state == EnemyState::Dying {
        return;
    }
    let kind = enemy.kind;
    let s = stats(kind);
    let mut updated = *enemy;
    updated.health -= amount;
    updated.hit_flash = tuning::ENEMY_HIT_FLASH;
    updated.velocity += knockback;
    let dead = updated.health <= 0.0;
    if dead {
        updated.state = EnemyState::Dying;
        updated.death_timer = tuning::ENEMY_DEATH_TIME;
    }
    let position = updated.position;
    boomer_world.set_enemy(game_entity, updated);

    fx::hit(boomer_world, world, point, s.color);
    if dead {
        let count = match kind {
            EnemyKind::Swarmer => 60,
            EnemyKind::Imp => 90,
            EnemyKind::Caster => 110,
        };
        fx::death(
            boomer_world,
            world,
            position + vec3(0.0, tuning::ENEMY_CENTER_HEIGHT, 0.0),
            s.color,
            count,
        );
        game::award(boomer_world, s.score);
        audio::play(boomer_world, world, audio::ENEMY_DEATH, 1.0);
        pickups::maybe_drop(boomer_world, world, position);
    } else {
        audio::play(boomer_world, world, audio::ENEMY_HURT, 0.7);
    }
}

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let player_center = player::position(boomer_world, world);
    let player_ground = vec3(player_center.x, 0.0, player_center.z);
    let bound = tuning::ARENA_HALF - 1.5;

    let mut snapshots: Vec<(Entity, Entity, Enemy)> = boomer_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine = boomer_world.get_engine_entity(game_entity)?.0;
            let enemy = *boomer_world.get_enemy(game_entity)?;
            Some((game_entity, engine, enemy))
        })
        .collect();

    let mut melee_damage = 0.0;
    let mut fireballs: Vec<(Vec3, Vec3)> = Vec::new();

    for (_, _, enemy) in snapshots.iter_mut() {
        if enemy.state == EnemyState::Dying {
            enemy.death_timer -= delta;
            continue;
        }
        let s = stats(enemy.kind);
        enemy.hit_flash -= delta;
        enemy.attack_cooldown -= delta;
        enemy.fire_cooldown -= delta;

        enemy.position += enemy.velocity * delta;
        enemy.velocity *= (1.0 - (tuning::ENEMY_KNOCKBACK_DECAY * delta).min(1.0)).max(0.0);

        let mut to_player = player_ground - enemy.position;
        to_player.y = 0.0;
        let distance = to_player.norm();
        let direction = if distance > 1e-3 {
            to_player / distance
        } else {
            vec3(0.0, 0.0, 1.0)
        };

        match enemy.kind {
            EnemyKind::Imp | EnemyKind::Swarmer => {
                if distance > s.attack_range {
                    enemy.state = EnemyState::Chase;
                    let steer = avoid(world, enemy.position, direction);
                    enemy.position += steer * s.speed * delta;
                } else {
                    enemy.state = EnemyState::Attack;
                    if enemy.attack_cooldown <= 0.0 {
                        enemy.attack_cooldown = s.attack_cooldown;
                        melee_damage += s.attack_damage;
                    }
                }
            }
            EnemyKind::Caster => {
                enemy.state = EnemyState::Chase;
                let preferred = tuning::CASTER_PREFERRED_RANGE;
                let move_dir = if distance > preferred + 1.5 {
                    direction
                } else if distance < preferred - 1.5 {
                    -direction
                } else {
                    vec3(direction.z, 0.0, -direction.x) * enemy.strafe_dir
                };
                let steer = avoid(world, enemy.position, move_dir);
                enemy.position += steer * s.speed * delta;

                if enemy.windup > 0.0 {
                    enemy.windup -= delta;
                    if enemy.windup <= 0.0 {
                        enemy.fire_cooldown = tuning::CASTER_FIRE_COOLDOWN;
                        fireballs.push((center(enemy), player_center));
                    }
                } else if enemy.fire_cooldown <= 0.0 {
                    enemy.windup = CASTER_WINDUP;
                }
            }
        }

        enemy.position.x = enemy.position.x.clamp(-bound, bound);
        enemy.position.z = enemy.position.z.clamp(-bound, bound);
        enemy.position.y = 0.0;
    }

    separate(&mut snapshots, bound);

    if melee_damage > 0.0 {
        game::damage_player(boomer_world, world, melee_damage);
    }
    for (origin, target) in fireballs {
        projectiles::spawn(boomer_world, world, origin, target);
        audio::play(boomer_world, world, audio::FIREBALL, 0.32);
    }

    for (game_entity, engine, enemy) in &snapshots {
        let s = stats(enemy.kind);
        if enemy.state == EnemyState::Dying {
            let fraction = (enemy.death_timer / tuning::ENEMY_DEATH_TIME).max(0.0);
            set_scale(world, *engine, vec3(s.width, s.height * fraction, 1.0));
        } else {
            let mut next = *enemy;
            let hurt = next.hit_flash > 0.0;
            if hurt != next.showing_hurt {
                next.showing_hurt = hurt;
                let material = if hurt { s.hurt } else { s.idle };
                world
                    .core
                    .set_material_ref(*engine, MaterialRef::new(material.to_string()));
            }
            let windup_scale = 1.0 + (next.windup / CASTER_WINDUP).clamp(0.0, 1.0) * 0.2;
            set_scale(
                world,
                *engine,
                vec3(s.width * windup_scale, s.height * windup_scale, 1.0),
            );
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
}

fn set_scale(world: &mut World, entity: Entity, scale: Vec3) {
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.scale = scale;
    }
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
}

fn clearance(world: &World, origin: Vec3, direction: Vec3, max: f32) -> f32 {
    if direction.norm() < 1e-3 {
        return max;
    }
    physics_world_cast_ray(
        &world.resources.physics,
        origin,
        direction.normalize(),
        max,
        None,
    )
    .map(|hit| hit.distance)
    .unwrap_or(max)
}

fn rotate_y(direction: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    vec3(
        direction.x * cos + direction.z * sin,
        0.0,
        -direction.x * sin + direction.z * cos,
    )
}

/// Steer toward `desired`, sidestepping cover via a few short probe rays.
fn avoid(world: &World, position: Vec3, desired: Vec3) -> Vec3 {
    if desired.norm() < 1e-3 {
        return desired;
    }
    let origin = position + vec3(0.0, PROBE_HEIGHT, 0.0);
    if clearance(world, origin, desired, PROBE_DISTANCE) >= PROBE_DISTANCE {
        return desired;
    }
    let left = rotate_y(desired, 0.7);
    let right = rotate_y(desired, -0.7);
    let left_clear = clearance(world, origin, left, PROBE_DISTANCE);
    let right_clear = clearance(world, origin, right, PROBE_DISTANCE);
    if left_clear >= right_clear {
        left
    } else {
        right
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
            if distance > 1e-3 && distance < tuning::ENEMY_SEPARATION {
                let push = offset / distance * (tuning::ENEMY_SEPARATION - distance) * 0.5;
                snapshots[first].2.position += push;
                snapshots[second].2.position -= push;
            }
        }
    }
    for (_, _, enemy) in snapshots.iter_mut() {
        enemy.position.x = enemy.position.x.clamp(-bound, bound);
        enemy.position.z = enemy.position.z.clamp(-bound, bound);
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

pub fn total_count(boomer_world: &BoomerWorld) -> usize {
    boomer_world.query_entities(ENEMY).count()
}
