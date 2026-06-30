use crate::campaign::{self, Objective};
use crate::content;
use crate::ecs::{CobaltWorld, EnemyKind, Phase, SpawnEntry, WeaponKind};
use crate::systems::common::{combo_multiplier, next_random, random_range};
use crate::systems::world::{audio, enemies, level, pickups, player, projectiles};
use crate::tuning;
use nalgebra_glm::vec3;
use nightshade::prelude::*;

const POST_HIT_IFRAMES: f32 = 0.25;
const DEATH_SHAKE: f32 = 1.2;
const EXIT_RADIUS: f32 = 2.8;
const BANNER_TIME: f32 = 2.4;
const BEST_SCORE_PATH: &str = "cobalt_best.txt";

pub fn start_at(cobalt_world: &mut CobaltWorld, world: &mut World, absolute_index: usize) {
    ensure_seeded(cobalt_world, world);
    reset_core(cobalt_world);
    load_level(cobalt_world, world, absolute_index);
}

pub fn teardown_world(cobalt_world: &mut CobaltWorld, world: &mut World) {
    enemies::despawn_all(cobalt_world, world);
    pickups::despawn_all(cobalt_world, world);
    projectiles::despawn_all(cobalt_world, world);
    level::despawn(cobalt_world, world);
}

pub fn load_level(cobalt_world: &mut CobaltWorld, world: &mut World, absolute_index: usize) {
    teardown_world(cobalt_world, world);

    let count = content::count();
    cobalt_world.resources.level.custom = false;
    cobalt_world.resources.level.story = false;
    cobalt_world.resources.level.objective = Objective::Exterminate;
    cobalt_world.resources.level.index = absolute_index % count;
    cobalt_world.resources.level.cycle = (absolute_index / count) as u32;
    cobalt_world.resources.level.banner = BANNER_TIME;

    let definition = content::level(absolute_index);
    level::build(cobalt_world, world, definition);

    let spawn = vec3(
        definition.spawn[0],
        definition.spawn[1],
        definition.spawn[2],
    );
    player::teleport(cobalt_world, world, spawn);

    let cycle = cobalt_world.resources.level.cycle;
    let scale = 1.0 + cycle as f32 * 0.5;
    let roster = scale_roster(definition.roster, scale);
    let waves = build_waves(cobalt_world, roster, cycle);
    arm_waves(cobalt_world, waves);

    pickups::spawn_initial(cobalt_world, world);
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
pub fn start_custom(cobalt_world: &mut CobaltWorld, world: &mut World) {
    ensure_seeded(cobalt_world, world);
    reset_core(cobalt_world);
    teardown_world(cobalt_world, world);

    let data = cobalt_world.resources.editor.data.clone();
    cobalt_world.resources.level.custom = true;
    cobalt_world.resources.level.story = false;
    cobalt_world.resources.level.index = 0;
    cobalt_world.resources.level.cycle = 0;
    cobalt_world.resources.level.banner = BANNER_TIME;
    cobalt_world.resources.level.custom_spawns = if data.spawn_points.is_empty() {
        DEFAULT_SPAWNS.to_vec()
    } else {
        data.spawn_points.clone()
    };

    level::build_dynamic(cobalt_world, world, &data);
    let spawn = vec3(data.spawn[0], data.spawn[1], data.spawn[2]);
    player::teleport(cobalt_world, world, spawn);

    let waves = build_waves(cobalt_world, data.roster, 0);
    arm_waves(cobalt_world, waves);

    pickups::spawn_initial(cobalt_world, world);
}

/// Start a Story-mode mission: a static level framed by an objective.
pub fn start_mission(cobalt_world: &mut CobaltWorld, world: &mut World, index: usize) {
    ensure_seeded(cobalt_world, world);
    reset_core(cobalt_world);
    teardown_world(cobalt_world, world);

    let mission = campaign::mission(index);
    let definition = content::level(mission.level);
    cobalt_world.resources.level.custom = false;
    cobalt_world.resources.level.story = true;
    cobalt_world.resources.level.objective = mission.objective;
    cobalt_world.resources.level.index = mission.level;
    cobalt_world.resources.level.cycle = 0;
    cobalt_world.resources.level.banner = BANNER_TIME;

    level::build(cobalt_world, world, definition);
    let spawn = vec3(
        definition.spawn[0],
        definition.spawn[1],
        definition.spawn[2],
    );
    player::teleport(cobalt_world, world, spawn);

    let mut waves = build_waves(cobalt_world, definition.roster, 0);
    if mission.objective == Objective::Boss {
        if let Some(last) = waves.last_mut() {
            last.push((EnemyKind::Brute, true, true));
        } else {
            waves.push(vec![(EnemyKind::Brute, true, true)]);
        }
    }
    arm_waves(cobalt_world, waves);

    pickups::spawn_initial(cobalt_world, world);

    if mission.objective == Objective::Reach {
        level::open_exit(cobalt_world, world);
    }
    if mission.objective == Objective::Keycard {
        let key = vec3(mission.key[0], mission.key[1], mission.key[2]);
        pickups::spawn_keycard(cobalt_world, world, key);
    }
}

/// Restart whatever the player is currently in: the same story mission, the
/// same custom level, or arcade from the start.
pub fn restart_current(cobalt_world: &mut CobaltWorld, world: &mut World) {
    if cobalt_world.resources.level.story {
        let mission = cobalt_world.resources.story.mission;
        start_mission(cobalt_world, world, mission);
    } else if cobalt_world.resources.level.custom {
        start_custom(cobalt_world, world);
    } else {
        start_at(cobalt_world, world, 0);
    }
}

pub fn award(cobalt_world: &mut CobaltWorld, base: u32) {
    let before = combo_multiplier(cobalt_world.resources.game.combo);
    {
        let game = &mut cobalt_world.resources.game;
        game.combo += 1;
        game.combo_timer = tuning::COMBO_WINDOW;
        game.since_kill = 0.0;
        game.pressure = (game.pressure - 1.0).max(0.0);
        let multiplier = combo_multiplier(game.combo);
        game.score += base * multiplier;
        game.score_flash = tuning::SCORE_FLASH_TIME;
    }
    if combo_multiplier(cobalt_world.resources.game.combo) > before {
        grant_combo_reward(cobalt_world);
    }
}

/// Stepping up the combo multiplier pays out overheal and ammo, so a hot
/// streak directly sustains the offense that earned it.
fn grant_combo_reward(cobalt_world: &mut CobaltWorld) {
    let stats = &mut cobalt_world.resources.stats;
    stats.health = (stats.health + tuning::COMBO_REWARD_HEAL).min(tuning::OVERHEAL_MAX);
    let weapon = &mut cobalt_world.resources.weapon;
    weapon.add_ammo(WeaponKind::Shotgun, tuning::COMBO_REWARD_SHELLS);
    weapon.add_ammo(WeaponKind::Nailgun, tuning::COMBO_REWARD_NAILS);
    weapon.add_ammo(WeaponKind::Rocket, tuning::COMBO_REWARD_ROCKETS);
    cobalt_world.resources.game.score_flash = tuning::SCORE_FLASH_TIME;
}

pub fn damage_player(cobalt_world: &mut CobaltWorld, world: &mut World, amount: f32) {
    if !matches!(cobalt_world.resources.game.phase, Phase::Playing) {
        return;
    }
    if cobalt_world.resources.player.iframes > 0.0 {
        return;
    }
    let amount = amount * cobalt_world.resources.settings.difficulty.damage_taken();
    cobalt_world.resources.stats.health -= amount;
    cobalt_world.resources.player.iframes = POST_HIT_IFRAMES;
    cobalt_world.resources.game.damage_flash = tuning::DAMAGE_FLASH_TIME;
    cobalt_world.resources.game.shake += tuning::PLAYER_HIT_SHAKE;
    cobalt_world.resources.game.cam_kick += tuning::PLAYER_HIT_KICK;
    cobalt_world.resources.game.fov_pop = cobalt_world
        .resources
        .game
        .fov_pop
        .max(tuning::PLAYER_HIT_FOV_POP);
    audio::play(cobalt_world, world, audio::PLAYER_HURT, 0.8);

    if cobalt_world.resources.stats.health <= 0.0 {
        cobalt_world.resources.stats.health = 0.0;
        cobalt_world.resources.game.phase = Phase::Dead;
        cobalt_world.resources.game.shake += DEATH_SHAKE;
        let best = cobalt_world
            .resources
            .game
            .best_score
            .max(cobalt_world.resources.game.score);
        if best > cobalt_world.resources.game.best_score {
            save_best(best);
        }
        cobalt_world.resources.game.best_score = best;
        audio::play(cobalt_world, world, audio::PLAYER_DEATH, 1.0);
    }
}

pub fn tick(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);

    {
        let game = &mut cobalt_world.resources.game;
        game.score_flash = (game.score_flash - delta).max(0.0);
        if game.combo_timer > 0.0 {
            game.combo_timer -= delta;
            if game.combo_timer <= 0.0 {
                game.combo = 0;
            }
        }
    }
    cobalt_world.resources.level.banner = (cobalt_world.resources.level.banner - delta).max(0.0);

    decay_overheal(cobalt_world, delta);
    apply_pressure(cobalt_world, world, delta);

    if !cobalt_world.resources.game.spawn_queue.is_empty() {
        cobalt_world.resources.game.spawn_timer -= delta;
        if cobalt_world.resources.game.spawn_timer <= 0.0 {
            if let Some((kind, elite, boss)) = cobalt_world.resources.game.spawn_queue.pop() {
                let position = spawn_point(cobalt_world);
                enemies::spawn(cobalt_world, world, kind, elite, boss, position);
            }
            cobalt_world.resources.game.spawn_timer = spawn_interval(cobalt_world);
        }
    } else if enemies::total_count(cobalt_world) == 0 {
        advance_wave(cobalt_world, world);
    }

    check_keycard(cobalt_world, world);

    if cobalt_world.resources.level.exit_active {
        let player_position = player::position(cobalt_world, world);
        let mut offset = player_position - cobalt_world.resources.level.exit_position;
        offset.y = 0.0;
        if offset.norm() < EXIT_RADIUS {
            if cobalt_world.resources.level.story {
                crate::systems::story::mission_complete(cobalt_world, world);
            } else if cobalt_world.resources.level.custom {
                crate::systems::editor::open(cobalt_world, world);
            } else {
                let next = cobalt_world.resources.level.cycle as usize * content::count()
                    + cobalt_world.resources.level.index
                    + 1;
                load_level(cobalt_world, world, next);
            }
        }
    }
}

