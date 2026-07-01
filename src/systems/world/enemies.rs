use crate::art;
use crate::ecs::{BrimstoneWorld, ENEMY, ENGINE_ENTITY, Enemy, EnemyKind, EnemyState, EngineEntity};
use crate::systems::common::random_range;
use crate::systems::world::{audio, billboard, fx, game, pickups, player, projectiles};
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::navmesh::find_path_with_algorithm;
use nightshade::ecs::navmesh::funnel::{simplify_path, smooth_path};
use nightshade::ecs::physics::resources::physics_world_cast_ray;
use nightshade::prelude::*;

const PROBE_HEIGHT: f32 = 0.6;
const PROBE_DISTANCE: f32 = 1.9;
const HURT_CODE: u8 = 200;

/// How an enemy moves and fights. Selects the per-frame branch in [`update`], so
/// a new archetype is a table entry plus (at most) one variant here, rather than
/// another arm grafted onto the movement loop.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Behavior {
    /// Ground melee: close in, telegraph, then lunge (imp, swarmer, brute).
    MeleeGround,
    /// Flyer that hovers, telegraphs, then dive-bombs (gargoyle).
    FlyingDive,
    /// Flyer that holds range and lobs fireballs (sentinel).
    FlyingRanged,
    /// Ground caster that holds range and lobs fireballs (caster).
    GroundRanged,
}

struct Stats {
    behavior: Behavior,
    health: f32,
    speed: f32,
    width: f32,
    height: f32,
    attack_range: f32,
    attack_damage: f32,
    attack_cooldown: f32,
    windup_time: f32,
    lunge_speed: f32,
    lunge_reach: f32,
    /// Ranged archetypes: seconds between shots and the stand-off range held.
    fire_cooldown: f32,
    preferred_range: f32,
    /// Flyers: the altitude hovered at between attacks.
    hover: f32,
    /// Particle count on death (a boss overrides this with its own larger burst).
    death_particles: u32,
    score: u32,
    key: &'static str,
    color: Vec3,
}

