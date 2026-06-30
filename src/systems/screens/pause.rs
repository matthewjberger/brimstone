use crate::ecs::{CobaltWorld, PauseHandles, Screen};
use crate::systems::lifecycle;
use crate::systems::screens::menu_button;
use crate::systems::world::game;
use crate::theme::*;
use nightshade::prelude::*;

pub fn build(tree: &mut UiTreeBuilder) -> PauseHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_intro(UiAnimationType::Fade, 0.18)
        .with_visible(false)
        .entity();

    let mut resume_button = Entity::default();
    let mut restart_button = Entity::default();
    let mut difficulty_button = Entity::default();
    let mut difficulty_label = Entity::default();
    let mut main_menu_button = Entity::default();
    let mut quit_button = Entity::default();

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(BACKDROP)
            .entity();

        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(380.0, 470.0)), Anchor::Center)
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(vec4(0.0, 0.0, 0.0, 0.7), vec2(0.0, 12.0), 36.0, 4.0)
            .entity();
        tree.in_parent(panel, |tree| {
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 32.0)),
                    Ab(vec2(320.0, 36.0)),
                    Anchor::TopCenter,
                )
                .with_text("PAUSED", 28.0)
                .text_center()
                .with_text_outline(ACCENT, 1.5)
                .color_raw::<UiBase>(WHITE)
                .entity();

            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 76.0)),
                    Ab(vec2(80.0, 2.0)),
                    Anchor::TopCenter,
                )
                .with_rect(0.0, 0.0, TRANSPARENT)
                .color_raw::<UiBase>(ACCENT)
                .entity();

            let column = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 108.0)),
                    Ab(vec2(MENU_BUTTON_SIZE.x, 308.0)),
                    Anchor::TopCenter,
                )
                .flow(FlowDirection::Vertical, 0.0, 12.0)
                .entity();
            tree.in_parent(column, |tree| {
                resume_button = menu_button::build(tree, "RESUME");
                restart_button = menu_button::build(tree, "RESTART");
                difficulty_button = menu_button::build(tree, "DIFFICULTY");
                main_menu_button = menu_button::build(tree, "MAIN MENU");
                quit_button = menu_button::build(tree, "QUIT TO DESKTOP");
            });

            difficulty_label = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 424.0)),
                    Ab(vec2(320.0, 24.0)),
                    Anchor::TopCenter,
                )
                .with_text("", 18.0)
                .text_center()
                .color_raw::<UiBase>(ACCENT_HOT)
                .entity();
        });
    });

    PauseHandles {
        root,
        resume_button,
        restart_button,
        difficulty_button,
        difficulty_label,
        main_menu_button,
        quit_button,
    }
}

pub fn handle_input(cobalt_world: &mut CobaltWorld, world: &mut World) {
    if !matches!(cobalt_world.resources.screen.current, Screen::Paused) {
        return;
    }
    let handles = cobalt_world.resources.ui_handles.pause;
    ui_set_text(
        world,
        handles.difficulty_label,
        &format!(
            "DIFFICULTY: {}",
            cobalt_world.resources.settings.difficulty.label()
        ),
    );

    let resume = handles.resume_button;
    let restart = handles.restart_button;
    let difficulty = handles.difficulty_button;
    let main_menu = handles.main_menu_button;
    let quit = handles.quit_button;
    let mut clicked_resume = false;
    let mut clicked_restart = false;
    let mut clicked_difficulty = false;
    let mut clicked_main_menu = false;
    let mut clicked_quit = false;
    for entity in ui_button_clicks(world) {
        if entity == resume {
            clicked_resume = true;
        } else if entity == restart {
            clicked_restart = true;
        } else if entity == difficulty {
            clicked_difficulty = true;
        } else if entity == main_menu {
            clicked_main_menu = true;
        } else if entity == quit {
            clicked_quit = true;
        }
    }
    if clicked_difficulty {
        let next = cobalt_world.resources.settings.difficulty.next();
        cobalt_world.resources.settings.difficulty = next;
        crate::settings::save(next);
    }
    if clicked_resume {
        lifecycle::enter(cobalt_world, world, Screen::InGame);
    } else if clicked_restart {
        game::restart_current(cobalt_world, world);
        lifecycle::enter(cobalt_world, world, Screen::InGame);
    } else if clicked_main_menu {
        game::start_at(cobalt_world, world, 0);
        lifecycle::enter(cobalt_world, world, Screen::Title);
    } else if clicked_quit {
        world.resources.window.should_exit = true;
    }
}