fn advance_wave(cobalt_world: &mut CobaltWorld, world: &mut World) {
    if !cobalt_world.resources.game.waves.is_empty() {
        let wave = cobalt_world.resources.game.waves.remove(0);
        cobalt_world.resources.game.spawn_queue = wave;
        cobalt_world.resources.game.spawn_timer = 0.6;
        cobalt_world.resources.level.wave += 1;
        cobalt_world.resources.level.banner = BANNER_TIME;
    } else if !cobalt_world.resources.level.exit_active
        && !matches!(cobalt_world.resources.level.objective, Objective::Keycard)
    {
        level::open_exit(cobalt_world, world);
        cobalt_world.resources.level.banner = BANNER_TIME;
        audio::play(cobalt_world, world, audio::CLEAR, 0.7);
    }
}

/// Unlock the gate the moment the keycard is recovered.
fn check_keycard(cobalt_world: &mut CobaltWorld, world: &mut World) {
    if matches!(cobalt_world.resources.level.objective, Objective::Keycard)
        && cobalt_world.resources.game.has_key
        && !cobalt_world.resources.level.exit_active
    {
        level::open_exit(cobalt_world, world);
        cobalt_world.resources.level.banner = BANNER_TIME;
        audio::play(cobalt_world, world, audio::CLEAR, 0.8);
    }
}