fn stats(kind: EnemyKind) -> Stats {
    match kind {
        EnemyKind::Imp => Stats {
            behavior: Behavior::MeleeGround,
            health: tuning::IMP_HEALTH,
            speed: tuning::IMP_SPEED,
            width: tuning::IMP_WIDTH,
            height: tuning::IMP_HEIGHT,
            attack_range: tuning::IMP_ATTACK_RANGE,
            attack_damage: tuning::IMP_DAMAGE,
            attack_cooldown: tuning::IMP_ATTACK_COOLDOWN,
            windup_time: tuning::IMP_WINDUP,
            lunge_speed: tuning::IMP_LUNGE,
            lunge_reach: tuning::IMP_LUNGE_REACH,
            fire_cooldown: 0.0,
            preferred_range: 0.0,
            hover: 0.0,
            death_particles: 90,
            score: tuning::IMP_SCORE,
            key: "imp",
            color: vec3(1.0, 0.3, 0.2),
        },
        EnemyKind::Swarmer => Stats {
            behavior: Behavior::MeleeGround,
            health: tuning::SWARM_HEALTH,
            speed: tuning::SWARM_SPEED,
            width: tuning::SWARM_WIDTH,
            height: tuning::SWARM_HEIGHT,
            attack_range: tuning::SWARM_ATTACK_RANGE,
            attack_damage: tuning::SWARM_DAMAGE,
            attack_cooldown: tuning::SWARM_ATTACK_COOLDOWN,
            windup_time: tuning::SWARM_WINDUP,
            lunge_speed: tuning::SWARM_LUNGE,
            lunge_reach: tuning::SWARM_LUNGE_REACH,
            fire_cooldown: 0.0,
            preferred_range: 0.0,
            hover: 0.0,
            death_particles: 60,
            score: tuning::SWARM_SCORE,
            key: "swarm",
            color: vec3(0.4, 1.0, 0.4),
        },
        EnemyKind::Caster => Stats {
            behavior: Behavior::GroundRanged,
            health: tuning::CASTER_HEALTH,
            speed: tuning::CASTER_SPEED,
            width: tuning::CASTER_WIDTH,
            height: tuning::CASTER_HEIGHT,
            attack_range: 0.0,
            attack_damage: 0.0,
            attack_cooldown: 0.0,
            windup_time: tuning::CASTER_WINDUP,
            lunge_speed: 0.0,
            lunge_reach: 0.0,
            fire_cooldown: tuning::CASTER_FIRE_COOLDOWN,
            preferred_range: tuning::CASTER_PREFERRED_RANGE,
            hover: 0.0,
            death_particles: 110,
            score: tuning::CASTER_SCORE,
            key: "caster",
            color: vec3(0.75, 0.4, 1.0),
        },
        EnemyKind::Brute => Stats {
            behavior: Behavior::MeleeGround,
            health: tuning::BRUTE_HEALTH,
            speed: tuning::BRUTE_SPEED,
            width: tuning::BRUTE_WIDTH,
            height: tuning::BRUTE_HEIGHT,
            attack_range: tuning::BRUTE_ATTACK_RANGE,
            attack_damage: tuning::BRUTE_DAMAGE,
            attack_cooldown: tuning::BRUTE_ATTACK_COOLDOWN,
            windup_time: tuning::BRUTE_WINDUP,
            lunge_speed: tuning::BRUTE_LUNGE,
            lunge_reach: tuning::BRUTE_LUNGE_REACH,
            fire_cooldown: 0.0,
            preferred_range: 0.0,
            hover: 0.0,
            death_particles: 150,
            score: tuning::BRUTE_SCORE,
            key: "brute",
            color: vec3(1.0, 0.45, 0.15),
        },
        EnemyKind::Gargoyle => Stats {
            behavior: Behavior::FlyingDive,
            health: tuning::GARGOYLE_HEALTH,
            speed: tuning::GARGOYLE_SPEED,
            width: tuning::GARGOYLE_WIDTH,
            height: tuning::GARGOYLE_HEIGHT,
            attack_range: tuning::GARGOYLE_ATTACK_RANGE,
            attack_damage: tuning::GARGOYLE_DAMAGE,
            attack_cooldown: tuning::GARGOYLE_ATTACK_COOLDOWN,
            windup_time: tuning::GARGOYLE_WINDUP,
            lunge_speed: tuning::GARGOYLE_LUNGE,
            lunge_reach: tuning::GARGOYLE_LUNGE_REACH,
            fire_cooldown: 0.0,
            preferred_range: 0.0,
            hover: tuning::GARGOYLE_HOVER,
            death_particles: 100,
            score: tuning::GARGOYLE_SCORE,
            key: "gargoyle",
            color: vec3(0.55, 0.55, 1.0),
        },
        EnemyKind::Sentinel => Stats {
            behavior: Behavior::FlyingRanged,
            health: tuning::SENTINEL_HEALTH,
            speed: tuning::SENTINEL_SPEED,
            width: tuning::SENTINEL_WIDTH,
            height: tuning::SENTINEL_HEIGHT,
            attack_range: 0.0,
            attack_damage: 0.0,
            attack_cooldown: 0.0,
            windup_time: tuning::SENTINEL_WINDUP,
            lunge_speed: 0.0,
            lunge_reach: 0.0,
            fire_cooldown: tuning::SENTINEL_FIRE_COOLDOWN,
            preferred_range: tuning::SENTINEL_PREFERRED_RANGE,
            hover: tuning::SENTINEL_HOVER,
            death_particles: 90,
            score: tuning::SENTINEL_SCORE,
            key: "sentinel",
            color: vec3(0.3, 0.9, 1.0),
        },
    }
}

fn is_flying(behavior: Behavior) -> bool {
    matches!(behavior, Behavior::FlyingDive | Behavior::FlyingRanged)
}

