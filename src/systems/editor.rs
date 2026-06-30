//! In-game level editor. Fly around, paint blocks onto a grid, drop the player
//! start / exit / jump pads / enemy spawns, then play-test instantly. Levels
//! save to and load from a file on disk so they persist between sessions.

use crate::content::{self, BlockKind, LevelData};
use crate::ecs::{CobaltWorld, EditorHandles, Screen};
use crate::systems::lifecycle;
use crate::systems::world::level::material_for;
use crate::systems::world::{game, level, player, textures};
use crate::theme::*;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::camera::systems::first_person_camera_look_system;
use nightshade::ecs::physics::commands::spawn_static_physics_cube_with_material;
use nightshade::ecs::world::commands::{spawn_light_entity, spawn_mesh_at};
use nightshade::prelude::*;

const SAVE_PATH: &str = "cobalt_custom_level.txt";
const FLY_SPEED: f32 = 16.0;
const GRID: f32 = 1.0;
const VANTAGE: Vec3 = Vec3::new(0.0, 9.0, 20.0);

pub fn brush_size(kind: BlockKind) -> Vec3 {
    match kind {
        BlockKind::Wall => vec3(4.0, 4.0, 1.0),
        BlockKind::Pillar => vec3(1.6, 5.0, 1.6),
        BlockKind::Platform => vec3(4.0, 2.0, 4.0),
        BlockKind::Cover => vec3(3.0, 1.0, 1.4),
        BlockKind::Choke => vec3(2.0, 1.0, 2.0),
        BlockKind::Monument => vec3(3.0, 6.0, 3.0),
        BlockKind::Core => vec3(3.0, 4.0, 3.0),
    }
}

/// Open the editor: tear down whatever level is showing, load the saved draft,
/// and stand the camera up at the vantage point.
pub fn open(cobalt_world: &mut CobaltWorld, world: &mut World) {
    game::teardown_world(cobalt_world, world);
    let empty = cobalt_world.resources.editor.data.blocks.is_empty()
        && cobalt_world.resources.editor.data.spawn_points.is_empty();
    if empty {
        if let Some(data) = load_from_disk() {
            cobalt_world.resources.editor.data = data;
        }
        cobalt_world.resources.editor.brush = BlockKind::Platform;
        cobalt_world.resources.editor.place_height = 0.0;
    }
    cobalt_world.resources.editor.active = true;
    rebuild_scene(cobalt_world, world);
    player::teleport(cobalt_world, world, VANTAGE);
    lifecycle::enter(cobalt_world, world, Screen::Editor);
    status(cobalt_world, "EDITOR");
}

/// Despawn every preview entity. Called when leaving the editor or play-testing.
pub fn teardown(cobalt_world: &mut CobaltWorld, world: &mut World) {
    for entity in cobalt_world.resources.editor.block_entities.drain(..) {
        despawn_recursive_immediate(world, entity);
    }
    for entity in cobalt_world.resources.editor.markers.drain(..) {
        despawn_recursive_immediate(world, entity);
    }
    if let Some(ghost) = cobalt_world.resources.editor.ghost.take() {
        despawn_recursive_immediate(world, ghost);
    }
}

fn rebuild_scene(cobalt_world: &mut CobaltWorld, world: &mut World) {
    teardown(cobalt_world, world);
    let data = cobalt_world.resources.editor.data.clone();

    level::apply_environment(
        world,
        content::atmosphere_for(data.atmosphere_index),
        data.fog,
    );

    let mut blocks: Vec<Entity> = Vec::new();
    for block in &data.blocks {
        blocks.push(spawn_block_entity(world, block));
    }
    cobalt_world.resources.editor.block_entities = blocks;

    rebuild_markers(cobalt_world, world);

    let ghost = spawn_mesh_at(world, "Cube", VANTAGE, vec3(1.0, 1.0, 1.0));
    world
        .core
        .set_material_ref(ghost, MaterialRef::new(textures::MAT_GHOST.to_string()));
    cobalt_world.resources.editor.ghost = Some(ghost);
}

