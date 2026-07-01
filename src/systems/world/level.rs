use crate::content::{BlockKind, BlockSpec, Level, LevelData, atmosphere_for};
use crate::ecs::CobaltWorld;
use crate::systems::world::textures::{self, MAT_EXIT};
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::graphics::resources::Fog;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::ecs::physics::commands::spawn_static_physics_cube_with_material;
use nightshade::ecs::world::commands::{spawn_light_entity, spawn_mesh_at};
use nightshade::prelude::*;

pub const PLAYER_SPAWN: Vec3 = Vec3::new(0.0, 1.2, 14.0);

// Tall enough that the player can't escape by stacking a pad launch (~5m) onto
// a pillar top and rocket-jumping (~6.25m) over the perimeter.
const WALL_HEIGHT: f32 = 12.0;
const WALL_THICKNESS: f32 = 1.0;

pub fn build(cobalt_world: &mut CobaltWorld, world: &mut World, level: &Level) {
    apply_environment(world, level.atmosphere, level.fog);
    cobalt_world.resources.level.half_x = level.half_x;
    cobalt_world.resources.level.half_z = level.half_z;
    let mut geometry = spawn_shell(world, level.half_x, level.half_z);
    let mut obstacles: Vec<(Vec3, Vec3)> = Vec::new();

    for (cx, cy, cz, sx, sy, sz, kind) in level.blocks {
        let center = vec3(*cx, *cy, *cz);
        let size = vec3(*sx, *sy, *sz);
        geometry.push(spawn_block(
            world,
            "Block",
            center,
            size,
            material_for(*kind),
        ));
        obstacles.push((center, size));
    }

    for (cx, cy, cz, sx, sy, sz, pitch, yaw) in level.ramps {
        let entity = spawn_block(
            world,
            "Ramp",
            vec3(*cx, *cy, *cz),
            vec3(*sx, *sy, *sz),
            textures::platform_material(),
        );
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.rotation = nalgebra_glm::quat_angle_axis(*yaw, &vec3(0.0, 1.0, 0.0))
                * nalgebra_glm::quat_angle_axis(*pitch, &vec3(1.0, 0.0, 0.0));
        }
        world
            .core
            .set_local_transform_dirty(entity, LocalTransformDirty);
        geometry.push(entity);
    }

    for (x, z, color) in level.beacons {
        let color = vec3(color[0], color[1], color[2]);
        let center = vec3(*x, 2.5, *z);
        let size = vec3(0.7, 5.0, 0.7);
        geometry.push(spawn_block(
            world,
            "Beacon",
            center,
            size,
            textures::beacon_material(color, 2.6),
        ));
        obstacles.push((center, size));
        geometry.push(spawn_lamp(world, vec3(*x, 3.0, *z), color, 28.0, 15.0));
        geometry.push(spawn_embers(world, vec3(*x, 0.2, *z), color));
    }

    let pads = spawn_pads(world, level.pads, &mut geometry);
    let exit_position = vec3(level.exit[0], 0.0, level.exit[1]);
    finalize_level(cobalt_world, world, geometry, pads, exit_position);
    rebuild_navmesh(world, &obstacles, level.half_x, level.half_z);
}

/// Build a level from owned editor/custom data (no beacons or ramps).
pub fn build_dynamic(cobalt_world: &mut CobaltWorld, world: &mut World, data: &LevelData) {
    apply_environment(world, atmosphere_for(data.atmosphere_index), data.fog);
    cobalt_world.resources.level.half_x = tuning::ARENA_HALF;
    cobalt_world.resources.level.half_z = tuning::ARENA_HALF;
    let mut geometry = spawn_shell(world, tuning::ARENA_HALF, tuning::ARENA_HALF);
    let mut obstacles: Vec<(Vec3, Vec3)> = Vec::new();

    for (cx, cy, cz, sx, sy, sz, kind) in &data.blocks {
        let center = vec3(*cx, *cy, *cz);
        let size = vec3(*sx, *sy, *sz);
        geometry.push(spawn_block(
            world,
            "Block",
            center,
            size,
            material_for(*kind),
        ));
        obstacles.push((center, size));
    }

    let pads = spawn_pads(world, &data.pads, &mut geometry);
    let exit_position = vec3(data.exit[0], 0.0, data.exit[1]);
    finalize_level(cobalt_world, world, geometry, pads, exit_position);
    rebuild_navmesh(world, &obstacles, tuning::ARENA_HALF, tuning::ARENA_HALF);
}