/// Resolve the registered material name for an enemy's current visual state.
fn enemy_material(key: &str, elite: bool, hurt: bool, frame: usize) -> String {
    if hurt {
        format!("brimstone_mat_{key}_hurt")
    } else if elite {
        format!("brimstone_mat_{key}_f{frame}_e")
    } else {
        format!("brimstone_mat_{key}_f{frame}")
    }
}

fn body_scale(s: &Stats, elite: bool, boss: bool) -> Vec3 {
    let mut multiplier = 1.0;
    if elite {
        multiplier *= tuning::ELITE_SCALE;
    }
    if boss {
        multiplier *= tuning::BOSS_SCALE;
    }
    vec3(s.width * multiplier, s.height * multiplier, 1.0)
}

pub fn center(enemy: &Enemy) -> Vec3 {
    enemy.position + vec3(0.0, tuning::ENEMY_CENTER_HEIGHT, 0.0)
}

/// Hit volume for shots and splash: a sphere centred on the body mid-height and
/// sized to the body half-width. A scaled-up elite — or the warlord, at 1.7x a
/// brute — is then as hittable as it looks, instead of a fixed 0.75m bubble down
/// at its feet that most shots sail clean over.
pub fn hit_sphere(enemy: &Enemy) -> (Vec3, f32) {
    let stats = stats(enemy.kind);
    let scale = body_scale(&stats, enemy.elite, enemy.boss);
    let center = enemy.position + vec3(0.0, scale.y * 0.5, 0.0);
    let radius = (scale.x * 0.5).max(tuning::ENEMY_HIT_RADIUS);
    (center, radius)
}

pub fn spawn(
    brimstone_world: &mut BrimstoneWorld,
    world: &mut World,
    kind: EnemyKind,
    elite: bool,
    boss: bool,
    position: Vec3,
) {
    let s = stats(kind);
    let mut spawn_position = position;
    if is_flying(s.behavior) {
        spawn_position.y = s.hover;
    }
    let idle_material = enemy_material(s.key, elite, false, 0);
    let engine = billboard::spawn(
        world,
        &idle_material,
        spawn_position,
        body_scale(&s, elite, boss),
    );
    let strafe_roll = random_range(&mut brimstone_world.resources.game.random_state, 0.0, 1.0);
    let fire_jitter = random_range(&mut brimstone_world.resources.game.random_state, 0.4, 1.0)
        * tuning::CASTER_FIRE_COOLDOWN;
    let mut health = s.health;
    if elite {
        health *= tuning::ELITE_HEALTH_MULT;
    }
    if boss {
        health *= tuning::BOSS_HEALTH_MULT;
    }
    health *= brimstone_world.resources.settings.difficulty.enemy_health();
    let game_entity = brimstone_world.spawn_entities(ENEMY | ENGINE_ENTITY, 1)[0];
    brimstone_world.set_engine_entity(game_entity, EngineEntity(engine));
    if boss {
        brimstone_world.resources.game.boss_entity = Some(game_entity);
        brimstone_world.resources.game.boss_max_health = health;
        audio::play(brimstone_world, world, audio::BOSS, 0.9);
    }
    brimstone_world.set_enemy(
        game_entity,
        Enemy {
            kind,
            elite,
            boss,
            position: spawn_position,
            velocity: Vec3::zeros(),
            health,
            state: EnemyState::Chase,
            attack_cooldown: 0.0,
            fire_cooldown: fire_jitter,
            windup: 0.0,
            hit_flash: 0.0,
            death_timer: 0.0,
            anim_time: strafe_roll * 3.0,
            shown: 255,
            strafe_dir: if strafe_roll < 0.5 { -1.0 } else { 1.0 },
        },
    );
    fx::hit(
        brimstone_world,
        world,
        spawn_position + vec3(0.0, 1.0, 0.0),
        s.color,
    );
}

