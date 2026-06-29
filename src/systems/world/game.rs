use crate::ecs::{BoomerWorld, Phase};
use crate::systems::world::{audio, enemies, pickups, player};
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const START_AMMO: u32 = 24;

pub fn start(boomer_world: &mut BoomerWorld, world: &mut World) {
    reset_state(boomer_world);
    pickups::spawn_all(boomer_world, world);
    enemies::start_first_wave(boomer_world, world);
}

pub fn reset(boomer_world: &mut BoomerWorld, world: &mut World) {
    enemies::despawn_all(boomer_world, world);
    pickups::despawn_all(boomer_world, world);
    despawn_flashes(boomer_world, world);
    reset_state(boomer_world);
    player::reset(boomer_world, world);
    pickups::spawn_all(boomer_world, world);
    enemies::start_first_wave(boomer_world, world);
}

pub fn damage_player(boomer_world: &mut BoomerWorld, world: &mut World, amount: f32) {
    if !matches!(boomer_world.resources.game.phase, Phase::Playing) {
        return;
    }
    boomer_world.resources.stats.health -= amount;
    boomer_world.resources.game.damage_flash = 0.5;
    audio::play(boomer_world, world, audio::PLAYER_HURT, 0.7);
    if boomer_world.resources.stats.health <= 0.0 {
        boomer_world.resources.stats.health = 0.0;
        boomer_world.resources.game.phase = Phase::Dead;
    }
}

fn reset_state(boomer_world: &mut BoomerWorld) {
    boomer_world.resources.stats.health = boomer_world.resources.stats.max_health;
    boomer_world.resources.weapon.ammo = START_AMMO;
    boomer_world.resources.weapon.cooldown = 0.0;
    boomer_world.resources.game.phase = Phase::Playing;
    boomer_world.resources.game.wave = 0;
    boomer_world.resources.game.damage_flash = 0.0;
}

fn despawn_flashes(boomer_world: &mut BoomerWorld, world: &mut World) {
    for flash in boomer_world.resources.flashes.items.drain(..) {
        queue_ecs_command(
            world,
            EcsCommand::DespawnRecursive {
                entity: flash.entity,
            },
        );
    }
}
