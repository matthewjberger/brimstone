use crate::campaign::{self, Objective};
use crate::content;
use crate::ecs::{BoomerWorld, EnemyKind, Phase, SpawnEntry};
use crate::systems::common::{combo_multiplier, next_random, random_range};
use crate::systems::world::{audio, enemies, level, pickups, player, projectiles};
use crate::tuning;
use nalgebra_glm::vec3;
use nightshade::prelude::*;

const POST_HIT_IFRAMES: f32 = 0.25;
const DEATH_SHAKE: f32 = 1.2;
const EXIT_RADIUS: f32 = 2.8;
const BANNER_TIME: f32 = 2.4;
const BEST_SCORE_PATH: &str = "boom_best.txt";

pub fn start_at(boomer_world: &mut BoomerWorld, world: &mut World, absolute_index: usize) {
    if !boomer_world.resources.game.seeded {
        let uptime = world.resources.window.timing.uptime_milliseconds;
        boomer_world.resources.game.random_state = 0x9e37_79b9_7f4a_7c15 ^ (uptime | 1);
        boomer_world.resources.game.best_score = load_best();
        boomer_world.resources.game.seeded = true;
    }
    reset_core(boomer_world);
    load_level(boomer_world, world, absolute_index);
}

pub fn teardown_world(boomer_world: &mut BoomerWorld, world: &mut World) {
    enemies::despawn_all(boomer_world, world);
    pickups::despawn_all(boomer_world, world);
    projectiles::despawn_all(boomer_world, world);
    level::despawn(boomer_world, world);
}

pub fn load_level(boomer_world: &mut BoomerWorld, world: &mut World, absolute_index: usize) {
    teardown_world(boomer_world, world);

    let count = content::count();
    boomer_world.resources.level.custom = false;
    boomer_world.resources.level.story = false;
    boomer_world.resources.level.objective = Objective::Exterminate;
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

    let cycle = boomer_world.resources.level.cycle;
    let scale = 1.0 + cycle as f32 * 0.5;
    let imps = (definition.roster.imps as f32 * scale).round() as u32;
    let swarmers = (definition.roster.swarmers as f32 * scale).round() as u32;
    let casters = (definition.roster.casters as f32 * scale).round() as u32;
    let brutes = (definition.roster.brutes as f32 * scale).round() as u32;
    let gargoyles = (definition.roster.gargoyles as f32 * scale).round() as u32;
    let mut waves = build_waves(
        boomer_world,
        imps,
        swarmers,
        casters,
        brutes,
        gargoyles,
        cycle,
    );

    let first = if waves.is_empty() {
        Vec::new()
    } else {
        waves.remove(0)
    };
    boomer_world.resources.game.waves = waves;
    boomer_world.resources.game.spawn_queue = first;
    boomer_world.resources.game.spawn_timer = 0.6;
    boomer_world.resources.level.wave = 1;
    boomer_world.resources.level.wave_count = tuning::WAVES_PER_LEVEL as u32;

    pickups::spawn_initial(boomer_world, world);
}

const DEFAULT_SPAWNS: &[(f32, f32)] = &[
    (16.0, 0.0),
    (-16.0, 0.0),
    (0.0, 16.0),
    (0.0, -16.0),
    (12.0, 12.0),
    (-12.0, -12.0),
];

/// Start a play session from the editor's authored level.
pub fn start_custom(boomer_world: &mut BoomerWorld, world: &mut World) {
    if !boomer_world.resources.game.seeded {
        let uptime = world.resources.window.timing.uptime_milliseconds;
        boomer_world.resources.game.random_state = 0x9e37_79b9_7f4a_7c15 ^ (uptime | 1);
        boomer_world.resources.game.best_score = load_best();
        boomer_world.resources.game.seeded = true;
    }
    reset_core(boomer_world);
    teardown_world(boomer_world, world);

    let data = boomer_world.resources.editor.data.clone();
    boomer_world.resources.level.custom = true;
    boomer_world.resources.level.story = false;
    boomer_world.resources.level.index = 0;
    boomer_world.resources.level.cycle = 0;
    boomer_world.resources.level.banner = BANNER_TIME;
    boomer_world.resources.level.custom_spawns = if data.spawn_points.is_empty() {
        DEFAULT_SPAWNS.to_vec()
    } else {
        data.spawn_points.clone()
    };

    level::build_dynamic(boomer_world, world, &data);
    let spawn = vec3(data.spawn[0], data.spawn[1], data.spawn[2]);
    player::teleport(boomer_world, world, spawn);

    let roster = data.roster;
    let mut waves = build_waves(
        boomer_world,
        roster.imps,
        roster.swarmers,
        roster.casters,
        roster.brutes,
        roster.gargoyles,
        0,
    );
    let first = if waves.is_empty() {
        Vec::new()
    } else {
        waves.remove(0)
    };
    boomer_world.resources.game.waves = waves;
    boomer_world.resources.game.spawn_queue = first;
    boomer_world.resources.game.spawn_timer = 0.6;
    boomer_world.resources.level.wave = 1;
    boomer_world.resources.level.wave_count = tuning::WAVES_PER_LEVEL as u32;

    pickups::spawn_initial(boomer_world, world);
}

