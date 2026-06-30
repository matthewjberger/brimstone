use crate::ecs::{CobaltWorld, Phase, Screen};
use crate::systems::lifecycle;
use crate::systems::world::game;
use nightshade::prelude::*;

pub fn handle_global(cobalt_world: &mut CobaltWorld, world: &mut World) {
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
    let phase = cobalt_world.resources.game.phase;

    match cobalt_world.resources.screen.current {
        Screen::Title => {
            if escape {
                world.resources.window.should_exit = true;
            }
        }
        Screen::LevelSelect => {
            if escape {
                lifecycle::enter(cobalt_world, world, Screen::Title);
            }
        }
        Screen::MissionSelect => {
            if escape {
                lifecycle::enter(cobalt_world, world, Screen::Title);
            }
        }
        Screen::InGame => {
            if escape || start {
                lifecycle::enter(cobalt_world, world, Screen::Paused);
            } else if restart || (confirm && !matches!(phase, Phase::Playing)) {
                game::restart_current(cobalt_world, world);
                lifecycle::enter(cobalt_world, world, Screen::InGame);
            }
        }
        Screen::Paused => {
            if escape || start {
                lifecycle::enter(cobalt_world, world, Screen::InGame);
            }
        }
        Screen::Editor => {
            if escape {
                crate::systems::editor::teardown(cobalt_world, world);
                cobalt_world.resources.editor.active = false;
                game::start_at(cobalt_world, world, 0);
                lifecycle::enter(cobalt_world, world, Screen::Title);
            }
        }
        Screen::Cutscene => {}
    }
}