/// Build a free-standing area for adventure mode: floor, perimeter walls, the
/// given solid blocks, a sun and a soft overhead fill light, with a navmesh baked
/// over it. Returns every spawned entity for later teardown. No exit gate, pads,
/// or arcade wave state — the caller owns spawns and accent lighting.
pub fn build_arena(
    world: &mut World,
    blocks: &[BlockSpec],
    half_x: f32,
    half_z: f32,
) -> Vec<Entity> {
    let mut geometry = spawn_shell(world, half_x, half_z);
    let mut obstacles: Vec<(Vec3, Vec3)> = Vec::new();
    for (cx, cy, cz, sx, sy, sz, kind) in blocks {
        let center = vec3(*cx, *cy, *cz);
        let size = vec3(*sx, *sy, *sz);
        geometry.push(spawn_block(
            world,
            "Block",
            center,
            size,
            material_for(*kind),
        ));
        obstacles.push((center, size));
    }
    geometry.push(spawn_sun(world));
    // A gentle overhead fill so the large floor isn't pitch black, but low enough
    // that it doesn't blow past the bloom threshold and make the ground glow.
    geometry.push(spawn_lamp(
        world,
        vec3(0.0, 12.0, 0.0),
        vec3(0.5, 0.5, 0.62),
        10.0,
        half_x.max(half_z) * 1.1,
    ));
    rebuild_navmesh(world, &obstacles, half_x, half_z);
    geometry
}

/// A coloured point light at `position` for adventure-mode accent lighting.
pub fn spawn_accent_light(world: &mut World, position: Vec3, color: Vec3) -> Entity {
    spawn_lamp(world, position, color, 11.0, 14.0)
}

/// A stretched, material-textured cube used as a world marker (e.g. a portal
/// gate). Real geometry rather than a billboard, so it stays visible from every
/// side instead of vanishing when you walk behind a camera-facing plane.
pub fn spawn_marker(world: &mut World, position: Vec3, size: Vec3, material: &str) -> Entity {
    let entity = spawn_mesh_at(world, "Cube", position, size);
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material.to_string()));
    entity
}

pub fn apply_environment(world: &mut World, atmosphere: Atmosphere, fog: [f32; 3]) {
    let settings = &mut world.resources.render_settings;
    settings.atmosphere = atmosphere;
    settings.fog = Some(Fog {
        color: fog,
        start: 16.0,
        end: 60.0,
    });
    capture_procedural_atmosphere_ibl(world, atmosphere, 0.0);
}

fn spawn_shell(world: &mut World, half_x: f32, half_z: f32) -> Vec<Entity> {
    let mut geometry: Vec<Entity> = Vec::new();

    geometry.push(spawn_block(
        world,
        "Floor",
        vec3(0.0, -0.5, 0.0),
        vec3(half_x * 2.0, 1.0, half_z * 2.0),
        textures::floor_material(),
    ));

    let edge_x = half_x + WALL_THICKNESS * 0.5;
    let edge_z = half_z + WALL_THICKNESS * 0.5;
    let length_x = half_x * 2.0 + WALL_THICKNESS * 2.0;
    let length_z = half_z * 2.0 + WALL_THICKNESS * 2.0;
    let height_center = WALL_HEIGHT * 0.5;
    let walls = [
        (
            vec3(0.0, height_center, -edge_z),
            vec3(length_x, WALL_HEIGHT, WALL_THICKNESS),
        ),
        (
            vec3(0.0, height_center, edge_z),
            vec3(length_x, WALL_HEIGHT, WALL_THICKNESS),
        ),
        (
            vec3(-edge_x, height_center, 0.0),
            vec3(WALL_THICKNESS, WALL_HEIGHT, length_z),
        ),
        (
            vec3(edge_x, height_center, 0.0),
            vec3(WALL_THICKNESS, WALL_HEIGHT, length_z),
        ),
    ];
    for (center, size) in walls {
        geometry.push(spawn_block(
            world,
            "Wall",
            center,
            size,
            textures::wall_material(),
        ));
    }
    geometry
}