pub fn damage(
    brimstone_world: &mut BrimstoneWorld,
    world: &mut World,
    game_entity: Entity,
    amount: f32,
    point: Vec3,
    knockback: Vec3,
) {
    let Some(enemy) = brimstone_world.get_enemy(game_entity) else {
        return;
    };
    if enemy.state == EnemyState::Dying {
        return;
    }
    let kind = enemy.kind;
    let elite = enemy.elite;
    let boss = enemy.boss;
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
    brimstone_world.set_enemy(game_entity, updated);

    fx::hit(brimstone_world, world, point, s.color);
    if dead {
        let count = if boss { 320 } else { s.death_particles };
        fx::death(
            brimstone_world,
            world,
            position + vec3(0.0, tuning::ENEMY_CENTER_HEIGHT, 0.0),
            s.color,
            count,
        );
        let mut score = s.score;
        if elite {
            score *= tuning::ELITE_SCORE_MULT;
        }
        if boss {
            score *= tuning::BOSS_SCORE_MULT;
        }
        game::award(brimstone_world, score);
        if boss {
            brimstone_world.resources.game.boss_entity = None;
            brimstone_world.resources.game.shake += 1.2;
            brimstone_world.resources.game.hitstop = brimstone_world.resources.game.hitstop.max(0.12);
        } else if elite || matches!(kind, EnemyKind::Brute) {
            brimstone_world.resources.game.shake += 0.25;
            brimstone_world.resources.game.hitstop = brimstone_world.resources.game.hitstop.max(0.04);
        }
        audio::play(brimstone_world, world, audio::ENEMY_DEATH, 1.0);
        pickups::maybe_drop(brimstone_world, world, position);
    } else {
        audio::play(brimstone_world, world, audio::ENEMY_HURT, 0.4);
    }
}

pub fn update(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let player_center = player::position(brimstone_world, world);
    let player_ground = vec3(player_center.x, 0.0, player_center.z);
    let bound_x = (brimstone_world.resources.level.half_x - 1.5).max(2.0);
    let bound_z = (brimstone_world.resources.level.half_z - 1.5).max(2.0);

    let mut snapshots: Vec<(Entity, Entity, Enemy)> = brimstone_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine = brimstone_world.get_engine_entity(game_entity)?.0;
            let enemy = *brimstone_world.get_enemy(game_entity)?;
            Some((game_entity, engine, enemy))
        })
        .collect();

    let mut effects = Effects::default();
    let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

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
        let context = StepContext {
            player_center,
            direction,
            distance,
            delta,
            time,
        };

        match s.behavior {
            Behavior::FlyingRanged => {
                behave_flying_ranged(enemy, &s, &context, world, &mut effects)
            }
            Behavior::FlyingDive => behave_flying_dive(enemy, &s, &context, world, &mut effects),
            Behavior::MeleeGround => behave_melee(enemy, &s, &context, world, &mut effects),
            Behavior::GroundRanged => {
                behave_ground_ranged(enemy, &s, &context, world, &mut effects)
            }
        }

        enemy.position.x = enemy.position.x.clamp(-bound_x, bound_x);
        enemy.position.z = enemy.position.z.clamp(-bound_z, bound_z);
        if is_flying(s.behavior) {
            enemy.position.y = enemy
                .position
                .y
                .clamp(tuning::GARGOYLE_ALT_MIN, tuning::GARGOYLE_ALT_MAX);
        } else {
            enemy.position.y = 0.0;
        }
    }

    separate(&mut snapshots, bound_x, bound_z);

    if effects.melee_damage > 0.0 {
        game::damage_player(brimstone_world, world, effects.melee_damage);
    }
    for (origin, target, damage) in effects.fireballs {
        projectiles::spawn(brimstone_world, world, origin, target, damage);
        audio::play(brimstone_world, world, audio::FIREBALL, 0.32);
    }
    for (position, color) in effects.telegraphs {
        fx::hit(brimstone_world, world, position, color);
    }

    for (game_entity, engine, enemy) in &snapshots {
        let s = stats(enemy.kind);
        let base = body_scale(&s, enemy.elite, enemy.boss);
        if enemy.state == EnemyState::Dying {
            let fraction = (enemy.death_timer / tuning::ENEMY_DEATH_TIME).max(0.0);
            set_scale(world, *engine, vec3(base.x, base.y * fraction, 1.0));
        } else {
            let mut next = *enemy;
            let hurt = next.hit_flash > 0.0;
            let moving = matches!(next.state, EnemyState::Chase);
            let rate = if moving { 1.5 } else { 1.0 };
            next.anim_time += delta * tuning::ANIM_FPS * rate;
            let frame = (next.anim_time as usize) % art::ANIM_FRAMES;
            let code = if hurt {
                HURT_CODE
            } else {
                frame as u8 + if enemy.elite { 100 } else { 0 }
            };
            if code != next.shown {
                next.shown = code;
                let material = enemy_material(s.key, enemy.elite, hurt, frame);
                world
                    .core
                    .set_material_ref(*engine, MaterialRef::new(material));
            }
            let windup_fraction = if s.windup_time > 0.0 {
                (next.windup / s.windup_time).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let windup_scale = 1.0 + windup_fraction * 0.25;
            set_scale(
                world,
                *engine,
                vec3(base.x * windup_scale, base.y * windup_scale, 1.0),
            );
            if let Some(slot) = brimstone_world.get_enemy_mut(*game_entity) {
                *slot = next;
            }
            continue;
        }
        if let Some(slot) = brimstone_world.get_enemy_mut(*game_entity) {
            *slot = *enemy;
        }
    }

    remove_dead(brimstone_world, world);
}

