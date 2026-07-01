use crate::campaign::{self, Objective};
use crate::content;
use crate::ecs::{BrimstoneWorld, EnemyKind, Phase, SpawnEntry, WeaponKind};
use crate::systems::common::{combo_multiplier, next_random, random_range};
use crate::systems::world::{audio, enemies, level, pickups, player, projectiles};
use crate::tuning;
use nalgebra_glm::vec3;
use nightshade::prelude::*;

const POST_HIT_IFRAMES: f32 = 0.25;
const DEATH_SHAKE: f32 = 1.2;
const EXIT_RADIUS: f32 = 2.8;
const BANNER_TIME: f32 = 2.4;
const BEST_SCORE_PATH: &str = "brimstone_best.txt";

pub fn start_at(brimstone_world: &mut BrimstoneWorld, world: &mut World, absolute_index: usize) {
    ensure_seeded(brimstone_world, world);
    reset_core(brimstone_world);
    load_level(brimstone_world, world, absolute_index);
}

pub fn teardown_world(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    enemies::despawn_all(brimstone_world, world);
    pickups::despawn_all(brimstone_world, world);
    projectiles::despawn_all(brimstone_world, world);
    level::despawn(brimstone_world, world);
}

pub fn load_level(brimstone_world: &mut BrimstoneWorld, world: &mut World, absolute_index: usize) {
    teardown_world(brimstone_world, world);

    let count = content::count();
    brimstone_world.resources.level.custom = false;
    brimstone_world.resources.level.story = false;
    brimstone_world.resources.level.objective = Objective::Exterminate;
    brimstone_world.resources.level.index = absolute_index % count;
    brimstone_world.resources.level.cycle = (absolute_index / count) as u32;
    brimstone_world.resources.level.banner = BANNER_TIME;

    let definition = content::level(absolute_index);
    level::build(brimstone_world, world, definition);

    let spawn = vec3(
        definition.spawn[0],
        definition.spawn[1],
        definition.spawn[2],
    );
    player::teleport(brimstone_world, world, spawn);

    let cycle = brimstone_world.resources.level.cycle;
    let scale = 1.0 + cycle as f32 * 0.5;
    let roster = scale_roster(definition.roster, scale);
    let waves = build_waves(brimstone_world, roster, cycle);
    arm_waves(brimstone_world, waves);

    // Levels with a power core are lock-and-key: the exit stays sealed until the
    // core is seized from its spoke, so you explore rather than camp the entrance.
    if let Some(key) = definition.key {
        brimstone_world.resources.level.objective = Objective::Keycard;
        brimstone_world.resources.game.has_key = false;
        pickups::spawn_keycard(brimstone_world, world, vec3(key[0], key[1], key[2]));
    }

    pickups::spawn_initial(brimstone_world, world);
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
pub fn start_custom(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    ensure_seeded(brimstone_world, world);
    reset_core(brimstone_world);
    teardown_world(brimstone_world, world);

    let data = brimstone_world.resources.editor.data.clone();
    brimstone_world.resources.level.custom = true;
    brimstone_world.resources.level.story = false;
    brimstone_world.resources.level.index = 0;
    brimstone_world.resources.level.cycle = 0;
    brimstone_world.resources.level.banner = BANNER_TIME;
    brimstone_world.resources.level.custom_spawns = if data.spawn_points.is_empty() {
        DEFAULT_SPAWNS.to_vec()
    } else {
        data.spawn_points.clone()
    };

    level::build_dynamic(brimstone_world, world, &data);
    let spawn = vec3(data.spawn[0], data.spawn[1], data.spawn[2]);
    player::teleport(brimstone_world, world, spawn);

    let waves = build_waves(brimstone_world, data.roster, 0);
    arm_waves(brimstone_world, waves);

    pickups::spawn_initial(brimstone_world, world);
}

/// Start a Story-mode mission: a static level framed by an objective.
pub fn start_mission(brimstone_world: &mut BrimstoneWorld, world: &mut World, index: usize) {
    ensure_seeded(brimstone_world, world);
    reset_core(brimstone_world);
    teardown_world(brimstone_world, world);

    let mission = campaign::mission(index);
    let definition = content::level(mission.level);
    brimstone_world.resources.level.custom = false;
    brimstone_world.resources.level.story = true;
    brimstone_world.resources.level.objective = mission.objective;
    brimstone_world.resources.level.index = mission.level;
    brimstone_world.resources.level.cycle = 0;
    brimstone_world.resources.level.banner = BANNER_TIME;

    level::build(brimstone_world, world, definition);
    let spawn = vec3(
        definition.spawn[0],
        definition.spawn[1],
        definition.spawn[2],
    );
    player::teleport(brimstone_world, world, spawn);

    let mut waves = build_waves(brimstone_world, definition.roster, 0);
    if mission.objective == Objective::Boss {
        if let Some(last) = waves.last_mut() {
            last.push((EnemyKind::Brute, true, true));
        } else {
            waves.push(vec![(EnemyKind::Brute, true, true)]);
        }
    }
    arm_waves(brimstone_world, waves);

    pickups::spawn_initial(brimstone_world, world);

    if mission.objective == Objective::Reach {
        level::open_exit(brimstone_world, world);
    }
    if mission.objective == Objective::Keycard {
        let key = vec3(mission.key[0], mission.key[1], mission.key[2]);
        pickups::spawn_keycard(brimstone_world, world, key);
    }
}

/// Restart whatever the player is currently in: the same story mission, the
/// same custom level, or arcade from the start.
pub fn restart_current(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if brimstone_world.resources.level.story {
        let mission = brimstone_world.resources.story.mission;
        start_mission(brimstone_world, world, mission);
    } else if brimstone_world.resources.level.custom {
        start_custom(brimstone_world, world);
    } else {
        start_at(brimstone_world, world, 0);
    }
}

pub fn award(brimstone_world: &mut BrimstoneWorld, base: u32) {
    let before = combo_multiplier(brimstone_world.resources.game.combo);
    {
        let game = &mut brimstone_world.resources.game;
        game.combo += 1;
        game.kills += 1;
        game.combo_timer = tuning::COMBO_WINDOW;
        game.since_kill = 0.0;
        game.pressure = (game.pressure - 1.0).max(0.0);
        let multiplier = combo_multiplier(game.combo);
        game.score += base * multiplier;
        game.score_flash = tuning::SCORE_FLASH_TIME;
    }
    if combo_multiplier(brimstone_world.resources.game.combo) > before {
        grant_combo_reward(brimstone_world);
    }
}

/// Stepping up the combo multiplier pays out overheal and ammo, so a hot
/// streak directly sustains the offense that earned it.
fn grant_combo_reward(brimstone_world: &mut BrimstoneWorld) {
    let stats = &mut brimstone_world.resources.stats;
    stats.health = (stats.health + tuning::COMBO_REWARD_HEAL).min(tuning::OVERHEAL_MAX);
    let weapon = &mut brimstone_world.resources.weapon;
    weapon.add_ammo(WeaponKind::Shotgun, tuning::COMBO_REWARD_SHELLS);
    weapon.add_ammo(WeaponKind::Nailgun, tuning::COMBO_REWARD_NAILS);
    weapon.add_ammo(WeaponKind::Rocket, tuning::COMBO_REWARD_ROCKETS);
    brimstone_world.resources.game.score_flash = tuning::SCORE_FLASH_TIME;
}

pub fn damage_player(brimstone_world: &mut BrimstoneWorld, world: &mut World, amount: f32) {
    if !matches!(brimstone_world.resources.game.phase, Phase::Playing) {
        return;
    }
    if brimstone_world.resources.player.iframes > 0.0 {
        return;
    }
    let amount = amount * brimstone_world.resources.settings.difficulty.damage_taken();
    brimstone_world.resources.stats.health -= amount;
    brimstone_world.resources.player.iframes = POST_HIT_IFRAMES;
    brimstone_world.resources.game.damage_flash = tuning::DAMAGE_FLASH_TIME;
    brimstone_world.resources.game.shake += tuning::PLAYER_HIT_SHAKE;
    brimstone_world.resources.game.cam_kick += tuning::PLAYER_HIT_KICK;
    brimstone_world.resources.game.fov_pop = brimstone_world
        .resources
        .game
        .fov_pop
        .max(tuning::PLAYER_HIT_FOV_POP);
    audio::play(brimstone_world, world, audio::PLAYER_HURT, 0.8);

    if brimstone_world.resources.stats.health <= 0.0 {
        brimstone_world.resources.stats.health = 0.0;
        brimstone_world.resources.game.phase = Phase::Dead;
        brimstone_world.resources.game.shake += DEATH_SHAKE;
        let best = brimstone_world
            .resources
            .game
            .best_score
            .max(brimstone_world.resources.game.score);
        if best > brimstone_world.resources.game.best_score {
            save_best(best);
        }
        brimstone_world.resources.game.best_score = best;
        audio::play(brimstone_world, world, audio::PLAYER_DEATH, 1.0);
    }
}

pub fn tick(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);

    {
        let game = &mut brimstone_world.resources.game;
        game.score_flash = (game.score_flash - delta).max(0.0);
        if game.combo_timer > 0.0 {
            game.combo_timer -= delta;
            if game.combo_timer <= 0.0 {
                game.combo = 0;
            }
        }
    }
    brimstone_world.resources.level.banner = (brimstone_world.resources.level.banner - delta).max(0.0);

    decay_overheal(brimstone_world, delta);
    apply_pressure(brimstone_world, world, delta);

    if !brimstone_world.resources.game.spawn_queue.is_empty() {
        brimstone_world.resources.game.spawn_timer -= delta;
        if brimstone_world.resources.game.spawn_timer <= 0.0 {
            if let Some((kind, elite, boss)) = brimstone_world.resources.game.spawn_queue.pop() {
                let position = spawn_point(brimstone_world);
                enemies::spawn(brimstone_world, world, kind, elite, boss, position);
            }
            brimstone_world.resources.game.spawn_timer = spawn_interval(brimstone_world);
        }
    } else if enemies::total_count(brimstone_world) == 0 {
        advance_wave(brimstone_world, world);
    }

    check_keycard(brimstone_world, world);

    if brimstone_world.resources.level.exit_active {
        let player_position = player::position(brimstone_world, world);
        let mut offset = player_position - brimstone_world.resources.level.exit_position;
        offset.y = 0.0;
        if offset.norm() < EXIT_RADIUS {
            if brimstone_world.resources.level.story {
                crate::systems::story::mission_complete(brimstone_world, world);
            } else if brimstone_world.resources.level.custom {
                crate::systems::editor::open(brimstone_world, world);
            } else {
                let next = brimstone_world.resources.level.cycle as usize * content::count()
                    + brimstone_world.resources.level.index
                    + 1;
                load_level(brimstone_world, world, next);
            }
        }
    }
}

