use crate::ecs::{BoomerWorld, EnemyKind, Phase};
use crate::systems::common::{combo_multiplier, next_random, random_range};
use crate::systems::world::{audio, enemies, pickups, player, projectiles};
use crate::tuning;
use nalgebra_glm::vec3;
use nightshade::prelude::*;

const POST_HIT_IFRAMES: f32 = 0.6;
const DEATH_SHAKE: f32 = 1.2;

pub fn start(boomer_world: &mut BoomerWorld, world: &mut World) {
    if !boomer_world.resources.game.seeded {
        let uptime = world.resources.window.timing.uptime_milliseconds;
        boomer_world.resources.game.random_state = 0x9e37_79b9_7f4a_7c15 ^ (uptime | 1);
        boomer_world.resources.game.seeded = true;
    }
    reset_state(boomer_world);
    pickups::spawn_initial(boomer_world, world);
}

pub fn reset(boomer_world: &mut BoomerWorld, world: &mut World) {
    enemies::despawn_all(boomer_world, world);
    pickups::despawn_all(boomer_world, world);
    projectiles::despawn_all(boomer_world, world);
    reset_state(boomer_world);
    player::reset(boomer_world, world);
    pickups::spawn_initial(boomer_world, world);
}

pub fn award(boomer_world: &mut BoomerWorld, base: u32) {
    let game = &mut boomer_world.resources.game;
    game.combo += 1;
    game.combo_timer = tuning::COMBO_WINDOW;
    let multiplier = combo_multiplier(game.combo);
    game.score += base * multiplier;
    game.score_flash = tuning::SCORE_FLASH_TIME;
}

pub fn damage_player(boomer_world: &mut BoomerWorld, world: &mut World, amount: f32) {
    if !matches!(boomer_world.resources.game.phase, Phase::Playing) {
        return;
    }
    if boomer_world.resources.player.iframes > 0.0 {
        return;
    }
    boomer_world.resources.stats.health -= amount;
    boomer_world.resources.player.iframes = POST_HIT_IFRAMES;
    boomer_world.resources.game.damage_flash = tuning::DAMAGE_FLASH_TIME;
    boomer_world.resources.game.shake += tuning::PLAYER_HIT_SHAKE;
    boomer_world.resources.game.cam_kick += tuning::PLAYER_HIT_KICK;
    boomer_world.resources.game.fov_pop = boomer_world
        .resources
        .game
        .fov_pop
        .max(tuning::PLAYER_HIT_FOV_POP);
    audio::play(boomer_world, world, audio::PLAYER_HURT, 0.8);

    if boomer_world.resources.stats.health <= 0.0 {
        boomer_world.resources.stats.health = 0.0;
        boomer_world.resources.game.phase = Phase::Dead;
        boomer_world.resources.game.shake += DEATH_SHAKE;
        boomer_world.resources.game.best_score = boomer_world
            .resources
            .game
            .best_score
            .max(boomer_world.resources.game.score);
        audio::play(boomer_world, world, audio::PLAYER_DEATH, 1.0);
    }
}

pub fn tick(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);

    let game = &mut boomer_world.resources.game;
    game.score_flash = (game.score_flash - delta).max(0.0);
    if game.combo_timer > 0.0 {
        game.combo_timer -= delta;
        if game.combo_timer <= 0.0 {
            game.combo = 0;
        }
    }

    let queue_empty = boomer_world.resources.game.spawn_queue.is_empty();
    let arena_clear = enemies::total_count(boomer_world) == 0;

    if queue_empty && arena_clear {
        boomer_world.resources.game.wave_break -= delta;
        if boomer_world.resources.game.wave_break <= 0.0 {
            start_next_wave(boomer_world, world);
        }
    } else if !queue_empty {
        boomer_world.resources.game.spawn_timer -= delta;
        if boomer_world.resources.game.spawn_timer <= 0.0 {
            if let Some(kind) = boomer_world.resources.game.spawn_queue.pop() {
                let position = ring_position(boomer_world);
                enemies::spawn(boomer_world, world, kind, position);
            }
            let wave = boomer_world.resources.game.wave as f32;
            boomer_world.resources.game.spawn_timer =
                (tuning::SPAWN_INTERVAL - wave * 0.03).max(tuning::SPAWN_INTERVAL_MIN);
        }
    }
}

fn start_next_wave(boomer_world: &mut BoomerWorld, world: &mut World) {
    boomer_world.resources.game.wave += 1;
    let wave = boomer_world.resources.game.wave;
    let count = tuning::WAVE_BASE_COUNT + (wave - 1) * tuning::WAVE_COUNT_PER_WAVE;
    let mut queue = Vec::with_capacity(count as usize);
    for _ in 0..count {
        queue.push(roll_kind(boomer_world, wave));
    }
    boomer_world.resources.game.spawn_queue = queue;
    boomer_world.resources.game.spawn_timer = 0.0;
    boomer_world.resources.game.wave_break = tuning::WAVE_BREAK;
    audio::play(boomer_world, world, audio::WAVE, 0.8);
}

fn roll_kind(boomer_world: &mut BoomerWorld, wave: u32) -> EnemyKind {
    if wave <= 1 {
        return EnemyKind::Imp;
    }
    let roll = next_random(&mut boomer_world.resources.game.random_state);
    let caster_chance = ((wave as f32 - 1.0) * 0.06).min(0.28);
    let swarm_chance = (0.2 + wave as f32 * 0.035).min(0.48);
    if roll < caster_chance {
        EnemyKind::Caster
    } else if roll < caster_chance + swarm_chance {
        EnemyKind::Swarmer
    } else {
        EnemyKind::Imp
    }
}

fn ring_position(boomer_world: &mut BoomerWorld) -> nalgebra_glm::Vec3 {
    let angle = random_range(
        &mut boomer_world.resources.game.random_state,
        0.0,
        std::f32::consts::TAU,
    );
    let radius = tuning::SPAWN_RADIUS
        * random_range(&mut boomer_world.resources.game.random_state, 0.72, 1.0);
    vec3(angle.cos() * radius, 0.0, angle.sin() * radius)
}

fn reset_state(boomer_world: &mut BoomerWorld) {
    boomer_world.resources.stats = Default::default();
    boomer_world.resources.weapon = Default::default();
    let best = boomer_world.resources.game.best_score;
    let random_state = boomer_world.resources.game.random_state;
    boomer_world.resources.game = Default::default();
    boomer_world.resources.game.best_score = best;
    boomer_world.resources.game.random_state = random_state;
    boomer_world.resources.game.seeded = true;
    boomer_world.resources.player.dash_timer = 0.0;
    boomer_world.resources.player.dash_cooldown = 0.0;
    boomer_world.resources.player.iframes = 0.0;
}