/// Per-frame inputs shared by every behaviour: where the player is, the ground
/// direction and distance to them, the frame delta, and the world clock.
struct StepContext {
    player_center: Vec3,
    direction: Vec3,
    distance: f32,
    delta: f32,
    time: f32,
}

/// World-mutating consequences a behaviour produces, drained after the movement
/// loop so the borrow of `world` stays read-only inside the per-enemy step.
#[derive(Default)]
struct Effects {
    melee_damage: f32,
    fireballs: Vec<(Vec3, Vec3, f32)>,
    telegraphs: Vec<(Vec3, Vec3)>,
}

/// Melee/dive strike damage, scaled by elite and boss multipliers.
fn lunge_damage(enemy: &Enemy, s: &Stats) -> f32 {
    let mut multiplier = 1.0;
    if enemy.elite {
        multiplier *= tuning::ELITE_DAMAGE_MULT;
    }
    if enemy.boss {
        multiplier *= tuning::BOSS_DAMAGE_MULT;
    }
    s.attack_damage * multiplier
}

/// Fireball damage for ranged casters, scaled by the elite multiplier.
fn fireball_damage(enemy: &Enemy) -> f32 {
    tuning::FIREBALL_DAMAGE
        * if enemy.elite {
            tuning::ELITE_DAMAGE_MULT
        } else {
            1.0
        }
}

/// Hold the preferred stand-off range: advance when too far, retreat when too
/// close, strafe when in the band.
fn ranged_move_dir(enemy: &Enemy, s: &Stats, ctx: &StepContext) -> Vec3 {
    if ctx.distance > s.preferred_range + 1.5 {
        ctx.direction
    } else if ctx.distance < s.preferred_range - 1.5 {
        -ctx.direction
    } else {
        vec3(ctx.direction.z, 0.0, -ctx.direction.x) * enemy.strafe_dir
    }
}

/// Wind up, then loose a fireball; otherwise begin a new wind-up once cool.
fn ranged_fire(enemy: &mut Enemy, s: &Stats, ctx: &StepContext, effects: &mut Effects) {
    if enemy.windup > 0.0 {
        enemy.windup -= ctx.delta;
        if enemy.windup <= 0.0 {
            enemy.fire_cooldown = s.fire_cooldown;
            effects
                .fireballs
                .push((center(enemy), ctx.player_center, fireball_damage(enemy)));
        }
    } else if enemy.fire_cooldown <= 0.0 {
        enemy.windup = s.windup_time;
    }
}