fn rebuild_markers(cobalt_world: &mut CobaltWorld, world: &mut World) {
    for entity in cobalt_world.resources.editor.markers.drain(..) {
        despawn_recursive_immediate(world, entity);
    }
    let data = cobalt_world.resources.editor.data.clone();
    let mut markers: Vec<Entity> = Vec::new();

    let span = 38.0;
    markers.push(named_block(
        world,
        "Floor",
        vec3(0.0, -0.5, 0.0),
        vec3(span, 1.0, span),
        textures::floor_material(),
    ));
    let edge = 19.5;
    let walls = [
        (vec3(0.0, 4.0, -edge), vec3(span + 2.0, 8.0, 1.0)),
        (vec3(0.0, 4.0, edge), vec3(span + 2.0, 8.0, 1.0)),
        (vec3(-edge, 4.0, 0.0), vec3(1.0, 8.0, span + 2.0)),
        (vec3(edge, 4.0, 0.0), vec3(1.0, 8.0, span + 2.0)),
    ];
    for (center, size) in walls {
        markers.push(named_block(
            world,
            "Wall",
            center,
            size,
            textures::wall_material(),
        ));
    }
    markers.push(spawn_lamp(
        world,
        vec3(0.0, 9.0, 0.0),
        vec3(0.55, 0.55, 0.85),
    ));

    for (x, z) in &data.pads {
        markers.push(marker_mesh(
            world,
            vec3(*x, 0.12, *z),
            vec3(2.4, 0.24, 2.4),
            textures::PAD_MATERIAL,
        ));
    }
    for (x, z) in &data.spawn_points {
        markers.push(marker_mesh(
            world,
            vec3(*x, 0.6, *z),
            vec3(0.8, 1.2, 0.8),
            textures::MARKER_ENEMY,
        ));
    }
    markers.push(marker_mesh(
        world,
        vec3(data.spawn[0], 0.9, data.spawn[2]),
        vec3(0.9, 1.8, 0.9),
        textures::MARKER_PLAYER,
    ));
    markers.push(marker_mesh(
        world,
        vec3(data.exit[0], 2.4, data.exit[1]),
        vec3(2.6, 4.8, 0.5),
        textures::MAT_EXIT,
    ));

    cobalt_world.resources.editor.markers = markers;
}

pub fn update(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    cobalt_world.resources.editor.status = (cobalt_world.resources.editor.status - delta).max(0.0);

    first_person_camera_look_system(world);
    fly(cobalt_world, world, delta);

    let cursor = cursor_on_grid(cobalt_world, world);
    cobalt_world.resources.editor.cursor = cursor;
    update_ghost(cobalt_world, world, cursor);

    handle_input(cobalt_world, world, cursor);
    refresh_status_text(cobalt_world, world);
}

const CONTROLS: &str = "LEVEL EDITOR\nWASD + E/Q fly, mouse look\n1-6 brush   R/F build height\nSPACE place block   X delete\nZ player start   C exit gate\nV jump pad   B enemy spawn   G remove marker\n+/- enemy count   K save   L load\nENTER play-test   ESC title";

pub fn build_ui(tree: &mut UiTreeBuilder) -> EditorHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();
    let mut status = Entity::default();
    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(0.0, 0.0)) + Ab(vec2(22.0, 20.0)),
                Ab(vec2(560.0, 320.0)),
                Anchor::TopLeft,
            )
            .with_text(CONTROLS, 16.0)
            .text_left()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 1.5)
            .color_raw::<UiBase>(TEXT_DIM)
            .entity();
        status = tree
            .add_node()
            .window(Rl(vec2(50.0, 92.0)), Ab(vec2(960.0, 34.0)), Anchor::Center)
            .with_text("", 22.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.0)
            .color_raw::<UiBase>(ACCENT)
            .entity();
    });
    EditorHandles { root, status }
}

