use crate::content;
use crate::ecs::{BoomerWorld, EnemyKind, Phase};
use crate::systems::common::{combo_multiplier, random_range};
use crate::systems::world::{audio, enemies, level, pickups, player, projectiles};
use crate::tuning;
use nalgebra_glm::vec3;
use nightshade::prelude::*;

const POST_HIT_IFRAMES: f32 = 0.25;
const DEATH_SHAKE: f32 = 1.2;
const EXIT_RADIUS: f32 = 2.8;
const BANNER_TIME: f32 = 2.4;

pub fn start_at(boomer_world: &mut BoomerWorld, world: &mut World, absolute_index: usize) {
    if !boomer_world.resources.game.seeded {
        let uptime = world.resources.window.timing.uptime_milliseconds;
        boomer_world.resources.game.random_state = 0x9e37_79b9_7f4a_7c15 ^ (uptime | 1);
        boomer_world.resources.game.seeded = true;
    }
    reset_core(boomer_world);
    load_level(boomer_world, world, absolute_index);
}

pub fn load_level(boomer_world: &mut BoomerWorld, world: &mut World, absolute_index: usize) {
    enemies::despawn_all(boomer_world, world);
    pickups::despawn_all(boomer_world, world);
    projectiles::despawn_all(boomer_world, world);
    level::despawn(boomer_world, world);

    let count = content::count();
    boomer_world.resources.level.index = absolute_index % count;
    boomer_world.resources.level.cycle = (absolute_index / count) as u32;
    boomer_world.resources.level.banner = BANNER_TIME;

    let definition = content::level(absolute_index);
    level::build(boomer_world, world, definition);

    let spawn = vec3(
        definition.spawn[0],
        definition.spawn[1],
        definition.spawn[2],
    );
    player::teleport(boomer_world, world, spawn);

    let scale = 1.0 + boomer_world.resources.level.cycle as f32 * 0.5;
    let imps = (definition.roster.imps as f32 * scale).round() as u32;
    let swarmers = (definition.roster.swarmers as f32 * scale).round() as u32;
    let casters = (definition.roster.casters as f32 * scale).round() as u32;
    boomer_world.resources.game.spawn_queue = build_queue(imps, swarmers, casters);
    boomer_world.resources.game.spawn_timer = 0.6;

    pickups::spawn_initial(boomer_world, world);
    audio::play(boomer_world, world, audio::WAVE, 0.7);
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
    boomer_world.resources.level.banner = (boomer_world.resources.level.banner - delta).max(0.0);

    if !boomer_world.resources.game.spawn_queue.is_empty() {
        boomer_world.resources.game.spawn_timer -= delta;
        if boomer_world.resources.game.spawn_timer <= 0.0 {
            if let Some(kind) = boomer_world.resources.game.spawn_queue.pop() {
                let position = spawn_point(boomer_world);
                enemies::spawn(boomer_world, world, kind, position);
            }
            let interval = (tuning::SPAWN_INTERVAL
                - boomer_world.resources.level.cycle as f32 * 0.05)
                .max(tuning::SPAWN_INTERVAL_MIN);
            boomer_world.resources.game.spawn_timer = interval;
        }
    } else if enemies::total_count(boomer_world) == 0 && !boomer_world.resources.level.exit_active {
        level::open_exit(boomer_world, world);
        boomer_world.resources.level.banner = BANNER_TIME;
        audio::play(boomer_world, world, audio::WAVE, 0.7);
    }

    if boomer_world.resources.level.exit_active {
        let player_position = player::position(boomer_world, world);
        let mut offset = player_position - boomer_world.resources.level.exit_position;
        offset.y = 0.0;
        if offset.norm() < EXIT_RADIUS {
            let next = boomer_world.resources.level.cycle as usize * content::count()
                + boomer_world.resources.level.index
                + 1;
            load_level(boomer_world, world, next);
        }
    }
}

fn spawn_point(boomer_world: &mut BoomerWorld) -> nalgebra_glm::Vec3 {
    let points = content::level(boomer_world.resources.level.index).spawn_points;
    let pick = random_range(
        &mut boomer_world.resources.game.random_state,
        0.0,
        points.len() as f32,
    ) as usize;
    let (x, z) = points[pick.min(points.len() - 1)];
    vec3(x, 0.0, z)
}

fn build_queue(imps: u32, swarmers: u32, casters: u32) -> Vec<EnemyKind> {
    let max = imps.max(swarmers).max(casters);
    let mut queue = Vec::new();
    for index in 0..max {
        if index < casters {
            queue.push(EnemyKind::Caster);
        }
        if index < swarmers {
            queue.push(EnemyKind::Swarmer);
        }
        if index < imps {
            queue.push(EnemyKind::Imp);
        }
    }
    queue
}

fn reset_core(boomer_world: &mut BoomerWorld) {
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
