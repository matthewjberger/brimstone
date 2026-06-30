//! Every gameplay and feel constant in one place, grouped by system.

// Player movement (Quake-style: accelerate toward wishdir, friction with a
// stop-speed, and air control that rewards strafe-jumping with real momentum).
//
// Heights are derived from GRAVITY: apex = impulse^2 / (2*|GRAVITY|). With
// GRAVITY = -18 a 9.9 jump apexes ~2.7m (airtime ~1.1s), clearing the 1.0-2.5m
// platforms; pads apex ~5.1m to reach the tall ledges; wall-jumps ~2.8m. The
// engine's world gravity is set to GRAVITY at startup; only the player is a
// physics body, so nothing else is affected.
pub const GRAVITY: f32 = -18.0;
pub const MOVE_SPEED: f32 = 12.0;
pub const GROUND_ACCEL: f32 = 11.0;
pub const AIR_ACCEL: f32 = 72.0;
pub const AIR_SPEED_CAP: f32 = 1.6;
pub const GROUND_FRICTION: f32 = 8.0;
pub const STOP_SPEED: f32 = 4.0;
pub const MAX_GROUND_SPEED: f32 = 34.0;
pub const JUMP_IMPULSE: f32 = 9.9;
pub const GAMEPAD_DEADZONE: f32 = 0.15;
/// Extra ground wishspeed per combo multiplier step above x1 (fraction of base).
pub const COMBO_SPEED_PER_STEP: f32 = 0.06;

// Jump pads
pub const PAD_RADIUS: f32 = 1.6;
pub const PAD_IMPULSE: f32 = 13.5;

// Wallrunning (ported from the nightshade movement demo, tuned to boom's speed)
pub const WALL_DETECT_DISTANCE: f32 = 0.8;
pub const WALL_RUN_MIN_SPEED: f32 = 6.0;
pub const WALL_RUN_DURATION: f32 = 1.5;
pub const WALL_RUN_FALL_RATE: f32 = -1.6;
pub const WALL_RUN_SPEED: f32 = 14.0;
pub const WALL_RUN_FORWARD_BOOST: f32 = 2.0;
pub const WALL_RUN_STICK: f32 = 4.0;
pub const WALL_RUN_COOLDOWN: f32 = 0.22;
pub const WALL_RUN_CAMERA_TILT: f32 = 0.16;
pub const WALL_RUN_TILT_LERP: f32 = 14.0;
pub const WALL_RUN_FOV_POP: f32 = 6.0;
pub const WALL_JUMP_LATERAL: f32 = 12.0;
pub const WALL_JUMP_VERTICAL: f32 = 10.0;
pub const WALL_JUMP_FORWARD: f32 = 7.0;

// Dash
pub const DASH_SPEED: f32 = 26.0;
pub const DASH_TIME: f32 = 0.13;
pub const DASH_COOLDOWN: f32 = 0.85;
pub const DASH_IFRAMES: f32 = 0.2;
pub const DASH_SHAKE: f32 = 0.1;

// Weapons (shared)
pub const WEAPON_RANGE: f32 = 70.0;

// Shotgun
pub const SHOTGUN_PELLETS: u32 = 10;
pub const SHOTGUN_SPREAD: f32 = 0.07;
pub const SHOTGUN_DAMAGE: f32 = 9.0;
pub const SHOTGUN_COOLDOWN: f32 = 0.55;
pub const SHOTGUN_KNOCKBACK: f32 = 11.0;
pub const SHOTGUN_SHAKE: f32 = 0.32;
pub const SHOTGUN_KICK: f32 = 0.4;
pub const SHOTGUN_FOV_POP: f32 = 6.0;
pub const SHOTGUN_START: u32 = 24;
pub const SHOTGUN_MAX: u32 = 60;
pub const SHOTGUN_PICKUP: u32 = 8;

