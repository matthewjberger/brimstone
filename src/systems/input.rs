use crate::ecs::{BoomerWorld, Phase, Screen};
use crate::systems::lifecycle;
use crate::systems::world::game;
use nightshade::prelude::*;

pub fn handle_global(boomer_world: &mut BoomerWorld, world: &mut World) {
    let escape = world.resources.input.keyboard.just_pressed(KeyCode::Escape);
    let restart = world.resources.input.keyboard.just_pressed(KeyCode::KeyR);
    let start = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::Start);
    let confirm = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::South);
    let phase = boomer_world.resources.game.phase;

    match boomer_world.resources.screen.current {
        Screen::Title => {
            if escape {
                world.resources.window.should_exit = true;
            }
        }
        Screen::InGame => {
            if escape || start {
                lifecycle::enter(boomer_world, world, Screen::Paused);
            } else if restart || (confirm && !matches!(phase, Phase::Playing)) {
                game::reset(boomer_world, world);
            }
        }
        Screen::Paused => {
            if escape || start {
                lifecycle::enter(boomer_world, world, Screen::InGame);
            }
        }
    }
}