fn refresh_status_text(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let editor = &cobalt_world.resources.editor;
    let text = if editor.status > 0.0 {
        format!(
            "{}   |   BRUSH {}   H {:.0}",
            editor.status_text,
            editor.brush.label(),
            editor.place_height
        )
    } else {
        format!(
            "BRUSH {}   HEIGHT {:.0}   BLOCKS {}   ENEMY SPAWNS {}",
            editor.brush.label(),
            editor.place_height,
            editor.data.blocks.len(),
            editor.data.spawn_points.len()
        )
    };
    let status = cobalt_world.resources.ui_handles.editor.status;
    ui_set_text(world, status, &text);
}

fn fly(cobalt_world: &mut CobaltWorld, world: &mut World, delta: f32) {
    let Some(camera) = cobalt_world.resources.player.camera_entity else {
        return;
    };
    let Some(transform) = world.core.get_global_transform(camera) else {
        return;
    };
    let forward = transform.forward_vector().normalize();
    let right = transform.right_vector().normalize();
    let keyboard = &world.resources.input.keyboard;
    let mut move_dir = Vec3::zeros();
    if keyboard.is_key_pressed(KeyCode::KeyW) {
        move_dir += forward;
    }
    if keyboard.is_key_pressed(KeyCode::KeyS) {
        move_dir -= forward;
    }
    if keyboard.is_key_pressed(KeyCode::KeyD) {
        move_dir += right;
    }
    if keyboard.is_key_pressed(KeyCode::KeyA) {
        move_dir -= right;
    }
    if keyboard.is_key_pressed(KeyCode::KeyE) {
        move_dir += vec3(0.0, 1.0, 0.0);
    }
    if keyboard.is_key_pressed(KeyCode::KeyQ) {
        move_dir -= vec3(0.0, 1.0, 0.0);
    }
    if move_dir.norm() < 1e-3 {
        return;
    }
    let step = move_dir.normalize() * FLY_SPEED * delta;
    let Some(player) = cobalt_world.resources.player.player_entity else {
        return;
    };
    if let Some(local) = world.core.get_local_transform_mut(player) {
        local.translation += step;
        local.translation.x = local.translation.x.clamp(-24.0, 24.0);
        local.translation.y = local.translation.y.clamp(0.5, 24.0);
        local.translation.z = local.translation.z.clamp(-24.0, 24.0);
    }
    mark_local_transform_dirty(world, player);
    let current = world
        .core
        .get_local_transform(player)
        .map(|local| local.translation)
        .unwrap_or(VANTAGE);
    if let Some(interpolation) = world.core.get_physics_interpolation_mut(player) {
        interpolation.previous_translation = current;
        interpolation.current_translation = current;
    }
}

fn cursor_on_grid(cobalt_world: &CobaltWorld, world: &World) -> Vec3 {
    let plane_y = cobalt_world.resources.editor.place_height;
    let Some(camera) = cobalt_world.resources.player.camera_entity else {
        return vec3(0.0, plane_y, 0.0);
    };
    let Some(transform) = world.core.get_global_transform(camera) else {
        return vec3(0.0, plane_y, 0.0);
    };
    let origin = transform.translation();
    let forward = transform.forward_vector().normalize();
    let point = if forward.y.abs() > 1e-3 {
        let t = (plane_y - origin.y) / forward.y;
        if t > 0.5 && t < 90.0 {
            origin + forward * t
        } else {
            origin + forward * 12.0
        }
    } else {
        origin + forward * 12.0
    };
    let snap = |value: f32| (value / GRID).round() * GRID;
    vec3(
        snap(point.x).clamp(-19.0, 19.0),
        plane_y,
        snap(point.z).clamp(-19.0, 19.0),
    )
}

fn update_ghost(cobalt_world: &mut CobaltWorld, world: &mut World, cursor: Vec3) {
    let Some(ghost) = cobalt_world.resources.editor.ghost else {
        return;
    };
    let size = brush_size(cobalt_world.resources.editor.brush);
    let center = cursor + vec3(0.0, size.y * 0.5, 0.0);
    if let Some(local) = world.core.get_local_transform_mut(ghost) {
        local.translation = center;
        local.scale = size;
    }
    mark_local_transform_dirty(world, ghost);
}

