//! First-person weapon viewmodel: a screen-space sprite of the held weapon
//! pinned to the bottom of the view. Each weapon has two poses — an angled
//! hip-fire sprite (the default) and an upright aim-down-sights sprite — uploaded
//! once as UI images. A single node swaps to the current weapon/pose and is offset
//! per frame for a walk bob and a recoil kick, sliding from the lower-right hip
//! position toward centre as you aim. Purely cosmetic — aim, crosshair, and hit
//! detection all stay at screen centre.

use crate::art;
use crate::ecs::CobaltWorld;
use crate::systems::world::player;
use nalgebra_glm::{Vec2, vec2};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::ui::components::UiNodeContent;
use nightshade::ecs::ui::layout_types::UiLayoutType;
use nightshade::prelude::*;

/// Display size of the gun sprite.
const VIEW_SIZE: Vec2 = Vec2::new(520.0, 400.0);
/// Tilt of the hip-fire pose relative to the upright aim pose.
const HIP_TILT: f32 = 0.55;

/// Upload each weapon's aim (upright) and hip (angled) viewmodel sprite as UI
/// images, caching their texture layer + UV sub-rect by weapon index.
pub fn load(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let sprites = [
        art::viewmodel_shotgun(),
        art::viewmodel_nailgun(),
        art::viewmodel_rocket(),
        art::viewmodel_railgun(),
        art::viewmodel_pistol(),
    ];
    let mut images = Vec::new();
    let mut hip_images = Vec::new();
    for sprite in sprites {
        images.push(upload(world, &sprite));
        hip_images.push(upload(world, &art::tilted(&sprite, HIP_TILT)));
    }
    cobalt_world.resources.viewmodel.images = images;
    cobalt_world.resources.viewmodel.hip_images = hip_images;
    cobalt_world.resources.viewmodel.shown = -1;
}

fn upload(world: &mut World, sprite: &art::Sprite) -> (u32, Vec2, Vec2) {
    match ui_upload_image(world, &sprite.rgba, sprite.width, sprite.height) {
        Some(upload) => (upload.layer, upload.uv_min, upload.uv_max),
        None => (0, vec2(0.0, 0.0), vec2(1.0, 1.0)),
    }
}

/// Build the single bottom-anchored viewmodel node (hidden until shown by screen).
pub fn build(tree: &mut UiTreeBuilder, hip_images: &[(u32, Vec2, Vec2)]) -> Entity {
    let (layer, uv_min, uv_max) =
        hip_images
            .first()
            .copied()
            .unwrap_or((0, vec2(0.0, 0.0), vec2(1.0, 1.0)));
    tree.add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, 10.0)),
            Ab(VIEW_SIZE),
            Anchor::BottomCenter,
        )
        .with_image_uv(layer, uv_min, uv_max)
        .without_pointer_events()
        .with_visible(false)
        .entity()
}

pub fn update(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let node = cobalt_world.resources.viewmodel.node;
    if cobalt_world.resources.viewmodel.images.is_empty() {
        return;
    }
    let delta = world.resources.window.timing.delta_time.clamp(0.001, 0.1);
    let weapon_index = cobalt_world.resources.weapon.current.index();
    let recoil = cobalt_world.resources.weapon.recoil;

    // Aim down sights on held right-mouse or gamepad left trigger.
    let mouse_aim = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::RIGHT_CLICKED);
    let pad_aim = query_active_gamepad(world)
        .map(|gamepad| gamepad.is_pressed(gilrs::Button::LeftTrigger2))
        .unwrap_or(false);
    let aiming = mouse_aim || pad_aim;

    cobalt_world.resources.viewmodel.aim += ((if aiming { 1.0 } else { 0.0 })
        - cobalt_world.resources.viewmodel.aim)
        * (delta * 14.0).min(1.0);
    let aim = cobalt_world.resources.viewmodel.aim.clamp(0.0, 1.0);

    // Swap the shown image when the weapon or pose changes.
    let use_ads = aim > 0.5;
    let key = weapon_index as i32 * 2 + i32::from(use_ads);
    if cobalt_world.resources.viewmodel.shown != key {
        let pose = if use_ads {
            &cobalt_world.resources.viewmodel.images
        } else {
            &cobalt_world.resources.viewmodel.hip_images
        };
        if let Some(&(layer, uv_min, uv_max)) = pose.get(weapon_index) {
            set_image(world, node, layer, uv_min, uv_max);
        }
        cobalt_world.resources.viewmodel.shown = key;
    }

    let position = player::position(cobalt_world, world);
    let viewmodel = &mut cobalt_world.resources.viewmodel;
    let mut moved = position - viewmodel.last_position;
    moved.y = 0.0;
    let speed = (moved.norm() / delta).min(20.0);
    viewmodel.last_position = position;
    let move_amount = (speed * 0.1).min(1.0);
    viewmodel.bob_phase += delta * (5.0 + speed * 0.6);
    let bob_x = viewmodel.bob_phase.sin() * 10.0 * move_amount;
    let bob_y = (viewmodel.bob_phase * 2.0).sin().abs() * 7.0 * move_amount;

    // Slide from the lower-right hip position toward the raised centre as we aim.
    let hip = vec2(150.0 + bob_x, 64.0 + bob_y);
    let ads = vec2(bob_x * 0.25, 8.0 + bob_y * 0.3 + recoil * 36.0);
    let offset = hip * (1.0 - aim) + ads * aim;
    set_position(world, node, Rl(vec2(50.0, 100.0)) + Ab(offset));
}

fn set_image(world: &mut World, entity: Entity, layer: u32, uv_min: Vec2, uv_max: Vec2) {
    if let Some(content) = world.ui.get_ui_node_content_mut(entity) {
        *content = UiNodeContent::Image {
            texture_index: layer,
            uv_min,
            uv_max,
        };
    }
}

fn set_position(world: &mut World, entity: Entity, position: UiValue<Vec2>) {
    if let Some(node) = world.ui.get_ui_layout_node_mut(entity)
        && let Some(UiLayoutType::Window(mut window)) = node.base_layout
    {
        window.position = position;
        node.base_layout = Some(UiLayoutType::Window(window));
    }
}
