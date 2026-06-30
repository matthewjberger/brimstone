use crate::ecs::BoomerWorld;
use crate::systems::common::next_random;
use nightshade::ecs::audio::components::AudioSource;
use nightshade::ecs::audio::resources::audio_engine_load_sound;
use nightshade::ecs::audio::systems::load_sound_from_bytes;
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const SOUND_TTL: f32 = 3.0;

// Each event is a round-robin set of clip keys; `play` cycles through them so
// repeated sounds vary. All samples are from the se_old_pack00 sound pack.
pub const SHOTGUN: &[&str] = &["sfx_gun00", "sfx_gun01", "sfx_gun02", "sfx_gun03"];
pub const NAILGUN: &[&str] = &["sfx_kachi00", "sfx_kachi01", "sfx_kachi02", "sfx_kachi03"];
pub const ROCKET: &[&str] = &["sfx_swing00", "sfx_swing01", "sfx_swing02"];
pub const EXPLOSION: &[&str] = &["sfx_bom00", "sfx_bom01", "sfx_bom02", "sfx_bom03"];
pub const EMPTY: &[&str] = &["sfx_cursor01", "sfx_cursor02", "sfx_cursor03"];
pub const ENEMY_HURT: &[&str] = &["sfx_hit00", "sfx_hit01", "sfx_hit02", "sfx_hit03"];
pub const ENEMY_DEATH: &[&str] = &["sfx_crash00", "sfx_crash01", "sfx_crash02", "sfx_crash03"];
pub const PLAYER_HURT: &[&str] = &["sfx_don00", "sfx_don01", "sfx_don02"];
pub const PLAYER_DEATH: &[&str] = &["sfx_gara00", "sfx_gara01", "sfx_gara02"];
pub const PICKUP: &[&str] = &["sfx_coin00", "sfx_coin01", "sfx_coin02", "sfx_coin03"];
pub const FIREBALL: &[&str] = &["sfx_fire00", "sfx_fire01", "sfx_fire02"];
pub const DASH: &[&str] = &["sfx_push00", "sfx_push01", "sfx_push02"];
pub const PAD: &[&str] = &["sfx_power00", "sfx_power02", "sfx_power03"];
pub const BOSS: &[&str] = &["sfx_alarm00", "sfx_alarm01"];
pub const CLEAR: &[&str] = &["sfx_bell00", "sfx_bell01", "sfx_bell02"];
pub const RAILGUN: &[&str] = &["sfx_biri00", "sfx_biri01"];

