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
        Screen::LevelSelect => {
            if escape {
                lifecycle::enter(boomer_world, world, Screen::Title);
            }
        }
        Screen::MissionSelect => {
            if escape {
                lifecycle::enter(boomer_world, world, Screen::Title);
            }
        }
        Screen::InGame => {
            if escape || start {
                lifecycle::enter(boomer_world, world, Screen::Paused);
            } else if restart || (confirm && !matches!(phase, Phase::Playing)) {
                if boomer_world.resources.level.story {
                    let mission = boomer_world.resources.story.mission;
                    game::start_mission(boomer_world, world, mission);
                    lifecycle::enter(boomer_world, world, Screen::InGame);
                } else if boomer_world.resources.level.custom {
                    game::start_custom(boomer_world, world);
                    lifecycle::enter(boomer_world, world, Screen::InGame);
                } else {
                    game::start_at(boomer_world, world, 0);
                }
            }
        }
        Screen::Paused => {
            if escape || start {
                lifecycle::enter(boomer_world, world, Screen::InGame);
            }
        }
        Screen::Editor => {
            if escape {
                crate::systems::editor::teardown(boomer_world, world);
                boomer_world.resources.editor.active = false;
                game::start_at(boomer_world, world, 0);
                lifecycle::enter(boomer_world, world, Screen::Title);
            }
        }
        Screen::Cutscene => {}
    }
}