fn spawn_pads(world: &mut World, pads: &[(f32, f32)], geometry: &mut Vec<Entity>) -> Vec<Vec3> {
    let mut out: Vec<Vec3> = Vec::new();
    for (x, z) in pads {
        let pad_color = vec3(0.3, 1.4, 1.7);
        let pad = spawn_mesh_at(world, "Cube", vec3(*x, 0.12, *z), vec3(2.4, 0.24, 2.4));
        if let Some(name) = world.core.get_name_mut(pad) {
            name.0 = "Pad".to_string();
        }
        world
            .core
            .set_material_ref(pad, MaterialRef::new(textures::PAD_MATERIAL.to_string()));
        geometry.push(pad);
        geometry.push(spawn_lamp(world, vec3(*x, 0.8, *z), pad_color, 18.0, 9.0));
        out.push(vec3(*x, 0.0, *z));
    }
    out
}

fn finalize_level(
    cobalt_world: &mut CobaltWorld,
    world: &mut World,
    mut geometry: Vec<Entity>,
    pads: Vec<Vec3>,
    exit_position: Vec3,
) {
    cobalt_world.resources.level.pads = pads;

    let exit = spawn_mesh_at(
        world,
        "Cube",
        vec3(exit_position.x, 2.4, exit_position.z),
        vec3(2.6, 4.8, 0.5),
    );
    if let Some(name) = world.core.get_name_mut(exit) {
        name.0 = "Exit".to_string();
    }
    world
        .core
        .set_material_ref(exit, MaterialRef::new(MAT_EXIT.to_string()));
    world
        .core
        .set_visibility(exit, Visibility { visible: false });
    geometry.push(exit);

    geometry.push(spawn_sun(world));
    geometry.push(spawn_lamp(
        world,
        vec3(0.0, 9.0, 0.0),
        vec3(0.55, 0.55, 0.85),
        38.0,
        28.0,
    ));

    cobalt_world.resources.level.geometry = geometry;
    cobalt_world.resources.level.exit_entity = Some(exit);
    cobalt_world.resources.level.exit_position = exit_position;
    cobalt_world.resources.level.exit_active = false;
}

pub fn despawn(cobalt_world: &mut CobaltWorld, world: &mut World) {
    for entity in cobalt_world.resources.level.geometry.drain(..) {
        despawn_recursive_immediate(world, entity);
    }
    cobalt_world.resources.level.exit_entity = None;
    clear_navmesh(world);
}

/// Bake a navmesh for the level so ground enemies route around walls and blocks
/// instead of clipping through them. The walkable floor is one quad at y=0 and
/// every solid block/beacon is punched in as a box obstacle; the Recast bake runs
/// synchronously and lands in `world.resources.navmesh`. Built from the level's
/// authored geometry rather than physics colliders, which are not yet synced into
/// the collider set at level-build time.
fn rebuild_navmesh(world: &mut World, obstacles: &[(Vec3, Vec3)], half_x: f32, half_z: f32) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();
    push_floor(&mut vertices, &mut indices, half_x, half_z);
    for (center, size) in obstacles {
        push_box(&mut vertices, &mut indices, *center, *size);
    }

    let config = RecastNavMeshConfig {
        agent_radius: tuning::NAV_AGENT_RADIUS,
        agent_height: tuning::NAV_AGENT_HEIGHT,
        walkable_climb: tuning::NAV_WALKABLE_CLIMB,
        ..Default::default()
    };
    match generate_navmesh_recast(&vertices, &indices, &config) {
        Some(navmesh) => world.resources.navmesh = navmesh,
        None => clear_navmesh(world),
    }
}

/// The walkable ground: a single quad spanning the level footprint at y=0.
fn push_floor(vertices: &mut Vec<[f32; 3]>, indices: &mut Vec<[u32; 3]>, half_x: f32, half_z: f32) {
    let base = vertices.len() as u32;
    vertices.push([-half_x, 0.0, -half_z]);
    vertices.push([half_x, 0.0, -half_z]);
    vertices.push([half_x, 0.0, half_z]);
    vertices.push([-half_x, 0.0, half_z]);
    indices.push([base, base + 2, base + 1]);
    indices.push([base, base + 3, base + 2]);
}