/// Start a Story-mode mission: a static level framed by an objective.
pub fn start_mission(boomer_world: &mut BoomerWorld, world: &mut World, index: usize) {
    if !boomer_world.resources.game.seeded {
        let uptime = world.resources.window.timing.uptime_milliseconds;
        boomer_world.resources.game.random_state = 0x9e37_79b9_7f4a_7c15 ^ (uptime | 1);
        boomer_world.resources.game.best_score = load_best();
        boomer_world.resources.game.seeded = true;
    }
    reset_core(boomer_world);
    teardown_world(boomer_world, world);

    let mission = campaign::mission(index);
    let definition = content::level(mission.level);
    boomer_world.resources.level.custom = false;
    boomer_world.resources.level.story = true;
    boomer_world.resources.level.objective = mission.objective;
    boomer_world.resources.level.index = mission.level;
    boomer_world.resources.level.cycle = 0;
    boomer_world.resources.level.banner = BANNER_TIME;

    level::build(boomer_world, world, definition);
    let spawn = vec3(
        definition.spawn[0],
        definition.spawn[1],
        definition.spawn[2],
    );
    player::teleport(boomer_world, world, spawn);

    let roster = definition.roster;
    let mut waves = build_waves(
        boomer_world,
        roster.imps,
        roster.swarmers,
        roster.casters,
        roster.brutes,
        roster.gargoyles,
        0,
    );
    if mission.objective == Objective::Boss {
        if let Some(last) = waves.last_mut() {
            last.push((EnemyKind::Brute, true));
        } else {
            waves.push(vec![(EnemyKind::Brute, true)]);
        }
    }
    let first = if waves.is_empty() {
        Vec::new()
    } else {
        waves.remove(0)
    };
    boomer_world.resources.game.waves = waves;
    boomer_world.resources.game.spawn_queue = first;
    boomer_world.resources.game.spawn_timer = 0.6;
    boomer_world.resources.level.wave = 1;
    boomer_world.resources.level.wave_count = tuning::WAVES_PER_LEVEL as u32;

    pickups::spawn_initial(boomer_world, world);

    if mission.objective == Objective::Reach {
        level::open_exit(boomer_world, world);
    }
    if mission.objective == Objective::Keycard {
        let key = vec3(mission.key[0], mission.key[1], mission.key[2]);
        pickups::spawn_keycard(boomer_world, world, key);
    }
}

pub fn award(boomer_world: &mut BoomerWorld, base: u32) {
    let before = combo_multiplier(boomer_world.resources.game.combo);
    {
        let game = &mut boomer_world.resources.game;
        game.combo += 1;
        game.combo_timer = tuning::COMBO_WINDOW;
        game.since_kill = 0.0;
        game.pressure = (game.pressure - 1.0).max(0.0);
        let multiplier = combo_multiplier(game.combo);
        game.score += base * multiplier;
        game.score_flash = tuning::SCORE_FLASH_TIME;
    }
    if combo_multiplier(boomer_world.resources.game.combo) > before {
        grant_combo_reward(boomer_world);
    }
}