// Nailgun (cheap, ammo-rich, sustained DPS)
pub const NAIL_DAMAGE: f32 = 7.0;
pub const NAIL_SPREAD: f32 = 0.02;
pub const NAIL_COOLDOWN: f32 = 0.085;
pub const NAIL_KNOCKBACK: f32 = 2.5;
pub const NAIL_SHAKE: f32 = 0.035;
pub const NAIL_KICK: f32 = 0.09;
pub const NAIL_FOV_POP: f32 = 1.6;
pub const NAIL_START: u32 = 80;
pub const NAIL_MAX: u32 = 200;
pub const NAIL_PICKUP: u32 = 40;

// Rocket launcher (splash, crowd control, rocket-jumps)
pub const ROCKET_DAMAGE: f32 = 60.0;
pub const ROCKET_SPLASH_DAMAGE: f32 = 48.0;
pub const ROCKET_SPLASH_RADIUS: f32 = 4.6;
pub const ROCKET_SPEED: f32 = 22.0;
pub const ROCKET_COOLDOWN: f32 = 0.85;
pub const ROCKET_KNOCKBACK: f32 = 16.0;
pub const ROCKET_SHAKE: f32 = 0.5;
pub const ROCKET_KICK: f32 = 0.6;
pub const ROCKET_FOV_POP: f32 = 8.0;
pub const ROCKET_LIFETIME: f32 = 5.0;
pub const ROCKET_SCALE: f32 = 0.55;
pub const ROCKET_HITSTOP: f32 = 0.05;
pub const ROCKET_START: u32 = 5;
pub const ROCKET_MAX: u32 = 20;
pub const ROCKET_PICKUP: u32 = 3;
/// Self-effects when caught in your own blast: enables rocket-jumps.
pub const ROCKET_SELF_DAMAGE: f32 = 12.0;
pub const ROCKET_SELF_PUSH: f32 = 15.0;

// Pistol (weak infinite-ammo sidearm — the always-available fallback). Low DPS
// on purpose: enough to kill your way back into ammo when every other pool is
// dry, never enough to retire the real guns.
pub const PISTOL_DAMAGE: f32 = 11.0;
pub const PISTOL_SPREAD: f32 = 0.015;
pub const PISTOL_COOLDOWN: f32 = 0.26;
pub const PISTOL_KNOCKBACK: f32 = 2.0;
pub const PISTOL_SHAKE: f32 = 0.05;
pub const PISTOL_KICK: f32 = 0.12;
pub const PISTOL_FOV_POP: f32 = 1.4;

// Railgun (piercing hitscan beam — punches through a whole line of enemies)
pub const RAIL_DAMAGE: f32 = 85.0;
pub const RAIL_COOLDOWN: f32 = 1.1;
pub const RAIL_KNOCKBACK: f32 = 14.0;
pub const RAIL_SHAKE: f32 = 0.45;
pub const RAIL_KICK: f32 = 0.5;
pub const RAIL_FOV_POP: f32 = 7.0;
pub const RAIL_HITSTOP: f32 = 0.05;
pub const RAIL_START: u32 = 12;
pub const RAIL_MAX: u32 = 40;
pub const RAIL_PICKUP: u32 = 5;

// Enemy shared
pub const ENEMY_HIT_FLASH: f32 = 0.1;
pub const ENEMY_DEATH_TIME: f32 = 0.32;
pub const ENEMY_KNOCKBACK_DECAY: f32 = 6.0;
pub const ENEMY_SEPARATION: f32 = 1.4;
pub const ENEMY_HIT_RADIUS: f32 = 0.75;
pub const ENEMY_CENTER_HEIGHT: f32 = 1.0;

// Elite variants (scale behaviour, not just counts)
pub const ELITE_HEALTH_MULT: f32 = 2.0;
pub const ELITE_DAMAGE_MULT: f32 = 1.4;
pub const ELITE_SCALE: f32 = 1.25;
pub const ELITE_SCORE_MULT: u32 = 2;

