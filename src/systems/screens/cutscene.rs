//! Full-screen narrative cutscene: a title, a body of story text, and a prompt
//! to continue. Advancing is driven by the story director.

use crate::ecs::{BrimstoneWorld, CutsceneHandles, Screen};
use crate::systems::story;
use crate::theme::*;
use nightshade::prelude::*;

pub fn build(tree: &mut UiTreeBuilder) -> CutsceneHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();

    let mut title = Entity::default();
    let mut body = Entity::default();
    let mut hint = Entity::default();

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(vec4(0.02, 0.02, 0.05, 0.94))
            .entity();

        title = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 120.0)),
                Ab(vec2(1000.0, 60.0)),
                Anchor::TopCenter,
            )
            .with_text("", 44.0)
            .text_center()
            .with_text_outline(ACCENT, 2.0)
            .color_raw::<UiBase>(WHITE)
            .entity();

        body = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 42.0)),
                Ab(vec2(1080.0, 320.0)),
                Anchor::Center,
            )
            .with_text("", 26.0)
            .text_center()
            .color_raw::<UiBase>(TEXT_DIM)
            .entity();

        hint = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -64.0)),
                Ab(vec2(760.0, 28.0)),
                Anchor::BottomCenter,
            )
            .with_text("PRESS SPACE / (A) TO CONTINUE", 18.0)
            .text_center()
            .color_raw::<UiBase>(TEXT_FAINT)
            .entity();
    });

    CutsceneHandles {
        root,
        title,
        body,
        hint,
    }
}

const REVEAL_CHARS_PER_SECOND: f32 = 48.0;

pub fn update(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if !matches!(brimstone_world.resources.screen.current, Screen::Cutscene) {
        return;
    }
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    brimstone_world.resources.story.reveal += delta * REVEAL_CHARS_PER_SECOND;
    let handles = brimstone_world.resources.ui_handles.cutscene;
    let story = &brimstone_world.resources.story;
    if let Some(slide) = story.slides.get(story.slide_index) {
        let revealed = (story.reveal as usize).min(slide.body.chars().count());
        let shown: String = slide.body.chars().take(revealed).collect();
        ui_set_text(world, handles.title, &slide.title);
        ui_set_text(world, handles.body, &shown);
    }
}

pub fn handle_input(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if !matches!(brimstone_world.resources.screen.current, Screen::Cutscene) {
        return;
    }
    let keyboard = &world.resources.input.keyboard;
    let pressed = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || world
            .resources
            .input
            .gamepad
            .just_pressed_buttons
            .contains(&gilrs::Button::South);
    if !pressed {
        return;
    }
    // First press finishes the typewriter; the next advances the cutscene.
    let story = &brimstone_world.resources.story;
    let full = story
        .slides
        .get(story.slide_index)
        .map(|slide| slide.body.chars().count())
        .unwrap_or(0);
    if (story.reveal as usize) < full {
        brimstone_world.resources.story.reveal = full as f32;
    } else {
        story::advance(brimstone_world, world);
    }
}