/// Stepping up the combo multiplier pays out overheal and ammo, so a hot
/// streak directly sustains the offense that earned it.
fn grant_combo_reward(boomer_world: &mut BoomerWorld) {
    let stats = &mut boomer_world.resources.stats;
    stats.health = (stats.health + tuning::COMBO_REWARD_HEAL).min(tuning::OVERHEAL_MAX);
    let weapon = &mut boomer_world.resources.weapon;
    weapon.shells = (weapon.shells + tuning::COMBO_REWARD_SHELLS).min(tuning::SHOTGUN_MAX);
    weapon.nails = (weapon.nails + tuning::COMBO_REWARD_NAILS).min(tuning::NAIL_MAX);
    weapon.rockets = (weapon.rockets + tuning::COMBO_REWARD_ROCKETS).min(tuning::ROCKET_MAX);
    boomer_world.resources.game.score_flash = tuning::SCORE_FLASH_TIME;
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
        let best = boomer_world
            .resources
            .game
            .best_score
            .max(boomer_world.resources.game.score);
        if best > boomer_world.resources.game.best_score {
            save_best(best);
        }
        boomer_world.resources.game.best_score = best;
        audio::play(boomer_world, world, audio::PLAYER_DEATH, 1.0);
    }
}

pub fn tick(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);

    {
        let game = &mut boomer_world.resources.game;
        game.score_flash = (game.score_flash - delta).max(0.0);
        if game.combo_timer > 0.0 {
            game.combo_timer -= delta;
            if game.combo_timer <= 0.0 {
                game.combo = 0;
            }
        }
    }
    boomer_world.resources.level.banner = (boomer_world.resources.level.banner - delta).max(0.0);

    decay_overheal(boomer_world, delta);
    apply_pressure(boomer_world, world, delta);

    if !boomer_world.resources.game.spawn_queue.is_empty() {
        boomer_world.resources.game.spawn_timer -= delta;
        if boomer_world.resources.game.spawn_timer <= 0.0 {
            if let Some((kind, elite)) = boomer_world.resources.game.spawn_queue.pop() {
                let position = spawn_point(boomer_world);
                enemies::spawn(boomer_world, world, kind, elite, position);
            }
            boomer_world.resources.game.spawn_timer = spawn_interval(boomer_world);
        }
    } else if enemies::total_count(boomer_world) == 0 {
        advance_wave(boomer_world, world);
    }

    check_keycard(boomer_world, world);

    if boomer_world.resources.level.exit_active {
        let player_position = player::position(boomer_world, world);
        let mut offset = player_position - boomer_world.resources.level.exit_position;
        offset.y = 0.0;
        if offset.norm() < EXIT_RADIUS {
            if boomer_world.resources.level.story {
                crate::systems::story::mission_complete(boomer_world, world);
            } else if boomer_world.resources.level.custom {
                crate::systems::editor::open(boomer_world, world);
            } else {
                let next = boomer_world.resources.level.cycle as usize * content::count()
                    + boomer_world.resources.level.index
                    + 1;
                load_level(boomer_world, world, next);
            }
        }
    }
}

fn advance_wave(boomer_world: &mut BoomerWorld, world: &mut World) {
    if !boomer_world.resources.game.waves.is_empty() {
        let wave = boomer_world.resources.game.waves.remove(0);
        boomer_world.resources.game.spawn_queue = wave;
        boomer_world.resources.game.spawn_timer = 0.6;
        boomer_world.resources.level.wave += 1;
        boomer_world.resources.level.banner = BANNER_TIME;
    } else if !boomer_world.resources.level.exit_active
        && !matches!(boomer_world.resources.level.objective, Objective::Keycard)
    {
        level::open_exit(boomer_world, world);
        boomer_world.resources.level.banner = BANNER_TIME;
    }
}

/// Unlock the gate the moment the keycard is recovered.
fn check_keycard(boomer_world: &mut BoomerWorld, world: &mut World) {
    if matches!(boomer_world.resources.level.objective, Objective::Keycard)
        && boomer_world.resources.game.has_key
        && !boomer_world.resources.level.exit_active
    {
        level::open_exit(boomer_world, world);
        boomer_world.resources.level.banner = BANNER_TIME;
        audio::play(boomer_world, world, audio::PICKUP, 0.9);
    }
}

fn decay_overheal(boomer_world: &mut BoomerWorld, delta: f32) {
    let stats = &mut boomer_world.resources.stats;
    if stats.health > stats.max_health {
        stats.health = (stats.health - tuning::OVERHEAL_DECAY * delta).max(stats.max_health);
    }
}

