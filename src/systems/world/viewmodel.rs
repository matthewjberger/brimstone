//! First-person weapon viewmodel: a screen-space sprite of the held
//! weapon pinned to the bottom of the view. Each weapon's art is uploaded once as
//! a UI image; a single bottom-centre node swaps to the current weapon and is
//! offset per frame for a walk bob and a recoil kick. Purely cosmetic — aim,
//! crosshair, and hit detection all stay at screen centre.

use crate::art;
use crate::ecs::CobaltWorld;
use crate::systems::world::player;
use nalgebra_glm::{Vec2, vec2};
use nightshade::ecs::ui::components::UiNodeContent;
use nightshade::ecs::ui::layout_types::UiLayoutType;
use nightshade::prelude::*;

/// Display size of the gun sprite (matches the 192x144 art aspect).
const VIEW_SIZE: Vec2 = Vec2::new(464.0, 348.0);

/// Upload each weapon's viewmodel sprite as a UI image and cache its texture
/// layer + UV sub-rect, indexed by [`crate::ecs::WeaponKind::index`].
pub fn load(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let sprites = [
        art::viewmodel_shotgun(),
        art::viewmodel_nailgun(),
        art::viewmodel_rocket(),
        art::viewmodel_railgun(),
        art::viewmodel_pistol(),
    ];
    let mut images = Vec::new();
    for sprite in sprites {
        match ui_upload_image(world, &sprite.rgba, sprite.width, sprite.height) {
            Some(upload) => images.push((upload.layer, upload.uv_min, upload.uv_max)),
            None => images.push((0, vec2(0.0, 0.0), vec2(1.0, 1.0))),
        }
    }
    cobalt_world.resources.viewmodel.images = images;
    cobalt_world.resources.viewmodel.shown = -1;
}

/// Build the single bottom-centre viewmodel node (hidden until shown by screen).
pub fn build(tree: &mut UiTreeBuilder, images: &[(u32, Vec2, Vec2)]) -> Entity {
    let (layer, uv_min, uv_max) =
        images
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

    if cobalt_world.resources.viewmodel.shown != weapon_index as i32 {
        if let Some(&(layer, uv_min, uv_max)) =
            cobalt_world.resources.viewmodel.images.get(weapon_index)
        {
            set_image(world, node, layer, uv_min, uv_max);
        }
        cobalt_world.resources.viewmodel.shown = weapon_index as i32;
    }

    let position = player::position(cobalt_world, world);
    let recoil = cobalt_world.resources.weapon.recoil;
    let viewmodel = &mut cobalt_world.resources.viewmodel;
    let mut moved = position - viewmodel.last_position;
    moved.y = 0.0;
    let speed = (moved.norm() / delta).min(20.0);
    viewmodel.last_position = position;
    let move_amount = (speed * 0.1).min(1.0);
    viewmodel.bob_phase += delta * (5.0 + speed * 0.6);
    let bob_x = viewmodel.bob_phase.sin() * 10.0 * move_amount;
    let bob_y = (viewmodel.bob_phase * 2.0).sin().abs() * 7.0 * move_amount;
    let offset = vec2(bob_x, 10.0 + bob_y + recoil * 36.0);
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