fn decay_overheal(cobalt_world: &mut CobaltWorld, delta: f32) {
    let stats = &mut cobalt_world.resources.stats;
    if stats.health > stats.max_health {
        stats.health = (stats.health - tuning::OVERHEAL_DECAY * delta).max(stats.max_health);
    }
}

/// Camp with enemies alive and pressure builds until the horde reinforces,
/// nudging the player to keep pushing rather than turtle in a corner.
fn apply_pressure(cobalt_world: &mut CobaltWorld, world: &mut World, delta: f32) {
    cobalt_world.resources.game.since_kill += delta;
    let enemies_alive = enemies::total_count(cobalt_world) > 0;
    let camping = enemies_alive && cobalt_world.resources.game.since_kill > tuning::PRESSURE_GRACE;
    if !camping {
        return;
    }
    cobalt_world.resources.game.pressure += tuning::PRESSURE_BUILD * delta;
    if cobalt_world.resources.game.pressure >= tuning::PRESSURE_SPAWN_AT {
        cobalt_world.resources.game.pressure = 0.0;
        cobalt_world.resources.game.shake += 0.3;
        let position = spawn_point(cobalt_world);
        enemies::spawn(
            cobalt_world,
            world,
            EnemyKind::Swarmer,
            false,
            false,
            position,
        );
    }
}

fn spawn_interval(cobalt_world: &CobaltWorld) -> f32 {
    let cycle = cobalt_world.resources.level.cycle as f32;
    let wave = cobalt_world.resources.level.wave as f32;
    (tuning::SPAWN_INTERVAL - cycle * 0.05 - wave * 0.06).max(tuning::SPAWN_INTERVAL_MIN)
}

fn spawn_point(cobalt_world: &mut CobaltWorld) -> nalgebra_glm::Vec3 {
    let custom = cobalt_world.resources.level.custom;
    let len = if custom {
        cobalt_world.resources.level.custom_spawns.len()
    } else {
        content::level(cobalt_world.resources.level.index)
            .spawn_points
            .len()
    };
    if len == 0 {
        return vec3(0.0, 0.0, 16.0);
    }
    let pick = (random_range(
        &mut cobalt_world.resources.game.random_state,
        0.0,
        len as f32,
    ) as usize)
        .min(len - 1);
    let (x, z) = if custom {
        cobalt_world.resources.level.custom_spawns[pick]
    } else {
        content::level(cobalt_world.resources.level.index).spawn_points[pick]
    };
    vec3(x, 0.0, z)
}