/// Camp with enemies alive and pressure builds until the horde reinforces,
/// nudging the player to keep pushing rather than turtle in a corner.
fn apply_pressure(boomer_world: &mut BoomerWorld, world: &mut World, delta: f32) {
    boomer_world.resources.game.since_kill += delta;
    let enemies_alive = enemies::total_count(boomer_world) > 0;
    let camping = enemies_alive && boomer_world.resources.game.since_kill > tuning::PRESSURE_GRACE;
    if !camping {
        return;
    }
    boomer_world.resources.game.pressure += tuning::PRESSURE_BUILD * delta;
    if boomer_world.resources.game.pressure >= tuning::PRESSURE_SPAWN_AT {
        boomer_world.resources.game.pressure = 0.0;
        boomer_world.resources.game.shake += 0.3;
        let position = spawn_point(boomer_world);
        enemies::spawn(boomer_world, world, EnemyKind::Swarmer, false, position);
    }
}

fn spawn_interval(boomer_world: &BoomerWorld) -> f32 {
    let cycle = boomer_world.resources.level.cycle as f32;
    let wave = boomer_world.resources.level.wave as f32;
    (tuning::SPAWN_INTERVAL - cycle * 0.05 - wave * 0.06).max(tuning::SPAWN_INTERVAL_MIN)
}

fn spawn_point(boomer_world: &mut BoomerWorld) -> nalgebra_glm::Vec3 {
    let custom = boomer_world.resources.level.custom;
    let len = if custom {
        boomer_world.resources.level.custom_spawns.len()
    } else {
        content::level(boomer_world.resources.level.index)
            .spawn_points
            .len()
    };
    if len == 0 {
        return vec3(0.0, 0.0, 16.0);
    }
    let pick = (random_range(
        &mut boomer_world.resources.game.random_state,
        0.0,
        len as f32,
    ) as usize)
        .min(len - 1);
    let (x, z) = if custom {
        boomer_world.resources.level.custom_spawns[pick]
    } else {
        content::level(boomer_world.resources.level.index).spawn_points[pick]
    };
    vec3(x, 0.0, z)
}

fn elite_fraction(cycle: u32) -> f32 {
    (cycle as f32 * 0.18).min(0.6)
}

/// Split the level roster into escalating waves. Fodder spreads across waves
/// for a swelling cadence; brutes anchor the final wave as a mini-boss cap.
fn build_waves(
    boomer_world: &mut BoomerWorld,
    imps: u32,
    swarmers: u32,
    casters: u32,
    brutes: u32,
    gargoyles: u32,
    cycle: u32,
) -> Vec<Vec<SpawnEntry>> {
    let fraction = elite_fraction(cycle);
    let count = tuning::WAVES_PER_LEVEL.max(1);
    let mut waves: Vec<Vec<SpawnEntry>> = (0..count).map(|_| Vec::new()).collect();
    let mut cursor = 0usize;

    for _ in 0..imps {
        let elite = next_random(&mut boomer_world.resources.game.random_state) < fraction;
        waves[cursor % count].push((EnemyKind::Imp, elite));
        cursor += 1;
    }
    for _ in 0..swarmers {
        waves[cursor % count].push((EnemyKind::Swarmer, false));
        cursor += 1;
    }
    for _ in 0..casters {
        let elite = next_random(&mut boomer_world.resources.game.random_state) < fraction;
        waves[cursor % count].push((EnemyKind::Caster, elite));
        cursor += 1;
    }
    for _ in 0..gargoyles {
        let elite = next_random(&mut boomer_world.resources.game.random_state) < fraction;
        waves[cursor % count].push((EnemyKind::Gargoyle, elite));
        cursor += 1;
    }
    let last = count - 1;
    for _ in 0..brutes {
        let elite = next_random(&mut boomer_world.resources.game.random_state) < fraction;
        waves[last].push((EnemyKind::Brute, elite));
    }
    waves
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
    boomer_world.resources.player.wall_run_side = 0;
    boomer_world.resources.player.wall_run_timer = 0.0;
    boomer_world.resources.player.wall_run_cooldown = 0.0;
    boomer_world.resources.player.wall_run_tilt = 0.0;
    boomer_world.resources.player.wall_run_normal = nalgebra_glm::Vec3::zeros();
}

fn load_best() -> u32 {
    std::fs::read_to_string(BEST_SCORE_PATH)
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

fn save_best(score: u32) {
    let _ = std::fs::write(BEST_SCORE_PATH, score.to_string());
}