/// Sentinel: bob at hover altitude, hold range, and lob fireballs.
fn behave_flying_ranged(
    enemy: &mut Enemy,
    s: &Stats,
    ctx: &StepContext,
    world: &World,
    effects: &mut Effects,
) {
    let target_alt = s.hover + (ctx.time * 1.8 + enemy.position.x).sin() * 0.4;
    enemy.position.y += (target_alt - enemy.position.y) * (3.0 * ctx.delta).min(1.0);
    enemy.state = EnemyState::Chase;
    let steer = avoid(world, enemy.position, ranged_move_dir(enemy, s, ctx));
    enemy.position += steer * s.speed * ctx.delta;
    ranged_fire(enemy, s, ctx, effects);
}

/// Caster: hold range on the ground and lob fireballs. Unlike the flying
/// sentinel, when closing the gap it routes toward the player via the navmesh so
/// it does not advance straight through a wall.
fn behave_ground_ranged(
    enemy: &mut Enemy,
    s: &Stats,
    ctx: &StepContext,
    world: &World,
    effects: &mut Effects,
) {
    enemy.state = EnemyState::Chase;
    let move_dir = ranged_move_dir(enemy, s, ctx);
    let move_dir = if ctx.distance > s.preferred_range + 1.5 {
        nav_direction(world, enemy.position, ctx.player_center).unwrap_or(move_dir)
    } else {
        move_dir
    };
    let steer = avoid(world, enemy.position, move_dir);
    enemy.position += steer * s.speed * ctx.delta;
    ranged_fire(enemy, s, ctx, effects);
}

/// Gargoyle: hover and close, telegraph, then dive-bomb the player in 3D.
fn behave_flying_dive(
    enemy: &mut Enemy,
    s: &Stats,
    ctx: &StepContext,
    world: &World,
    effects: &mut Effects,
) {
    let center_offset = s.height * 0.5;
    let to_player_3d = ctx.player_center - (enemy.position + vec3(0.0, center_offset, 0.0));
    let dist3 = to_player_3d.norm();
    let dir3 = if dist3 > 1e-3 {
        to_player_3d / dist3
    } else {
        vec3(0.0, 0.0, 1.0)
    };
    if enemy.windup > 0.0 {
        enemy.state = EnemyState::Attack;
        enemy.windup -= ctx.delta;
        if enemy.windup <= 0.0 {
            enemy.attack_cooldown = s.attack_cooldown;
            enemy.velocity += dir3 * s.lunge_speed;
            if dist3 <= s.attack_range + s.lunge_reach {
                effects.melee_damage += lunge_damage(enemy, s);
            }
        }
    } else if dist3 > s.attack_range {
        enemy.state = EnemyState::Chase;
        let steer = avoid(world, enemy.position, ctx.direction);
        enemy.position += steer * s.speed * ctx.delta;
        let target_alt = s.hover + (ctx.time * 2.2 + enemy.position.x).sin() * 0.45;
        enemy.position.y += (target_alt - enemy.position.y) * (3.0 * ctx.delta).min(1.0);
    } else if enemy.attack_cooldown <= 0.0 {
        enemy.state = EnemyState::Attack;
        enemy.windup = s.windup_time;
        effects
            .telegraphs
            .push((enemy.position + vec3(0.0, s.height * 0.6, 0.0), s.color));
    } else {
        enemy.state = EnemyState::Attack;
        enemy.position.y += (s.hover - enemy.position.y) * (2.0 * ctx.delta).min(1.0);
    }
}

