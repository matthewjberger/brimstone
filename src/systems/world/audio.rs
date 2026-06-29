use crate::ecs::BoomerWorld;
use nightshade::ecs::audio::components::AudioSource;
use nightshade::ecs::audio::resources::audio_engine_load_sound;
use nightshade::ecs::audio::systems::load_sound_from_bytes;
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const SOUND_TTL: f32 = 3.0;

pub const SHOOT: &str = "boomer_shoot";
pub const EMPTY: &str = "boomer_empty";
pub const IMP_HURT: &str = "boomer_imp_hurt";
pub const IMP_DEATH: &str = "boomer_imp_death";
pub const PLAYER_HURT: &str = "boomer_player_hurt";
pub const PICKUP: &str = "boomer_pickup";

pub fn load(world: &mut World) {
    register(
        world,
        SHOOT,
        include_bytes!("../../../assets/kenney/audio/build_thud_0.ogg"),
    );
    register(
        world,
        EMPTY,
        include_bytes!("../../../assets/kenney/audio/ui_click.ogg"),
    );
    register(
        world,
        IMP_HURT,
        include_bytes!("../../../assets/kenney/audio/orc_rustle.ogg"),
    );
    register(
        world,
        IMP_DEATH,
        include_bytes!("../../../assets/kenney/audio/build_thud_2.ogg"),
    );
    register(
        world,
        PLAYER_HURT,
        include_bytes!("../../../assets/kenney/audio/build_clang_0.ogg"),
    );
    register(
        world,
        PICKUP,
        include_bytes!("../../../assets/kenney/audio/coin_pickup.ogg"),
    );
}

fn register(world: &mut World, key: &str, bytes: &'static [u8]) {
    match load_sound_from_bytes(bytes) {
        Ok(sound) => audio_engine_load_sound(&mut world.resources.audio, key.to_string(), sound),
        Err(error) => tracing::error!("failed to load audio '{}': {}", key, error),
    }
}

pub fn play(boomer_world: &mut BoomerWorld, world: &mut World, key: &str, volume: f32) {
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | AUDIO_SOURCE,
        1,
    )[0];
    world.core.set_audio_source(
        entity,
        AudioSource::new(key.to_string())
            .with_volume(volume)
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