fn advance_wave(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if !brimstone_world.resources.game.waves.is_empty() {
        let wave = brimstone_world.resources.game.waves.remove(0);
        brimstone_world.resources.game.spawn_queue = wave;
        brimstone_world.resources.game.spawn_timer = 0.6;
        brimstone_world.resources.level.wave += 1;
        brimstone_world.resources.level.banner = BANNER_TIME;
    } else if !brimstone_world.resources.level.exit_active
        && !matches!(brimstone_world.resources.level.objective, Objective::Keycard)
    {
        level::open_exit(brimstone_world, world);
        brimstone_world.resources.level.banner = BANNER_TIME;
        audio::play(brimstone_world, world, audio::CLEAR, 0.7);
    }
}

/// Unlock the gate the moment the keycard is recovered.
fn check_keycard(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if matches!(brimstone_world.resources.level.objective, Objective::Keycard)
        && brimstone_world.resources.game.has_key
        && !brimstone_world.resources.level.exit_active
    {
        level::open_exit(brimstone_world, world);
        brimstone_world.resources.level.banner = BANNER_TIME;
        audio::play(brimstone_world, world, audio::CLEAR, 0.8);
    }
}

fn decay_overheal(brimstone_world: &mut BrimstoneWorld, delta: f32) {
    let stats = &mut brimstone_world.resources.stats;
    if stats.health > stats.max_health {
        stats.health = (stats.health - tuning::OVERHEAL_DECAY * delta).max(stats.max_health);
    }
}