fn handle_input(cobalt_world: &mut CobaltWorld, world: &mut World, cursor: Vec3) {
    let keyboard = &world.resources.input.keyboard;
    let place = keyboard.just_pressed(KeyCode::Space);
    let brush_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ];
    let brush_index = brush_keys
        .iter()
        .position(|key| keyboard.just_pressed(*key));
    let raise = keyboard.just_pressed(KeyCode::KeyR);
    let lower = keyboard.just_pressed(KeyCode::KeyF);
    let delete = keyboard.just_pressed(KeyCode::KeyX);
    let set_spawn = keyboard.just_pressed(KeyCode::KeyZ);
    let set_exit = keyboard.just_pressed(KeyCode::KeyC);
    let add_pad = keyboard.just_pressed(KeyCode::KeyV);
    let add_enemy = keyboard.just_pressed(KeyCode::KeyB);
    let remove_marker = keyboard.just_pressed(KeyCode::KeyG);
    let more = keyboard.just_pressed(KeyCode::Equal);
    let fewer = keyboard.just_pressed(KeyCode::Minus);
    let save = keyboard.just_pressed(KeyCode::KeyK);
    let load = keyboard.just_pressed(KeyCode::KeyL);
    let play_test = keyboard.just_pressed(KeyCode::Enter);

    if let Some(index) = brush_index {
        cobalt_world.resources.editor.brush = BlockKind::ALL[index];
        status(cobalt_world, BlockKind::ALL[index].label());
    }
    if raise {
        cobalt_world.resources.editor.place_height =
            (cobalt_world.resources.editor.place_height + GRID).min(16.0);
        status(cobalt_world, "RAISE");
    }
    if lower {
        cobalt_world.resources.editor.place_height =
            (cobalt_world.resources.editor.place_height - GRID).max(0.0);
        status(cobalt_world, "LOWER");
    }
    if place {
        place_block(cobalt_world, world, cursor);
    }
    if delete {
        delete_block(cobalt_world, world, cursor);
    }
    if set_spawn {
        cobalt_world.resources.editor.data.spawn = [cursor.x, 1.2 + cursor.y, cursor.z];
        rebuild_markers(cobalt_world, world);
        status(cobalt_world, "PLAYER START");
    }
    if set_exit {
        cobalt_world.resources.editor.data.exit = [cursor.x, cursor.z];
        rebuild_markers(cobalt_world, world);
        status(cobalt_world, "EXIT");
    }
    if add_pad {
        cobalt_world
            .resources
            .editor
            .data
            .pads
            .push((cursor.x, cursor.z));
        rebuild_markers(cobalt_world, world);
        status(cobalt_world, "PAD");
    }
    if add_enemy {
        cobalt_world
            .resources
            .editor
            .data
            .spawn_points
            .push((cursor.x, cursor.z));
        rebuild_markers(cobalt_world, world);
        status(cobalt_world, "ENEMY SPAWN");
    }
    if remove_marker {
        remove_nearest_marker(cobalt_world, world, cursor);
    }
    if more {
        let roster = &mut cobalt_world.resources.editor.data.roster;
        roster.imps += 1;
        roster.swarmers += 1;
        status(cobalt_world, "MORE ENEMIES");
    }
    if fewer {
        let roster = &mut cobalt_world.resources.editor.data.roster;
        roster.imps = roster.imps.saturating_sub(1);
        roster.swarmers = roster.swarmers.saturating_sub(1);
        status(cobalt_world, "FEWER ENEMIES");
    }
    if save {
        save_to_disk(&cobalt_world.resources.editor.data);
        status(cobalt_world, "SAVED");
    }
    if load && let Some(data) = load_from_disk() {
        cobalt_world.resources.editor.data = data;
        rebuild_scene(cobalt_world, world);
        status(cobalt_world, "LOADED");
    }
    if play_test {
        play(cobalt_world, world);
    }
}

fn place_block(cobalt_world: &mut CobaltWorld, world: &mut World, cursor: Vec3) {
    let kind = cobalt_world.resources.editor.brush;
    let size = brush_size(kind);
    let center = cursor + vec3(0.0, size.y * 0.5, 0.0);
    let spec = (center.x, center.y, center.z, size.x, size.y, size.z, kind);
    let entity = spawn_block_entity(world, &spec);
    cobalt_world.resources.editor.data.blocks.push(spec);
    cobalt_world.resources.editor.block_entities.push(entity);
}