/// Imp / swarmer / brute: close on the ground, telegraph, then lunge.
fn behave_melee(
    enemy: &mut Enemy,
    s: &Stats,
    ctx: &StepContext,
    world: &World,
    effects: &mut Effects,
) {
    let vertical = ctx.player_center.y - (enemy.position.y + tuning::ENEMY_CENTER_HEIGHT);
    let attack_distance = (ctx.distance * ctx.distance + vertical * vertical).sqrt();
    if enemy.windup > 0.0 {
        enemy.state = EnemyState::Attack;
        enemy.windup -= ctx.delta;
        if enemy.windup <= 0.0 {
            enemy.attack_cooldown = s.attack_cooldown;
            enemy.velocity += ctx.direction * s.lunge_speed;
            if attack_distance <= s.attack_range + s.lunge_reach {
                effects.melee_damage += lunge_damage(enemy, s);
            }
        }
    } else if attack_distance > s.attack_range {
        enemy.state = EnemyState::Chase;
        let desired =
            nav_direction(world, enemy.position, ctx.player_center).unwrap_or(ctx.direction);
        let steer = avoid(world, enemy.position, desired);
        enemy.position += steer * s.speed * ctx.delta;
    } else if enemy.attack_cooldown <= 0.0 {
        enemy.state = EnemyState::Attack;
        enemy.windup = s.windup_time;
        effects
            .telegraphs
            .push((enemy.position + vec3(0.0, s.height * 0.6, 0.0), s.color));
    } else {
        enemy.state = EnemyState::Attack;
    }
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

/// Wall-aware direction from `from` toward `to` via the level navmesh: steers at
/// the next path waypoint so ground enemies route around geometry instead of
/// clipping it. Returns `None` when there is no navmesh or no path, so the caller
/// falls back to straight-line steering (and the local [`avoid`] probes).
fn nav_direction(world: &World, from: Vec3, to: Vec3) -> Option<Vec3> {
    let navmesh = &world.resources.navmesh;
    if navmesh.triangles.is_empty() {
        return None;
    }
    let request = PathRequest::new(from, to).with_radius(tuning::NAV_AGENT_RADIUS);
    let result = find_path_with_algorithm(navmesh, &request, navmesh.algorithm);
    if !matches!(result.status, PathStatus::Found | PathStatus::PartialPath) {
        return None;
    }
    // Funnel the triangle corridor into a straight string-pulled path. Steering at
    // raw triangle centres makes enemies crab sideways and stall near corners; the
    // smoothed path heads straight at the corner to round, like the engine's own
    // agent mover does.
    let path = simplify_path(&smooth_path(navmesh, &result.triangle_path, from, to), 1.0);
    let next = path.iter().skip(1).find(|waypoint| {
        let mut offset = **waypoint - from;
        offset.y = 0.0;
        offset.norm() > tuning::NAV_WAYPOINT_MIN
    })?;
    let mut direction = next - from;
    direction.y = 0.0;
    if direction.norm() < 1e-3 {
        return None;
    }
    Some(direction.normalize())
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

fn separate(snapshots: &mut [(Entity, Entity, Enemy)], bound_x: f32, bound_z: f32) {
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
        enemy.position.x = enemy.position.x.clamp(-bound_x, bound_x);
        enemy.position.z = enemy.position.z.clamp(-bound_z, bound_z);
    }
}

fn remove_dead(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let dead: Vec<(Entity, Entity)> = brimstone_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let enemy = brimstone_world.get_enemy(game_entity)?;
            if enemy.state == EnemyState::Dying && enemy.death_timer <= 0.0 {
                let engine = brimstone_world.get_engine_entity(game_entity)?.0;
                Some((game_entity, engine))
            } else {
                None
            }
        })
        .collect();
    for (game_entity, engine) in dead {
        despawn_recursive_immediate(world, engine);
        brimstone_world.despawn_entities(&[game_entity]);
    }
}

pub fn despawn_all(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let engines: Vec<Entity> = brimstone_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            brimstone_world
                .get_engine_entity(game_entity)
                .map(|link| link.0)
        })
        .collect();
    for engine in engines {
        despawn_recursive_immediate(world, engine);
    }
    let game_entities: Vec<Entity> = brimstone_world.query_entities(ENEMY).collect();
    if !game_entities.is_empty() {
        brimstone_world.despawn_entities(&game_entities);
    }
}

pub fn total_count(brimstone_world: &BrimstoneWorld) -> usize {
    brimstone_world.query_entities(ENEMY).count()
}