fn elite_fraction(cycle: u32) -> f32 {
    (cycle as f32 * 0.18).min(0.6)
}

/// Split the level roster into escalating waves. Fodder spreads across waves
/// for a swelling cadence; brutes anchor the final wave as a mini-boss cap.
/// Scale a level's base roster up by the endless-mode cycle multiplier.
fn scale_roster(roster: content::Roster, scale: f32) -> content::Roster {
    let bump = |count: u32| (count as f32 * scale).round() as u32;
    content::Roster {
        imps: bump(roster.imps),
        swarmers: bump(roster.swarmers),
        casters: bump(roster.casters),
        brutes: bump(roster.brutes),
        gargoyles: bump(roster.gargoyles),
        sentinels: bump(roster.sentinels),
    }
}

fn build_waves(
    cobalt_world: &mut CobaltWorld,
    roster: content::Roster,
    cycle: u32,
) -> Vec<Vec<SpawnEntry>> {
    let fraction = elite_fraction(cycle);
    let count = tuning::WAVES_PER_LEVEL.max(1);
    let mut waves: Vec<Vec<SpawnEntry>> = (0..count).map(|_| Vec::new()).collect();
    let mut cursor = 0usize;
    let spread: [(EnemyKind, u32, bool); 5] = [
        (EnemyKind::Imp, roster.imps, true),
        (EnemyKind::Swarmer, roster.swarmers, false),
        (EnemyKind::Caster, roster.casters, true),
        (EnemyKind::Gargoyle, roster.gargoyles, true),
        (EnemyKind::Sentinel, roster.sentinels, true),
    ];
    for (kind, amount, can_elite) in spread {
        for _ in 0..amount {
            let elite =
                can_elite && next_random(&mut cobalt_world.resources.game.random_state) < fraction;
            waves[cursor % count].push((kind, elite, false));
            cursor += 1;
        }
    }

    let last = count - 1;
    for _ in 0..roster.brutes {
        let elite = next_random(&mut cobalt_world.resources.game.random_state) < fraction;
        waves[last].push((EnemyKind::Brute, elite, false));
    }
    waves
}

/// Seed the run RNG and load the persisted best score the first time any session
/// starts this process. Idempotent: later sessions keep the live seed and best.
fn ensure_seeded(cobalt_world: &mut CobaltWorld, world: &World) {
    if cobalt_world.resources.game.seeded {
        return;
    }
    let uptime = world.resources.window.timing.uptime_milliseconds;
    cobalt_world.resources.game.random_state = 0x9e37_79b9_7f4a_7c15 ^ (uptime | 1);
    cobalt_world.resources.game.best_score = load_best();
    cobalt_world.resources.game.seeded = true;
}

/// Load `waves` into the spawn schedule: the first wave becomes the live spawn
/// queue, the rest are held back, and the wave counters reset to a level's start.
fn arm_waves(cobalt_world: &mut CobaltWorld, mut waves: Vec<Vec<SpawnEntry>>) {
    let first = if waves.is_empty() {
        Vec::new()
    } else {
        waves.remove(0)
    };
    cobalt_world.resources.game.waves = waves;
    cobalt_world.resources.game.spawn_queue = first;
    cobalt_world.resources.game.spawn_timer = 0.6;
    cobalt_world.resources.level.wave = 1;
    cobalt_world.resources.level.wave_count = tuning::WAVES_PER_LEVEL as u32;
}

fn reset_core(cobalt_world: &mut CobaltWorld) {
    cobalt_world.resources.stats = Default::default();
    cobalt_world.resources.weapon = Default::default();
    let best = cobalt_world.resources.game.best_score;
    let random_state = cobalt_world.resources.game.random_state;
    cobalt_world.resources.game = Default::default();
    cobalt_world.resources.game.best_score = best;
    cobalt_world.resources.game.random_state = random_state;
    cobalt_world.resources.game.seeded = true;
    cobalt_world.resources.player.dash_timer = 0.0;
    cobalt_world.resources.player.dash_cooldown = 0.0;
    cobalt_world.resources.player.iframes = 0.0;
    cobalt_world.resources.player.spawn_grace = 3;
    cobalt_world.resources.player.wall_run_side = 0;
    cobalt_world.resources.player.wall_run_timer = 0.0;
    cobalt_world.resources.player.wall_run_cooldown = 0.0;
    cobalt_world.resources.player.wall_run_tilt = 0.0;
    cobalt_world.resources.player.wall_run_normal = nalgebra_glm::Vec3::zeros();
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
