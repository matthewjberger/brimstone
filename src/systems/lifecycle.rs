use crate::ecs::{BrimstoneWorld, Screen, UiHandles};
use crate::systems::screens::{cutscene, hud, level_select, mission_select, pause, title};
use crate::systems::world::{audio, game, player, textures, viewmodel};
use nightshade::ecs::graphics::resources::ColorGradingPreset;
use nightshade::prelude::*;

pub fn initialize(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;

    brimstone_world.resources.settings.difficulty = crate::settings::load();
    brimstone_world.resources.settings.loaded = true;

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
    player::spawn(brimstone_world, world);
    viewmodel::spawn(brimstone_world, world);
    game::start_at(brimstone_world, world, 0);

    let mut tree = UiTreeBuilder::new(world);
    let title_handles = title::build(&mut tree);
    let level_select_handles = level_select::build(&mut tree);
    let pause_handles = pause::build(&mut tree);
    let hud_handles = hud::build(&mut tree);
    let editor_handles = crate::systems::editor::build_ui(&mut tree);
    let cutscene_handles = cutscene::build(&mut tree);
    let mission_select_handles = mission_select::build(&mut tree);
    let adventure_handles = crate::adventure::build_ui(&mut tree);
    tree.finish();
    brimstone_world.resources.ui_handles.title = title_handles;
    brimstone_world.resources.ui_handles.level_select = level_select_handles;
    brimstone_world.resources.ui_handles.mission_select = mission_select_handles;
    brimstone_world.resources.ui_handles.pause = pause_handles;
    brimstone_world.resources.ui_handles.hud = hud_handles;
    brimstone_world.resources.ui_handles.editor = editor_handles;
    brimstone_world.resources.ui_handles.cutscene = cutscene_handles;
    brimstone_world.resources.ui_handles.adventure = adventure_handles;

    enter(brimstone_world, world, Screen::Title);
}

pub fn enter(brimstone_world: &mut BrimstoneWorld, world: &mut World, screen: Screen) {
    let config = screen_config(&brimstone_world.resources.ui_handles, screen);

    brimstone_world.resources.screen.current = screen;
    apply_visibility(brimstone_world, world);
    if !matches!(screen, Screen::InGame | Screen::Paused | Screen::Adventure) {
        viewmodel::set_active(brimstone_world, world, -1);
    }

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
        Screen::Adventure => ScreenConfig {
            physics_enabled: true,
            cursor_locked: true,
            gamepad_nav: false,
            focus: None,
        },
    }
}

fn apply_visibility(brimstone_world: &BrimstoneWorld, world: &mut World) {
    let handles = &brimstone_world.resources.ui_handles;
    let screen = brimstone_world.resources.screen.current;
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
    ui_set_visible(
        world,
        handles.adventure.root,
        matches!(screen, Screen::Adventure),
    );
}
