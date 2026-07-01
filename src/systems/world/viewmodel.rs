//! First-person weapon viewmodel: a small 3D model built from colored cube parts,
//! parented to the camera and held in the lower-right so you see the weapon from a
//! 3/4 side angle. Holding aim (right-mouse / gamepad left trigger) slides it to
//! centre and head-on. It bobs while walking and kicks on each shot. Purely
//! cosmetic — the crosshair, aim, and hit detection all stay at screen centre.

use crate::ecs::CobaltWorld;
use crate::systems::world::player;
use nalgebra_glm::{Vec3, lerp, quat_angle_axis, quat_identity, vec3};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::material::components::Material;
use nightshade::ecs::transform::components::Parent;
use nightshade::ecs::world::{
    GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, PARENT,
};
use nightshade::prelude::*;

const DARK: [f32; 3] = [0.13, 0.14, 0.17];
const METAL: [f32; 3] = [0.42, 0.44, 0.5];
const GRIP: [f32; 3] = [0.28, 0.2, 0.13];

const HIP_POS: Vec3 = Vec3::new(0.24, -0.22, -0.62);
const ADS_POS: Vec3 = Vec3::new(0.0, -0.17, -0.42);
/// Distance ahead of the camera the barrel converges on, so hip and ADS both
/// point at the crosshair.
const CONVERGE_DIST: f32 = 8.0;
/// Duration of the lower-then-raise weapon-swap animation.
const SWITCH_TIME: f32 = 0.34;

/// One cube part of a weapon model: local offset, size, colour, and how brightly
/// it glows (0 = matte metal, >0 = emissive accent).
struct Part {
    local: Vec3,
    scale: Vec3,
    color: [f32; 3],
    glow: f32,
}

const fn matte(local: Vec3, scale: Vec3, color: [f32; 3]) -> Part {
    Part {
        local,
        scale,
        color,
        glow: 0.0,
    }
}

const fn glow(local: Vec3, scale: Vec3, color: [f32; 3]) -> Part {
    Part {
        local,
        scale,
        color,
        glow: 4.0,
    }
}

fn model(index: usize) -> Vec<Part> {
    match index {
        // Shotgun: twin barrels, receiver, pump, stock.
        0 => vec![
            matte(vec3(-0.022, 0.02, -0.16), vec3(0.028, 0.03, 0.42), DARK),
            matte(vec3(0.022, 0.02, -0.16), vec3(0.028, 0.03, 0.42), DARK),
            matte(vec3(0.0, 0.0, 0.06), vec3(0.085, 0.08, 0.16), METAL),
            matte(vec3(0.0, -0.05, -0.12), vec3(0.075, 0.045, 0.1), DARK),
            matte(vec3(0.0, -0.03, 0.2), vec3(0.05, 0.08, 0.12), GRIP),
            matte(vec3(0.0, -0.1, 0.05), vec3(0.045, 0.09, 0.05), GRIP),
            glow(
                vec3(0.0, 0.055, 0.06),
                vec3(0.07, 0.016, 0.12),
                [0.95, 0.55, 0.2],
            ),
        ],
        // Nailgun: boxy body, barrel bundle, drum mag.
        1 => vec![
            matte(vec3(0.0, 0.0, -0.02), vec3(0.09, 0.09, 0.28), METAL),
            matte(vec3(0.0, 0.01, -0.2), vec3(0.06, 0.06, 0.12), DARK),
            matte(vec3(0.0, -0.06, 0.03), vec3(0.07, 0.09, 0.07), DARK),
            matte(vec3(0.0, -0.12, 0.07), vec3(0.05, 0.1, 0.05), GRIP),
            glow(
                vec3(0.0, 0.05, -0.05),
                vec3(0.07, 0.02, 0.16),
                [0.25, 0.8, 0.9],
            ),
        ],
        // Rocket launcher: fat tube, wide muzzle, sight.
        2 => vec![
            matte(vec3(0.0, 0.01, -0.14), vec3(0.1, 0.1, 0.44), DARK),
            matte(vec3(0.0, 0.01, -0.34), vec3(0.12, 0.12, 0.05), METAL),
            matte(vec3(0.0, -0.04, 0.1), vec3(0.08, 0.08, 0.14), METAL),
            matte(vec3(0.0, -0.12, 0.08), vec3(0.05, 0.1, 0.05), GRIP),
            matte(vec3(0.0, 0.08, -0.05), vec3(0.02, 0.04, 0.05), DARK),
            glow(
                vec3(0.0, 0.01, -0.35),
                vec3(0.07, 0.07, 0.02),
                [0.35, 0.6, 1.0],
            ),
        ],
        // Railgun: long barrel, receiver, glowing coils, stock.
        3 => vec![
            matte(vec3(0.0, 0.02, -0.18), vec3(0.045, 0.05, 0.5), DARK),
            matte(vec3(0.0, 0.0, 0.08), vec3(0.07, 0.08, 0.18), METAL),
            matte(vec3(0.0, -0.03, 0.22), vec3(0.05, 0.09, 0.12), GRIP),
            matte(vec3(0.0, -0.09, 0.08), vec3(0.045, 0.09, 0.05), GRIP),
            glow(
                vec3(0.0, 0.02, -0.1),
                vec3(0.06, 0.06, 0.03),
                [0.7, 0.35, 0.95],
            ),
            glow(
                vec3(0.0, 0.02, -0.24),
                vec3(0.06, 0.06, 0.03),
                [0.7, 0.35, 0.95],
            ),
        ],
        // Pistol: slide, short barrel, angled grip.
        4 => vec![
            matte(vec3(0.0, 0.0, -0.02), vec3(0.05, 0.06, 0.2), DARK),
            matte(vec3(0.0, 0.01, -0.16), vec3(0.03, 0.035, 0.08), METAL),
            matte(vec3(0.0, -0.09, 0.05), vec3(0.05, 0.13, 0.055), GRIP),
            glow(
                vec3(0.0, 0.04, -0.14),
                vec3(0.02, 0.02, 0.03),
                [0.95, 0.82, 0.35],
            ),
        ],
        // Tesla cannon: coil body, prongs, and a glowing arc emitter.
        _ => vec![
            matte(vec3(0.0, 0.0, -0.02), vec3(0.08, 0.09, 0.22), METAL),
            matte(vec3(0.0, 0.02, -0.2), vec3(0.05, 0.05, 0.1), DARK),
            matte(vec3(0.0, -0.11, 0.07), vec3(0.05, 0.11, 0.05), GRIP),
            glow(
                vec3(0.0, 0.06, -0.05),
                vec3(0.09, 0.02, 0.14),
                [0.45, 0.85, 2.6],
            ),
            glow(
                vec3(0.03, 0.03, -0.28),
                vec3(0.02, 0.02, 0.06),
                [0.5, 0.95, 2.8],
            ),
            glow(
                vec3(-0.03, 0.03, -0.28),
                vec3(0.02, 0.02, 0.06),
                [0.5, 0.95, 2.8],
            ),
        ],
    }
}