/// Camp with enemies alive and pressure builds until the horde reinforces,
/// nudging the player to keep pushing rather than turtle in a corner.
fn apply_pressure(brimstone_world: &mut BrimstoneWorld, world: &mut World, delta: f32) {
    brimstone_world.resources.game.since_kill += delta;
    let enemies_alive = enemies::total_count(brimstone_world) > 0;
    let camping = enemies_alive && brimstone_world.resources.game.since_kill > tuning::PRESSURE_GRACE;
    if !camping {
        return;
    }
    brimstone_world.resources.game.pressure += tuning::PRESSURE_BUILD * delta;
    if brimstone_world.resources.game.pressure >= tuning::PRESSURE_SPAWN_AT {
        brimstone_world.resources.game.pressure = 0.0;
        brimstone_world.resources.game.shake += 0.3;
        let position = spawn_point(brimstone_world);
        enemies::spawn(
            brimstone_world,
            world,
            EnemyKind::Swarmer,
            false,
            false,
            position,
        );
    }
}

fn spawn_interval(brimstone_world: &BrimstoneWorld) -> f32 {
    let cycle = brimstone_world.resources.level.cycle as f32;
    let wave = brimstone_world.resources.level.wave as f32;
    (tuning::SPAWN_INTERVAL - cycle * 0.05 - wave * 0.06).max(tuning::SPAWN_INTERVAL_MIN)
}