/// A solid box obstacle (8 corners, 12 triangles). `size` is the full extent, so
/// the half-extents are `size * 0.5` about `center` — matching how blocks spawn.
fn push_box(vertices: &mut Vec<[f32; 3]>, indices: &mut Vec<[u32; 3]>, center: Vec3, size: Vec3) {
    let half = size * 0.5;
    let base = vertices.len() as u32;
    for sx in [-1.0_f32, 1.0] {
        for sy in [-1.0_f32, 1.0] {
            for sz in [-1.0_f32, 1.0] {
                vertices.push([
                    center.x + sx * half.x,
                    center.y + sy * half.y,
                    center.z + sz * half.z,
                ]);
            }
        }
    }
    // Corner index = (x_bit << 2) | (y_bit << 1) | z_bit, in the push order above.
    const FACES: [[u32; 3]; 12] = [
        [0, 2, 3],
        [0, 3, 1], // -X
        [4, 5, 7],
        [4, 7, 6], // +X
        [0, 1, 5],
        [0, 5, 4], // -Y
        [2, 6, 7],
        [2, 7, 3], // +Y
        [0, 4, 6],
        [0, 6, 2], // -Z
        [1, 3, 7],
        [1, 7, 5], // +Z
    ];
    for face in FACES {
        indices.push([base + face[0], base + face[1], base + face[2]]);
    }
}

pub fn open_exit(cobalt_world: &mut CobaltWorld, world: &mut World) {
    cobalt_world.resources.level.exit_active = true;
    if let Some(exit) = cobalt_world.resources.level.exit_entity {
        world
            .core
            .set_visibility(exit, Visibility { visible: true });
    }
    let position = cobalt_world.resources.level.exit_position;
    let lamp = spawn_lamp(
        world,
        vec3(position.x, 2.5, position.z),
        vec3(0.3, 1.8, 0.7),
        60.0,
        18.0,
    );
    cobalt_world.resources.level.geometry.push(lamp);
}

pub fn material_for(kind: BlockKind) -> Material {
    match kind {
        BlockKind::Wall => textures::wall_material(),
        BlockKind::Pillar => textures::pillar_material(),
        BlockKind::Cover => textures::platform_material(),
        BlockKind::Choke => textures::accent_material(),
        BlockKind::Monument => textures::pillar_material(),
        BlockKind::Platform => textures::platform_material(),
        BlockKind::Core => textures::core_material(),
    }
}

fn spawn_block(
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

fn spawn_lamp(
    world: &mut World,
    position: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
) -> Entity {
    let entity = spawn_light_entity(world, position, "Lamp");
    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Point,
            color,
            intensity,
            range,
            ..Default::default()
        },
    );
    entity
}

pub fn spawn_embers(world: &mut World, position: Vec3, color: Vec3) -> Entity {
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.5 },
        position,
        direction: vec3(0.0, 1.0, 0.0),
        spawn_rate: 12.0,
        burst_count: 0,
        particle_lifetime_min: 1.6,
        particle_lifetime_max: 3.2,
        initial_velocity_min: 0.4,
        initial_velocity_max: 1.2,
        velocity_spread: 0.6,
        gravity: vec3(0.0, 0.35, 0.0),
        drag: 0.25,
        size_start: 0.07,
        size_end: 0.0,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, nalgebra_glm::vec4(color.x, color.y, color.z, 0.0)),
                (0.3, nalgebra_glm::vec4(color.x, color.y, color.z, 0.9)),
                (
                    1.0,
                    nalgebra_glm::vec4(color.x * 0.4, color.y * 0.4, color.z * 0.4, 0.0),
                ),
            ],
        },
        emissive_strength: 4.0,
        turbulence_strength: 0.4,
        turbulence_frequency: 2.0,
        ..Default::default()
    };
    let entity = spawn_entities(world, NAME | PARTICLE_EMITTER, 1)[0];
    world.core.set_particle_emitter(entity, emitter);
    entity
}
