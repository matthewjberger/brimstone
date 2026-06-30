//! Story mission picker: replay or continue any unlocked mission. Progress is
//! tracked in [`crate::ecs::StoryState`] and persisted to disk by the director.

use crate::campaign;
use crate::ecs::{BoomerWorld, MissionSelectHandles, Screen};
use crate::systems::lifecycle;
use crate::systems::screens::menu_button;
use crate::systems::story;
use crate::theme::*;
use nightshade::prelude::*;

const BUTTON_GAP: f32 = 10.0;

pub fn build(tree: &mut UiTreeBuilder) -> MissionSelectHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_intro(UiAnimationType::Fade, 0.18)
        .with_visible(false)
        .entity();

    let mut mission_buttons = Vec::new();
    let mut back_button = Entity::default();

    let rows = campaign::count() as f32 + 1.0;
    let column_height = rows * MENU_BUTTON_HEIGHT + (rows - 1.0) * BUTTON_GAP;
    let panel_height = column_height + 156.0;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(BACKDROP)
            .entity();

        let panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(440.0, panel_height)),
                Anchor::Center,
            )
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(vec4(0.0, 0.0, 0.0, 0.7), vec2(0.0, 12.0), 36.0, 4.0)
            .entity();
        tree.in_parent(panel, |tree| {
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 32.0)),
                    Ab(vec2(400.0, 36.0)),
                    Anchor::TopCenter,
                )
                .with_text("STORY MISSIONS", 28.0)
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
                    Ab(vec2(MENU_BUTTON_SIZE.x, column_height)),
                    Anchor::TopCenter,
                )
                .flow(FlowDirection::Vertical, 0.0, BUTTON_GAP)
                .entity();
            tree.in_parent(column, |tree| {
                for index in 0..campaign::count() {
                    let label = format!("{}   {}", index + 1, campaign::mission(index).title);
                    mission_buttons.push(menu_button::build(tree, &label));
                }
                back_button = menu_button::build(tree, "BACK");
            });
        });
    });

    MissionSelectHandles {
        root,
        mission_buttons,
        back_button,
    }
}

pub fn handle_input(boomer_world: &mut BoomerWorld, world: &mut World) {
    if !matches!(boomer_world.resources.screen.current, Screen::MissionSelect) {
        return;
    }
    let back = boomer_world.resources.ui_handles.mission_select.back_button;
    let mut selected = None;
    let mut clicked_back = false;
    for entity in ui_button_clicks(world) {
        if entity == back {
            clicked_back = true;
        } else if let Some(index) = boomer_world
            .resources
            .ui_handles
            .mission_select
            .mission_buttons
            .iter()
            .position(|button| *button == entity)
        {
            selected = Some(index);
        }
    }
    if let Some(index) = selected {
        if index <= boomer_world.resources.story.unlocked {
            story::launch_mission(boomer_world, world, index);
        }
    } else if clicked_back {
        lifecycle::enter(boomer_world, world, Screen::Title);
    }
}