// Boss / warlord (a single climactic enemy in Boss missions)
// At 4.5x an elite brute is ~1170 HP: a real fight (~7s of focused shotgun
// fire, longer while dodging) without becoming a bullet-sponge slog.
pub const BOSS_HEALTH_MULT: f32 = 4.5;
pub const BOSS_DAMAGE_MULT: f32 = 1.8;
pub const BOSS_SCALE: f32 = 1.7;
pub const BOSS_SCORE_MULT: u32 = 8;

// Imp
pub const IMP_HEALTH: f32 = 30.0;
pub const IMP_SPEED: f32 = 3.8;
pub const IMP_DAMAGE: f32 = 12.0;
pub const IMP_ATTACK_RANGE: f32 = 2.0;
pub const IMP_ATTACK_COOLDOWN: f32 = 1.0;
pub const IMP_WINDUP: f32 = 0.45;
pub const IMP_LUNGE: f32 = 7.0;
pub const IMP_LUNGE_REACH: f32 = 1.0;
pub const IMP_SCORE: u32 = 100;
pub const IMP_WIDTH: f32 = 1.5;
pub const IMP_HEIGHT: f32 = 1.9;

// Swarmer
pub const SWARM_HEALTH: f32 = 10.0;
pub const SWARM_SPEED: f32 = 6.8;
pub const SWARM_DAMAGE: f32 = 6.0;
pub const SWARM_ATTACK_RANGE: f32 = 1.7;
pub const SWARM_ATTACK_COOLDOWN: f32 = 0.8;
pub const SWARM_WINDUP: f32 = 0.22;
pub const SWARM_LUNGE: f32 = 10.0;
pub const SWARM_LUNGE_REACH: f32 = 1.2;
pub const SWARM_SCORE: u32 = 60;
pub const SWARM_WIDTH: f32 = 1.1;
pub const SWARM_HEIGHT: f32 = 1.3;

// Brute (heavy bruiser, slow telegraphed slam)
pub const BRUTE_HEALTH: f32 = 130.0;
pub const BRUTE_SPEED: f32 = 2.6;
pub const BRUTE_DAMAGE: f32 = 22.0;
pub const BRUTE_ATTACK_RANGE: f32 = 2.6;
pub const BRUTE_ATTACK_COOLDOWN: f32 = 1.6;
pub const BRUTE_WINDUP: f32 = 0.7;
pub const BRUTE_LUNGE: f32 = 9.0;
pub const BRUTE_LUNGE_REACH: f32 = 1.6;
pub const BRUTE_SCORE: u32 = 250;
pub const BRUTE_WIDTH: f32 = 2.6;
pub const BRUTE_HEIGHT: f32 = 3.2;

// Gargoyle (flying melee diver)
pub const GARGOYLE_HEALTH: f32 = 24.0;
pub const GARGOYLE_SPEED: f32 = 5.0;
pub const GARGOYLE_DAMAGE: f32 = 11.0;
pub const GARGOYLE_ATTACK_RANGE: f32 = 2.6;
pub const GARGOYLE_ATTACK_COOLDOWN: f32 = 1.3;
pub const GARGOYLE_WINDUP: f32 = 0.4;
pub const GARGOYLE_LUNGE: f32 = 13.0;
pub const GARGOYLE_LUNGE_REACH: f32 = 1.5;
pub const GARGOYLE_SCORE: u32 = 180;
pub const GARGOYLE_WIDTH: f32 = 1.9;
pub const GARGOYLE_HEIGHT: f32 = 1.7;
pub const GARGOYLE_HOVER: f32 = 3.2;
pub const GARGOYLE_ALT_MIN: f32 = 0.6;
pub const GARGOYLE_ALT_MAX: f32 = 6.5;