fn spawn_point(brimstone_world: &mut BrimstoneWorld) -> nalgebra_glm::Vec3 {
    let custom = brimstone_world.resources.level.custom;
    let len = if custom {
        brimstone_world.resources.level.custom_spawns.len()
    } else {
        content::level(brimstone_world.resources.level.index)
            .spawn_points
            .len()
    };
    if len == 0 {
        return vec3(0.0, 0.0, 16.0);
    }
    let pick = (random_range(
        &mut brimstone_world.resources.game.random_state,
        0.0,
        len as f32,
    ) as usize)
        .min(len - 1);
    let (x, z) = if custom {
        brimstone_world.resources.level.custom_spawns[pick]
    } else {
        content::level(brimstone_world.resources.level.index).spawn_points[pick]
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
    brimstone_world: &mut BrimstoneWorld,
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
                can_elite && next_random(&mut brimstone_world.resources.game.random_state) < fraction;
            waves[cursor % count].push((kind, elite, false));
            cursor += 1;
        }
    }

    let last = count - 1;
    for _ in 0..roster.brutes {
        let elite = next_random(&mut brimstone_world.resources.game.random_state) < fraction;
        waves[last].push((EnemyKind::Brute, elite, false));
    }
    waves
}

/// Seed the run RNG and load the persisted best score the first time any session
/// starts this process. Idempotent: later sessions keep the live seed and best.
fn ensure_seeded(brimstone_world: &mut BrimstoneWorld, world: &World) {
    if brimstone_world.resources.game.seeded {
        return;
    }
    let uptime = world.resources.window.timing.uptime_milliseconds;
    brimstone_world.resources.game.random_state = 0x9e37_79b9_7f4a_7c15 ^ (uptime | 1);
    brimstone_world.resources.game.best_score = load_best();
    brimstone_world.resources.game.seeded = true;
}

/// Load `waves` into the spawn schedule: the first wave becomes the live spawn
/// queue, the rest are held back, and the wave counters reset to a level's start.
fn arm_waves(brimstone_world: &mut BrimstoneWorld, mut waves: Vec<Vec<SpawnEntry>>) {
    let first = if waves.is_empty() {
        Vec::new()
    } else {
        waves.remove(0)
    };
    brimstone_world.resources.game.waves = waves;
    brimstone_world.resources.game.spawn_queue = first;
    brimstone_world.resources.game.spawn_timer = 0.6;
    brimstone_world.resources.level.wave = 1;
    brimstone_world.resources.level.wave_count = tuning::WAVES_PER_LEVEL as u32;
}

fn reset_core(brimstone_world: &mut BrimstoneWorld) {
    brimstone_world.resources.stats = Default::default();
    brimstone_world.resources.weapon = Default::default();
    let best = brimstone_world.resources.game.best_score;
    let random_state = brimstone_world.resources.game.random_state;
    brimstone_world.resources.game = Default::default();
    brimstone_world.resources.game.best_score = best;
    brimstone_world.resources.game.random_state = random_state;
    brimstone_world.resources.game.seeded = true;
    brimstone_world.resources.player.dash_timer = 0.0;
    brimstone_world.resources.player.dash_cooldown = 0.0;
    brimstone_world.resources.player.iframes = 0.0;
    brimstone_world.resources.player.spawn_grace = 3;
    brimstone_world.resources.player.wall_run_side = 0;
    brimstone_world.resources.player.wall_run_timer = 0.0;
    brimstone_world.resources.player.wall_run_cooldown = 0.0;
    brimstone_world.resources.player.wall_run_tilt = 0.0;
    brimstone_world.resources.player.wall_run_normal = nalgebra_glm::Vec3::zeros();
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
