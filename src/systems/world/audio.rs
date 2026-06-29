use crate::ecs::BoomerWorld;
use crate::systems::common::next_random;
use nightshade::ecs::audio::components::AudioSource;
use nightshade::ecs::audio::resources::audio_engine_load_sound;
use nightshade::ecs::audio::systems::load_sound_from_bytes;
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const SOUND_TTL: f32 = 3.0;

pub const SHOTGUN: &str = "boom_shotgun";
pub const NAILGUN: &str = "boom_nailgun";
pub const EMPTY: &str = "boom_empty";
pub const ENEMY_HURT: &str = "boom_enemy_hurt";
pub const ENEMY_DEATH: &str = "boom_enemy_death";
pub const PLAYER_HURT: &str = "boom_player_hurt";
pub const PLAYER_DEATH: &str = "boom_player_death";
pub const PICKUP: &str = "boom_pickup";
pub const FIREBALL: &str = "boom_fireball";
pub const DASH: &str = "boom_dash";
pub const WAVE: &str = "boom_wave";

const CLIPS: &[(&str, &[u8])] = &[
    (
        SHOTGUN,
        include_bytes!("../../../assets/kenney/audio/build_thud_0.ogg"),
    ),
    (
        NAILGUN,
        include_bytes!("../../../assets/kenney/audio/build_clang_0.ogg"),
    ),
    (
        EMPTY,
        include_bytes!("../../../assets/kenney/audio/ui_click.ogg"),
    ),
    (
        ENEMY_HURT,
        include_bytes!("../../../assets/kenney/audio/orc_rustle.ogg"),
    ),
    (
        ENEMY_DEATH,
        include_bytes!("../../../assets/kenney/audio/build_thud_2.ogg"),
    ),
    (
        PLAYER_HURT,
        include_bytes!("../../../assets/kenney/audio/gate_creak.ogg"),
    ),
    (
        PLAYER_DEATH,
        include_bytes!("../../../assets/kenney/audio/build_thud_2.ogg"),
    ),
    (
        PICKUP,
        include_bytes!("../../../assets/kenney/audio/coin_pickup.ogg"),
    ),
    (
        FIREBALL,
        include_bytes!("../../../assets/kenney/audio/door_open.ogg"),
    ),
    (
        DASH,
        include_bytes!("../../../assets/kenney/audio/ui_hover.ogg"),
    ),
    (
        WAVE,
        include_bytes!("../../../assets/kenney/audio/gate_creak.ogg"),
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

pub fn play(boomer_world: &mut BoomerWorld, world: &mut World, key: &str, volume: f32) {
    let pitch = 0.93 + next_random(&mut boomer_world.resources.game.random_state) * 0.14;
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