// Sentinel (flying ranged: hovers high and lobs fireballs)
pub const SENTINEL_HEALTH: f32 = 18.0;
pub const SENTINEL_SPEED: f32 = 3.6;
pub const SENTINEL_PREFERRED_RANGE: f32 = 12.0;
pub const SENTINEL_FIRE_COOLDOWN: f32 = 2.2;
pub const SENTINEL_WINDUP: f32 = 0.4;
pub const SENTINEL_SCORE: u32 = 160;
pub const SENTINEL_WIDTH: f32 = 1.5;
pub const SENTINEL_HEIGHT: f32 = 1.5;
pub const SENTINEL_HOVER: f32 = 3.4;

// Enemy animation
pub const ANIM_FPS: f32 = 7.0;

// Caster
pub const CASTER_HEALTH: f32 = 22.0;
pub const CASTER_SPEED: f32 = 2.8;
pub const CASTER_PREFERRED_RANGE: f32 = 11.0;
pub const CASTER_FIRE_COOLDOWN: f32 = 2.7;
pub const CASTER_WINDUP: f32 = 0.45;
pub const CASTER_SCORE: u32 = 150;
pub const CASTER_WIDTH: f32 = 1.4;
pub const CASTER_HEIGHT: f32 = 1.8;

// Fireball projectile
pub const FIREBALL_SPEED: f32 = 13.0;
pub const FIREBALL_DAMAGE: f32 = 12.0;
pub const FIREBALL_RADIUS: f32 = 0.8;
pub const FIREBALL_LIFETIME: f32 = 4.0;
pub const FIREBALL_SCALE: f32 = 0.8;

// Player damage feel
pub const PLAYER_HIT_SHAKE: f32 = 0.5;
pub const PLAYER_HIT_KICK: f32 = 0.6;
pub const PLAYER_HIT_FOV_POP: f32 = 7.0;
pub const DAMAGE_FLASH_TIME: f32 = 0.5;
pub const LOW_HEALTH_FRACTION: f32 = 0.3;

// Health / overheal
pub const MAX_HEALTH: f32 = 100.0;
pub const OVERHEAL_MAX: f32 = 150.0;
pub const OVERHEAL_DECAY: f32 = 3.0;

// Camera / juice
pub const FOV_BASE_DEGREES: f32 = 78.0;
pub const SHAKE_FREQ_X: f32 = 92.0;
pub const SHAKE_FREQ_Y: f32 = 71.0;
pub const SHAKE_AMPLITUDE: f32 = 0.09;
pub const SHAKE_DECAY: f32 = 5.0;
pub const KICK_DECAY: f32 = 8.0;
pub const FOV_POP_DECAY: f32 = 7.0;
pub const HITSTOP_SHOTGUN: f32 = 0.03;

// Scoring
pub const COMBO_WINDOW: f32 = 3.2;
pub const SCORE_FLASH_TIME: f32 = 0.35;
/// Granted each time the combo multiplier steps up.
pub const COMBO_REWARD_HEAL: f32 = 12.0;
pub const COMBO_REWARD_SHELLS: u32 = 4;
pub const COMBO_REWARD_NAILS: u32 = 20;
pub const COMBO_REWARD_ROCKETS: u32 = 1;

// Anti-camping pressure: idle too long with enemies alive and the horde stirs.
pub const PRESSURE_GRACE: f32 = 5.0;
pub const PRESSURE_BUILD: f32 = 1.0;
pub const PRESSURE_SPAWN_AT: f32 = 4.0;

// Spawning
pub const SPAWN_INTERVAL: f32 = 0.7;
pub const SPAWN_INTERVAL_MIN: f32 = 0.28;
pub const WAVES_PER_LEVEL: usize = 3;

// Pickups
pub const PICKUP_RADIUS: f32 = 1.5;
pub const HEALTH_PICKUP_AMOUNT: f32 = 25.0;
pub const PICKUP_HOVER: f32 = 0.6;
pub const PICKUP_BOB_HEIGHT: f32 = 0.18;
pub const PICKUP_BOB_SPEED: f32 = 2.5;
pub const AMMO_DROP_CHANCE: f32 = 0.22;

// Arena
pub const ARENA_HALF: f32 = 19.0;