const CLIPS: &[(&str, &[u8])] = &[
    ("sfx_gun00", include_bytes!("../../../assets/sfx/gun00.wav")),
    ("sfx_gun01", include_bytes!("../../../assets/sfx/gun01.wav")),
    ("sfx_gun02", include_bytes!("../../../assets/sfx/gun02.wav")),
    ("sfx_gun03", include_bytes!("../../../assets/sfx/gun03.wav")),
    (
        "sfx_kachi00",
        include_bytes!("../../../assets/sfx/kachi00.wav"),
    ),
    (
        "sfx_kachi01",
        include_bytes!("../../../assets/sfx/kachi01.wav"),
    ),
    (
        "sfx_kachi02",
        include_bytes!("../../../assets/sfx/kachi02.wav"),
    ),
    (
        "sfx_kachi03",
        include_bytes!("../../../assets/sfx/kachi03.wav"),
    ),
    (
        "sfx_swing00",
        include_bytes!("../../../assets/sfx/swing00.wav"),
    ),
    (
        "sfx_swing01",
        include_bytes!("../../../assets/sfx/swing01.wav"),
    ),
    (
        "sfx_swing02",
        include_bytes!("../../../assets/sfx/swing02.wav"),
    ),
    ("sfx_bom00", include_bytes!("../../../assets/sfx/bom00.wav")),
    ("sfx_bom01", include_bytes!("../../../assets/sfx/bom01.wav")),
    ("sfx_bom02", include_bytes!("../../../assets/sfx/bom02.wav")),
    ("sfx_bom03", include_bytes!("../../../assets/sfx/bom03.wav")),
    (
        "sfx_cursor01",
        include_bytes!("../../../assets/sfx/cursor01.wav"),
    ),
    (
        "sfx_cursor02",
        include_bytes!("../../../assets/sfx/cursor02.wav"),
    ),
    (
        "sfx_cursor03",
        include_bytes!("../../../assets/sfx/cursor03.wav"),
    ),
    ("sfx_hit00", include_bytes!("../../../assets/sfx/hit00.wav")),
    ("sfx_hit01", include_bytes!("../../../assets/sfx/hit01.wav")),
    ("sfx_hit02", include_bytes!("../../../assets/sfx/hit02.wav")),
    ("sfx_hit03", include_bytes!("../../../assets/sfx/hit03.wav")),
    (
        "sfx_crash00",
        include_bytes!("../../../assets/sfx/crash00.wav"),
    ),
    (
        "sfx_crash01",
        include_bytes!("../../../assets/sfx/crash01.wav"),
    ),
    (
        "sfx_crash02",
        include_bytes!("../../../assets/sfx/crash02.wav"),
    ),
    (
        "sfx_crash03",
        include_bytes!("../../../assets/sfx/crash03.wav"),
    ),
    ("sfx_don00", include_bytes!("../../../assets/sfx/don00.wav")),
    ("sfx_don01", include_bytes!("../../../assets/sfx/don01.wav")),
    ("sfx_don02", include_bytes!("../../../assets/sfx/don02.wav")),
    (
        "sfx_gara00",
        include_bytes!("../../../assets/sfx/gara00.wav"),
    ),
    (
        "sfx_gara01",
        include_bytes!("../../../assets/sfx/gara01.wav"),
    ),
    (
        "sfx_gara02",
        include_bytes!("../../../assets/sfx/gara02.wav"),
    ),
    (
        "sfx_coin00",
        include_bytes!("../../../assets/sfx/coin00.wav"),
    ),
    (
        "sfx_coin01",
        include_bytes!("../../../assets/sfx/coin01.wav"),
    ),
    (
        "sfx_coin02",
        include_bytes!("../../../assets/sfx/coin02.wav"),
    ),
    (
        "sfx_coin03",
        include_bytes!("../../../assets/sfx/coin03.wav"),
    ),
    (
        "sfx_fire00",
        include_bytes!("../../../assets/sfx/fire00.wav"),
    ),
    (
        "sfx_fire01",
        include_bytes!("../../../assets/sfx/fire01.wav"),
    ),
    (
        "sfx_fire02",
        include_bytes!("../../../assets/sfx/fire02.wav"),
    ),
    (
        "sfx_push00",
        include_bytes!("../../../assets/sfx/push00.wav"),
    ),
    (
        "sfx_push01",
        include_bytes!("../../../assets/sfx/push01.wav"),
    ),
    (
        "sfx_push02",
        include_bytes!("../../../assets/sfx/push02.wav"),
    ),
    (
        "sfx_power00",
        include_bytes!("../../../assets/sfx/power00.wav"),
    ),
    (
        "sfx_power02",
        include_bytes!("../../../assets/sfx/power02.wav"),
    ),
    (
        "sfx_power03",
        include_bytes!("../../../assets/sfx/power03.wav"),
    ),
    (
        "sfx_alarm00",
        include_bytes!("../../../assets/sfx/alarm00.wav"),
    ),
    (
        "sfx_alarm01",
        include_bytes!("../../../assets/sfx/alarm01.wav"),
    ),
    (
        "sfx_bell00",
        include_bytes!("../../../assets/sfx/bell00.wav"),
    ),
    (
        "sfx_bell01",
        include_bytes!("../../../assets/sfx/bell01.wav"),
    ),
    (
        "sfx_bell02",
        include_bytes!("../../../assets/sfx/bell02.wav"),
    ),
    (
        "sfx_biri00",
        include_bytes!("../../../assets/sfx/biri00.wav"),
    ),
    (
        "sfx_biri01",
        include_bytes!("../../../assets/sfx/biri01.wav"),
    ),
];

pub fn load(world: &mut World) {
    for (key, bytes) in CLIPS {
        match load_sound_from_bytes(bytes) {
            Ok(sound) => {
                audio_engine_load_sound(&mut world.resources.audio, key.to_string(), sound)
            }
            Err(error) => tracing::error!("failed to load audio '{}': {}", key, error),
        }
    }
}

pub fn play(boomer_world: &mut BoomerWorld, world: &mut World, set: &[&'static str], volume: f32) {
    if set.is_empty() {
        return;
    }
    let index = {
        let counter = boomer_world
            .resources
            .audio
            .round_robin
            .entry(set[0])
            .or_insert(0);
        let current = *counter;
        *counter = counter.wrapping_add(1);
        current
    };
    let key = set[index % set.len()];
    let pitch = 0.94 + next_random(&mut boomer_world.resources.game.random_state) * 0.12;
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | AUDIO_SOURCE,
        1,
    )[0];
    world.core.set_audio_source(
        entity,
        AudioSource::new(key.to_string())
            .with_volume(volume)
            .with_playback_rate(pitch as f64)
            .playing(),
    );
    boomer_world
        .resources
        .audio
        .sources
        .push((entity, SOUND_TTL));
}

pub fn tick(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let mut index = 0;
    while index < boomer_world.resources.audio.sources.len() {
        boomer_world.resources.audio.sources[index].1 -= delta;
        if boomer_world.resources.audio.sources[index].1 <= 0.0 {
            let (entity, _) = boomer_world.resources.audio.sources.swap_remove(index);
            queue_ecs_command(world, EcsCommand::DespawnRecursive { entity });
        } else {
            index += 1;
        }
    }
}
