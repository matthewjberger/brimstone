use crate::ecs::{AdvPanel, BrimstoneWorld, Phase, Screen};
use crate::systems::lifecycle;
use crate::systems::world::game;
use nightshade::prelude::*;

pub fn handle_global(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
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
    let phase = brimstone_world.resources.game.phase;

    match brimstone_world.resources.screen.current {
        Screen::Title => {
            if escape {
                world.resources.window.should_exit = true;
            }
        }
        Screen::LevelSelect => {
            if escape {
                lifecycle::enter(brimstone_world, world, Screen::Title);
            }
        }
        Screen::MissionSelect => {
            if escape {
                lifecycle::enter(brimstone_world, world, Screen::Title);
            }
        }
        Screen::InGame => {
            if escape || start {
                lifecycle::enter(brimstone_world, world, Screen::Paused);
            } else if restart || (confirm && !matches!(phase, Phase::Playing)) {
                game::restart_current(brimstone_world, world);
                lifecycle::enter(brimstone_world, world, Screen::InGame);
            }
        }
        Screen::Paused => {
            if escape || start {
                lifecycle::enter(brimstone_world, world, Screen::InGame);
            }
        }
        Screen::Editor => {
            if escape {
                crate::systems::editor::teardown(brimstone_world, world);
                brimstone_world.resources.editor.active = false;
                game::start_at(brimstone_world, world, 0);
                lifecycle::enter(brimstone_world, world, Screen::Title);
            }
        }
        Screen::Adventure => {
            // Esc closes an open menu (handled in the adventure update); with no
            // menu open it leaves to the title screen.
            if (escape || start) && brimstone_world.resources.adventure.panel == AdvPanel::None {
                crate::adventure::leave(brimstone_world, world);
            }
        }
        Screen::Cutscene => {}
    }
}
