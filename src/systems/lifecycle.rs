use crate::ecs::{CobaltWorld, Screen, UiHandles};
use crate::systems::screens::{cutscene, hud, level_select, mission_select, pause, title};
use crate::systems::world::{audio, game, player, textures};
use nightshade::ecs::graphics::resources::ColorGradingPreset;
use nightshade::prelude::*;

pub fn initialize(cobalt_world: &mut CobaltWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;

    cobalt_world.resources.settings.difficulty = crate::settings::load();
    cobalt_world.resources.settings.loaded = true;

    // Snappier-than-earth gravity for arcade-FPS feel; every vertical impulse in
    // tuning is derived from this. Only the player is a physics body.
    world.resources.physics.gravity = nalgebra_glm::vec3(0.0, crate::tuning::GRAVITY, 0.0);

    let settings = &mut world.resources.render_settings;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.35;
    settings.bloom_threshold = 1.1;
    settings.color_grading = ColorGradingPreset::Vibrant.to_color_grading();
    settings.ambient_light = [0.32, 0.31, 0.38, 1.0];

    textures::load(world);
    audio::load(world);
    player::spawn(cobalt_world, world);
    game::start_at(cobalt_world, world, 0);

    let mut tree = UiTreeBuilder::new(world);
    let title_handles = title::build(&mut tree);
    let level_select_handles = level_select::build(&mut tree);
    let pause_handles = pause::build(&mut tree);
    let hud_handles = hud::build(&mut tree);
    let editor_handles = crate::systems::editor::build_ui(&mut tree);
    let cutscene_handles = cutscene::build(&mut tree);
    let mission_select_handles = mission_select::build(&mut tree);
    tree.finish();
    cobalt_world.resources.ui_handles.title = title_handles;
    cobalt_world.resources.ui_handles.level_select = level_select_handles;
    cobalt_world.resources.ui_handles.mission_select = mission_select_handles;
    cobalt_world.resources.ui_handles.pause = pause_handles;
    cobalt_world.resources.ui_handles.hud = hud_handles;
    cobalt_world.resources.ui_handles.editor = editor_handles;
    cobalt_world.resources.ui_handles.cutscene = cutscene_handles;

    enter(cobalt_world, world, Screen::Title);
}

pub fn enter(cobalt_world: &mut CobaltWorld, world: &mut World, screen: Screen) {
    let config = screen_config(&cobalt_world.resources.ui_handles, screen);

    cobalt_world.resources.screen.current = screen;
    apply_visibility(cobalt_world, world);

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
            focus: Some(handles.title.story_button),
        },
        Screen::LevelSelect => ScreenConfig {
            physics_enabled: false,
            cursor_locked: false,
            gamepad_nav: true,
            focus: handles.level_select.level_buttons.first().copied(),
        },
        Screen::MissionSelect => ScreenConfig {
            physics_enabled: false,
            cursor_locked: false,
            gamepad_nav: true,
            focus: handles.mission_select.mission_buttons.first().copied(),
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
        Screen::Editor => ScreenConfig {
            physics_enabled: false,
            cursor_locked: true,
            gamepad_nav: false,
            focus: None,
        },
        Screen::Cutscene => ScreenConfig {
            physics_enabled: false,
            cursor_locked: false,
            gamepad_nav: false,
            focus: None,
        },
    }
}

fn apply_visibility(cobalt_world: &CobaltWorld, world: &mut World) {
    let handles = &cobalt_world.resources.ui_handles;
    let screen = cobalt_world.resources.screen.current;
    ui_set_visible(world, handles.title.root, matches!(screen, Screen::Title));
    ui_set_visible(
        world,
        handles.level_select.root,
        matches!(screen, Screen::LevelSelect),
    );
    ui_set_visible(
        world,
        handles.mission_select.root,
        matches!(screen, Screen::MissionSelect),
    );
    ui_set_visible(world, handles.pause.root, matches!(screen, Screen::Paused));
    ui_set_visible(
        world,
        handles.hud.root,
        matches!(screen, Screen::InGame | Screen::Paused),
    );
    ui_set_visible(world, handles.editor.root, matches!(screen, Screen::Editor));
    ui_set_visible(
        world,
        handles.cutscene.root,
        matches!(screen, Screen::Cutscene),
    );
}