fn delete_block(cobalt_world: &mut CobaltWorld, world: &mut World, cursor: Vec3) {
    let blocks = &cobalt_world.resources.editor.data.blocks;
    let mut best: Option<(usize, f32)> = None;
    for (index, block) in blocks.iter().enumerate() {
        let center = vec3(block.0, block.1, block.2);
        let distance = (center - (cursor + vec3(0.0, block.4 * 0.5, 0.0))).norm();
        if distance < 3.0 && best.map(|(_, d)| distance < d).unwrap_or(true) {
            best = Some((index, distance));
        }
    }
    if let Some((index, _)) = best {
        cobalt_world.resources.editor.data.blocks.remove(index);
        let entity = cobalt_world.resources.editor.block_entities.remove(index);
        despawn_recursive_immediate(world, entity);
        status(cobalt_world, "DELETED");
    }
}

fn remove_nearest_marker(cobalt_world: &mut CobaltWorld, world: &mut World, cursor: Vec3) {
    let mut removed = false;
    {
        let pads = &mut cobalt_world.resources.editor.data.pads;
        if let Some(index) = nearest(pads, cursor) {
            pads.remove(index);
            removed = true;
        }
    }
    if !removed {
        let spawns = &mut cobalt_world.resources.editor.data.spawn_points;
        if let Some(index) = nearest(spawns, cursor) {
            spawns.remove(index);
            removed = true;
        }
    }
    if removed {
        rebuild_markers(cobalt_world, world);
        status(cobalt_world, "REMOVED MARKER");
    }
}

fn nearest(points: &[(f32, f32)], cursor: Vec3) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (index, (x, z)) in points.iter().enumerate() {
        let distance = ((x - cursor.x).powi(2) + (z - cursor.z).powi(2)).sqrt();
        if distance < 3.0 && best.map(|(_, d)| distance < d).unwrap_or(true) {
            best = Some((index, distance));
        }
    }
    best.map(|(index, _)| index)
}

fn play(cobalt_world: &mut CobaltWorld, world: &mut World) {
    if cobalt_world.resources.editor.data.blocks.is_empty()
        && cobalt_world.resources.editor.data.spawn_points.is_empty()
    {
        status(cobalt_world, "PLACE SOMETHING FIRST");
        return;
    }
    teardown(cobalt_world, world);
    cobalt_world.resources.editor.active = false;
    game::start_custom(cobalt_world, world);
    lifecycle::enter(cobalt_world, world, Screen::InGame);
}

fn spawn_block_entity(world: &mut World, spec: &content::BlockSpec) -> Entity {
    let (cx, cy, cz, sx, sy, sz, kind) = *spec;
    let entity = spawn_static_physics_cube_with_material(
        world,
        vec3(cx, cy, cz),
        vec3(sx, sy, sz),
        material_for(kind),
    );
    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "EditBlock".to_string();
    }
    entity
}

fn named_block(
    world: &mut World,
    name: &str,
    center: Vec3,
    size: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_static_physics_cube_with_material(world, center, size, material);
    if let Some(entity_name) = world.core.get_name_mut(entity) {
        entity_name.0 = name.to_string();
    }
    entity
}

fn marker_mesh(world: &mut World, center: Vec3, size: Vec3, material: &str) -> Entity {
    let entity = spawn_mesh_at(world, "Cube", center, size);
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material.to_string()));
    entity
}

fn spawn_lamp(world: &mut World, position: Vec3, color: Vec3) -> Entity {
    let entity = spawn_light_entity(world, position, "Lamp");
    world.core.set_light(
        entity,
        nightshade::ecs::light::components::Light {
            light_type: nightshade::ecs::light::components::LightType::Point,
            color,
            intensity: 38.0,
            range: 28.0,
            ..Default::default()
        },
    );
    entity
}

