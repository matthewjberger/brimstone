use crate::ecs::{BoomerWorld, Screen, UiHandles};
use crate::systems::screens::{hud, pause, title};
use crate::systems::world::{audio, game, level, player, textures};
use nightshade::ecs::graphics::resources::{ColorGradingPreset, Fog};
use nightshade::prelude::*;

pub fn initialize(boomer_world: &mut BoomerWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;

    let settings = &mut world.resources.render_settings;
    settings.atmosphere = Atmosphere::Nebula;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.9;
    settings.bloom_threshold = 0.7;
    settings.color_grading = ColorGradingPreset::Vibrant.to_color_grading();
    settings.ambient_light = [0.06, 0.05, 0.09, 1.0];
    settings.fog = Some(Fog {
        color: [0.05, 0.02, 0.10],
        start: 16.0,
        end: 58.0,
    });
    capture_procedural_atmosphere_ibl(world, Atmosphere::Nebula, 0.0);

    textures::load(world);
    audio::load(world);
    level::build(world);
    player::spawn(boomer_world, world);
    game::start(boomer_world, world);

    let mut tree = UiTreeBuilder::new(world);
    let title_handles = title::build(&mut tree);
    let pause_handles = pause::build(&mut tree);
    let hud_handles = hud::build(&mut tree);
    tree.finish();
    boomer_world.resources.ui_handles.title = title_handles;
    boomer_world.resources.ui_handles.pause = pause_handles;
    boomer_world.resources.ui_handles.hud = hud_handles;

    enter(boomer_world, world, Screen::Title);
}

pub fn enter(boomer_world: &mut BoomerWorld, world: &mut World, screen: Screen) {
    let config = screen_config(&boomer_world.resources.ui_handles, screen);

    boomer_world.resources.screen.current = screen;
    apply_visibility(boomer_world, world);

    world.resources.physics.enabled = config.physics_enabled;
    set_cursor_locked(world, config.cursor_locked);
    set_cursor_visible(world, !config.cursor_locked);
    world.resources.retained_ui.gamepad_nav.enabled = config.gamepad_nav;

    let interaction = world.resources.retained_ui.interaction_for_active_mut();
    interaction.focused_entity = config.focus;
    world.resources.retained_ui.overlays.focus_ring_visible = config.focus.is_some();
    if config.focus.is_none() {
        world.resources.retained_ui.gamepad_nav.held_direction = None;
    }
}

struct ScreenConfig {
    physics_enabled: bool,
    cursor_locked: bool,
    gamepad_nav: bool,
    focus: Option<Entity>,
}

fn screen_config(handles: &UiHandles, screen: Screen) -> ScreenConfig {
    match screen {
        Screen::Title => ScreenConfig {
            physics_enabled: false,
            cursor_locked: false,
            gamepad_nav: true,
            focus: Some(handles.title.play_button),
        },
        Screen::Paused => ScreenConfig {
            physics_enabled: false,
            cursor_locked: false,
            gamepad_nav: true,
            focus: Some(handles.pause.resume_button),
        },
        Screen::InGame => ScreenConfig {
            physics_enabled: true,
            cursor_locked: true,
            gamepad_nav: false,
            focus: None,
        },
    }
}

fn apply_visibility(boomer_world: &BoomerWorld, world: &mut World) {
    let handles = &boomer_world.resources.ui_handles;
    let screen = boomer_world.resources.screen.current;
    ui_set_visible(world, handles.title.root, matches!(screen, Screen::Title));
    ui_set_visible(world, handles.pause.root, matches!(screen, Screen::Paused));
    ui_set_visible(
        world,
        handles.hud.root,
        matches!(screen, Screen::InGame | Screen::Paused),
    );
}