/// Spawn the camera-parented root and every weapon's cube model (hidden).
pub fn spawn(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let Some(camera) = cobalt_world.resources.player.camera_entity else {
        return;
    };
    let root = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | PARENT,
        1,
    )[0];
    world.core.set_name(root, Name("Viewmodel".to_string()));
    world.core.set_parent(root, Parent(Some(camera)));
    world.core.set_local_transform(
        root,
        LocalTransform {
            translation: HIP_POS,
            rotation: quat_identity(),
            scale: vec3(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_global_transform(root, GlobalTransform::default());
    world.resources.transform_state.children_cache_valid = false;
    mark_local_transform_dirty(world, root);

    let models: Vec<Vec<Entity>> = (0..crate::ecs::WeaponKind::ALL.len())
        .map(|index| build(world, root, index))
        .collect();
    for group in &models {
        for entity in group {
            world
                .core
                .set_visibility(*entity, Visibility { visible: false });
        }
    }
    cobalt_world.resources.viewmodel.root = root;
    cobalt_world.resources.viewmodel.models = models;
    cobalt_world.resources.viewmodel.shown = -1;
}

fn build(world: &mut World, root: Entity, index: usize) -> Vec<Entity> {
    model(index)
        .into_iter()
        .map(|part| {
            let entity = spawn_cube_at(world, Vec3::zeros());
            world.core.add_components(entity, PARENT);
            world.core.set_parent(entity, Parent(Some(root)));
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation = part.local;
                transform.scale = part.scale;
            }
            mark_local_transform_dirty(world, entity);
            spawn_material(
                world,
                entity,
                format!("viewmodel_{}", entity.id),
                material(part.color, part.glow),
            );
            entity
        })
        .collect()
}

fn material(color: [f32; 3], glow: f32) -> Material {
    Material {
        base_color: [color[0], color[1], color[2], 1.0],
        emissive_factor: [color[0] * glow, color[1] * glow, color[2] * glow],
        emissive_strength: glow,
        roughness: 0.42,
        metallic: if glow > 0.0 { 0.0 } else { 0.7 },
        ..Default::default()
    }
}

/// World-space barrel tip of the held weapon for the current aim state, given
/// the camera frame. Effects anchored here (the tesla arc) line up with the
/// visible model whether it's held at the hip or slid to centre in ADS.
pub fn muzzle(
    cobalt_world: &CobaltWorld,
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
) -> Vec3 {
    let aim = cobalt_world.resources.viewmodel.aim.clamp(0.0, 1.0);
    let base = lerp(&HIP_POS, &ADS_POS, aim);
    let barrel = (vec3(0.0, 0.0, -CONVERGE_DIST) - base).normalize();
    let tip = base + barrel * 0.4;
    origin + right * tip.x + up * tip.y - forward * tip.z
}

/// Show only the active weapon (also used to hide everything off the game screen).
pub fn set_active(cobalt_world: &CobaltWorld, world: &mut World, active: i32) {
    for (index, group) in cobalt_world.resources.viewmodel.models.iter().enumerate() {
        let visible = index as i32 == active;
        for entity in group {
            world.core.set_visibility(*entity, Visibility { visible });
        }
    }
}

pub fn update(cobalt_world: &mut CobaltWorld, world: &mut World) {
    if cobalt_world.resources.viewmodel.models.is_empty() {
        return;
    }
    let delta = world.resources.window.timing.delta_time.clamp(0.001, 0.1);
    let weapon_index = cobalt_world.resources.weapon.current.index() as i32;
    let recoil = cobalt_world.resources.weapon.recoil;

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

    // Weapon-swap animation: lower the held weapon, swap models at the bottom of
    // the dip, then raise the new one. The first equip snaps in with no dip.
    if cobalt_world.resources.viewmodel.shown < 0 {
        set_active(cobalt_world, world, weapon_index);
        cobalt_world.resources.viewmodel.shown = weapon_index;
    } else if weapon_index != cobalt_world.resources.viewmodel.shown
        && cobalt_world.resources.viewmodel.switch <= 0.0
    {
        cobalt_world.resources.viewmodel.switch = SWITCH_TIME;
    }
    cobalt_world.resources.viewmodel.switch =
        (cobalt_world.resources.viewmodel.switch - delta).max(0.0);
    let switch = cobalt_world.resources.viewmodel.switch;
    let phase = 1.0 - switch / SWITCH_TIME;
    if switch > 0.0 && phase >= 0.5 && cobalt_world.resources.viewmodel.shown != weapon_index {
        set_active(cobalt_world, world, weapon_index);
        cobalt_world.resources.viewmodel.shown = weapon_index;
    }
    let dip = if switch > 0.0 {
        (phase * std::f32::consts::PI).sin()
    } else {
        0.0
    };

    let position = player::position(cobalt_world, world);
    let root = cobalt_world.resources.viewmodel.root;
    let viewmodel = &mut cobalt_world.resources.viewmodel;
    let mut moved = position - viewmodel.last_position;
    moved.y = 0.0;
    let speed = (moved.norm() / delta).min(20.0);
    viewmodel.last_position = position;
    let move_amount = (speed * 0.1).min(1.0);
    viewmodel.bob_phase += delta * (5.0 + speed * 0.6);
    let bob = vec3(
        viewmodel.bob_phase.sin() * 0.006 * move_amount,
        (viewmodel.bob_phase * 2.0).sin().abs() * 0.006 * move_amount,
        0.0,
    );

    // Slide from the lower-right hip hold to centred head-on as we aim; recoil
    // kicks the model back toward the camera and pitches it up.
    let base = lerp(&HIP_POS, &ADS_POS, aim);
    let kick = vec3(0.0, recoil * 0.01, recoil * 0.06);
    // Dip the weapon down and slightly back during a swap.
    let drop = vec3(0.0, -dip * 0.34, dip * 0.14);
    // Aim the barrel (local -Z) at a point straight ahead, so hip-fire points at
    // the crosshair just like ADS does.
    let aim_dir = vec3(0.0, 0.0, -CONVERGE_DIST) - base;
    let horizontal = (aim_dir.x * aim_dir.x + aim_dir.z * aim_dir.z)
        .sqrt()
        .max(1e-4);
    let yaw = (-aim_dir.x).atan2(-aim_dir.z);
    let pitch = aim_dir.y.atan2(horizontal) + recoil * 0.25 - dip * 0.3;
    if let Some(transform) = world.core.get_local_transform_mut(root) {
        transform.translation = base + bob + kick + drop;
        transform.rotation = quat_angle_axis(yaw, &vec3(0.0, 1.0, 0.0))
            * quat_angle_axis(pitch, &vec3(1.0, 0.0, 0.0));
    }
    mark_local_transform_dirty(world, root);
}