fn status(cobalt_world: &mut CobaltWorld, text: &str) {
    cobalt_world.resources.editor.status = 2.0;
    cobalt_world.resources.editor.status_text = text.to_string();
}

fn save_to_disk(data: &LevelData) {
    let mut out = String::new();
    out.push_str(&format!("ATMO {}\n", data.atmosphere_index));
    out.push_str(&format!(
        "FOG {} {} {}\n",
        data.fog[0], data.fog[1], data.fog[2]
    ));
    out.push_str(&format!(
        "SPAWN {} {} {}\n",
        data.spawn[0], data.spawn[1], data.spawn[2]
    ));
    out.push_str(&format!("EXIT {} {}\n", data.exit[0], data.exit[1]));
    out.push_str(&format!(
        "ROSTER {} {} {} {} {} {}\n",
        data.roster.imps,
        data.roster.swarmers,
        data.roster.casters,
        data.roster.brutes,
        data.roster.gargoyles,
        data.roster.sentinels
    ));
    for block in &data.blocks {
        out.push_str(&format!(
            "BLOCK {} {} {} {} {} {} {}\n",
            block.0,
            block.1,
            block.2,
            block.3,
            block.4,
            block.5,
            block.6.code()
        ));
    }
    for (x, z) in &data.pads {
        out.push_str(&format!("PAD {x} {z}\n"));
    }
    for (x, z) in &data.spawn_points {
        out.push_str(&format!("ESPAWN {x} {z}\n"));
    }
    let _ = std::fs::write(SAVE_PATH, out);
}

fn load_from_disk() -> Option<LevelData> {
    let text = std::fs::read_to_string(SAVE_PATH).ok()?;
    let mut data = LevelData::default();
    data.blocks.clear();
    data.pads.clear();
    data.spawn_points.clear();
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.first().copied() {
            Some("ATMO") => {
                if let Some(value) = parts.get(1).and_then(|value| value.parse().ok()) {
                    data.atmosphere_index = value;
                }
            }
            Some("FOG") => {
                for (index, value) in parts.iter().skip(1).take(3).enumerate() {
                    if let Ok(parsed) = value.parse() {
                        data.fog[index] = parsed;
                    }
                }
            }
            Some("SPAWN") => {
                for (index, value) in parts.iter().skip(1).take(3).enumerate() {
                    if let Ok(parsed) = value.parse() {
                        data.spawn[index] = parsed;
                    }
                }
            }
            Some("EXIT") => {
                for (index, value) in parts.iter().skip(1).take(2).enumerate() {
                    if let Ok(parsed) = value.parse() {
                        data.exit[index] = parsed;
                    }
                }
            }
            Some("ROSTER") => {
                let nums: Vec<u32> = parts
                    .iter()
                    .skip(1)
                    .filter_map(|value| value.parse().ok())
                    .collect();
                if nums.len() >= 5 {
                    data.roster = content::Roster {
                        imps: nums[0],
                        swarmers: nums[1],
                        casters: nums[2],
                        brutes: nums[3],
                        gargoyles: nums[4],
                        sentinels: nums.get(5).copied().unwrap_or(0),
                    };
                }
            }
            Some("BLOCK") => {
                let nums: Vec<f32> = parts
                    .iter()
                    .skip(1)
                    .filter_map(|value| value.parse().ok())
                    .collect();
                if nums.len() == 7 {
                    data.blocks.push((
                        nums[0],
                        nums[1],
                        nums[2],
                        nums[3],
                        nums[4],
                        nums[5],
                        BlockKind::from_code(nums[6] as u8),
                    ));
                }
            }
            Some("PAD") => parse_pair(&parts, &mut data.pads),
            Some("ESPAWN") => parse_pair(&parts, &mut data.spawn_points),
            _ => {}
        }
    }
    Some(data)
}

fn parse_pair(parts: &[&str], out: &mut Vec<(f32, f32)>) {
    if let (Some(Ok(x)), Some(Ok(z))) = (
        parts.get(1).map(|value| value.parse()),
        parts.get(2).map(|value| value.parse()),
    ) {
        out.push((x, z));
    }
}
