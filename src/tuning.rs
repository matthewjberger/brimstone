//! Every gameplay and feel constant in one place, grouped by system.

// Player movement
pub const MOVE_SPEED: f32 = 11.0;
pub const SPRINT_MULTIPLIER: f32 = 1.5;
pub const GROUND_ACCEL: f32 = 14.0;
pub const AIR_ACCEL: f32 = 9.0;
pub const GROUND_FRICTION: f32 = 11.0;
pub const JUMP_IMPULSE: f32 = 7.0;
pub const GAMEPAD_DEADZONE: f32 = 0.15;

// Dash
pub const DASH_SPEED: f32 = 26.0;
pub const DASH_TIME: f32 = 0.13;
pub const DASH_COOLDOWN: f32 = 0.85;
pub const DASH_IFRAMES: f32 = 0.2;
pub const DASH_SHAKE: f32 = 0.18;

// Weapons (shared)
pub const WEAPON_RANGE: f32 = 70.0;
pub const START_AMMO: u32 = 40;
pub const MAX_AMMO: u32 = 120;

// Shotgun
pub const SHOTGUN_PELLETS: u32 = 10;
pub const SHOTGUN_SPREAD: f32 = 0.07;
pub const SHOTGUN_DAMAGE: f32 = 9.0;
pub const SHOTGUN_COOLDOWN: f32 = 0.55;
pub const SHOTGUN_KNOCKBACK: f32 = 11.0;
pub const SHOTGUN_SHAKE: f32 = 0.5;
pub const SHOTGUN_KICK: f32 = 0.4;
pub const SHOTGUN_FOV_POP: f32 = 6.0;

// Nailgun
pub const NAIL_DAMAGE: f32 = 7.0;
pub const NAIL_SPREAD: f32 = 0.02;
pub const NAIL_COOLDOWN: f32 = 0.085;
pub const NAIL_KNOCKBACK: f32 = 2.5;
pub const NAIL_SHAKE: f32 = 0.12;
pub const NAIL_KICK: f32 = 0.09;
pub const NAIL_FOV_POP: f32 = 1.6;

// Enemy shared
pub const ENEMY_HIT_FLASH: f32 = 0.1;
pub const ENEMY_DEATH_TIME: f32 = 0.32;
pub const ENEMY_KNOCKBACK_DECAY: f32 = 6.0;
pub const ENEMY_SEPARATION: f32 = 1.4;
pub const ENEMY_HIT_RADIUS: f32 = 0.75;
pub const ENEMY_CENTER_HEIGHT: f32 = 1.0;

// Imp
pub const IMP_HEALTH: f32 = 30.0;
pub const IMP_SPEED: f32 = 3.8;
pub const IMP_DAMAGE: f32 = 9.0;
pub const IMP_ATTACK_RANGE: f32 = 2.0;
pub const IMP_ATTACK_COOLDOWN: f32 = 1.0;
pub const IMP_SCORE: u32 = 100;
pub const IMP_WIDTH: f32 = 1.5;
pub const IMP_HEIGHT: f32 = 1.9;

// Swarmer
pub const SWARM_HEALTH: f32 = 10.0;
pub const SWARM_SPEED: f32 = 6.8;
pub const SWARM_DAMAGE: f32 = 6.0;
pub const SWARM_ATTACK_RANGE: f32 = 1.7;
pub const SWARM_ATTACK_COOLDOWN: f32 = 0.8;
pub const SWARM_SCORE: u32 = 60;
pub const SWARM_WIDTH: f32 = 1.1;
pub const SWARM_HEIGHT: f32 = 1.3;

// Caster
pub const CASTER_HEALTH: f32 = 22.0;
pub const CASTER_SPEED: f32 = 2.8;
pub const CASTER_PREFERRED_RANGE: f32 = 11.0;
pub const CASTER_FIRE_COOLDOWN: f32 = 2.7;
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
pub const PLAYER_HIT_SHAKE: f32 = 0.7;
pub const PLAYER_HIT_KICK: f32 = 0.6;
pub const PLAYER_HIT_FOV_POP: f32 = 7.0;
pub const DAMAGE_FLASH_TIME: f32 = 0.5;
pub const LOW_HEALTH_FRACTION: f32 = 0.3;

// Camera / juice
pub const FOV_BASE_DEGREES: f32 = 90.0;
pub const SHAKE_FREQ_X: f32 = 92.0;
pub const SHAKE_FREQ_Y: f32 = 71.0;
pub const SHAKE_AMPLITUDE: f32 = 0.14;
pub const SHAKE_DECAY: f32 = 2.6;
pub const KICK_DECAY: f32 = 8.0;
pub const FOV_POP_DECAY: f32 = 7.0;
pub const HITSTOP_SHOTGUN: f32 = 0.03;

// Scoring
pub const COMBO_WINDOW: f32 = 3.2;
pub const SCORE_FLASH_TIME: f32 = 0.35;

// Spawning
pub const SPAWN_INTERVAL: f32 = 0.7;
pub const SPAWN_INTERVAL_MIN: f32 = 0.28;

// Pickups
pub const PICKUP_RADIUS: f32 = 1.5;
pub const HEALTH_PICKUP_AMOUNT: f32 = 25.0;
pub const AMMO_PICKUP_AMOUNT: u32 = 18;
pub const PICKUP_HOVER: f32 = 0.6;
pub const PICKUP_BOB_HEIGHT: f32 = 0.18;
pub const PICKUP_BOB_SPEED: f32 = 2.5;
pub const AMMO_DROP_CHANCE: f32 = 0.22;

// Arena
pub const ARENA_HALF: f32 = 19.0;
