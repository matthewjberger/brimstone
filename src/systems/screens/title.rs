use crate::ecs::{BoomerWorld, Screen, TitleHandles};
use crate::systems::lifecycle;
use crate::systems::screens::menu_button;
use crate::theme::*;
use nightshade::prelude::*;

const TITLE_TEXT: &str = "BOOMER";
const SUBTITLE_TEXT: &str = "SURVIVE THE HORDE";

pub fn build(tree: &mut UiTreeBuilder) -> TitleHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_intro(UiAnimationType::Fade, 0.4)
        .entity();

    let mut story_button = Entity::default();
    let mut play_button = Entity::default();
    let mut level_select_button = Entity::default();
    let mut editor_button = Entity::default();
    let mut quit_button = Entity::default();

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(VIGNETTE)
            .entity();

        tree.add_node()
            .window(
                Ab(vec2(0.0, 80.0)) + Rl(vec2(50.0, 0.0)),
                Ab(vec2(800.0, 80.0)),
                Anchor::TopCenter,
            )
            .with_text(TITLE_TEXT, 72.0)
            .text_center()
            .with_text_outline(ACCENT, 2.0)
            .color_raw::<UiBase>(WHITE)
            .entity();

        tree.add_node()
            .window(
                Ab(vec2(0.0, 176.0)) + Rl(vec2(50.0, 0.0)),
                Ab(vec2(600.0, 24.0)),
                Anchor::TopCenter,
            )
            .with_text(SUBTITLE_TEXT, 16.0)
            .text_center()
            .color_raw::<UiBase>(TEXT_DIM)
            .entity();

        let menu_column = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -72.0)),
                Ab(vec2(MENU_BUTTON_SIZE.x, 280.0)),
                Anchor::BottomCenter,
            )
            .flow(FlowDirection::Vertical, 8.0, 8.0)
            .entity();
        tree.in_parent(menu_column, |tree| {
            story_button = menu_button::build(tree, "STORY");
            play_button = menu_button::build(tree, "ARCADE");
            level_select_button = menu_button::build(tree, "SELECT LEVEL");
            editor_button = menu_button::build(tree, "LEVEL EDITOR");
            quit_button = menu_button::build(tree, "QUIT");
        });

        tree.add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -32.0)),
                Ab(vec2(760.0, 18.0)),
                Anchor::Center,
            )
            .with_text(
                "WASD  /  SHOOT LMB  /  DASH CTRL  /  STRAFE-JUMP & WALL-RUN SPACE  /  WEAPONS 1-3  /  ROCKET-JUMP DOWN+3  /  PAUSE ESC",
                12.0,
            )
            .text_center()
            .color_raw::<UiBase>(TEXT_FAINT)
            .entity();
    });

    TitleHandles {
        root,
        story_button,
        play_button,
        level_select_button,
        editor_button,
        quit_button,
    }
}

pub fn handle_input(boomer_world: &mut BoomerWorld, world: &mut World) {
    if !matches!(boomer_world.resources.screen.current, Screen::Title) {
        return;
    }
    let story = boomer_world.resources.ui_handles.title.story_button;
    let play = boomer_world.resources.ui_handles.title.play_button;
    let level_select = boomer_world.resources.ui_handles.title.level_select_button;
    let editor = boomer_world.resources.ui_handles.title.editor_button;
    let quit = boomer_world.resources.ui_handles.title.quit_button;
    let mut clicked_story = false;
    let mut clicked_play = false;
    let mut clicked_level_select = false;
    let mut clicked_editor = false;
    let mut clicked_quit = false;
    for entity in ui_button_clicks(world) {
        if entity == story {
            clicked_story = true;
        } else if entity == play {
            clicked_play = true;
        } else if entity == level_select {
            clicked_level_select = true;
        } else if entity == editor {
            clicked_editor = true;
        } else if entity == quit {
            clicked_quit = true;
        }
    }
    if clicked_story {
        crate::systems::story::open_select(boomer_world, world);
    } else if clicked_play {
        lifecycle::enter(boomer_world, world, Screen::InGame);
    } else if clicked_level_select {
        lifecycle::enter(boomer_world, world, Screen::LevelSelect);
    } else if clicked_editor {
        crate::systems::editor::open(boomer_world, world);
    } else if clicked_quit {
        world.resources.window.should_exit = true;
    }
}
