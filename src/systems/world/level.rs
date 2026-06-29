use crate::content::{BlockKind, Level};
use crate::ecs::BoomerWorld;
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

const WALL_HEIGHT: f32 = 8.0;
const WALL_THICKNESS: f32 = 1.0;

pub fn build(boomer_world: &mut BoomerWorld, world: &mut World, level: &Level) {
    let settings = &mut world.resources.render_settings;
    settings.atmosphere = level.atmosphere;
    settings.fog = Some(Fog {
        color: level.fog,
        start: 16.0,
        end: 60.0,
    });
    capture_procedural_atmosphere_ibl(world, level.atmosphere, 0.0);

    let mut geometry: Vec<Entity> = Vec::new();
    let span = tuning::ARENA_HALF * 2.0;

    geometry.push(spawn_block(
        world,
        "Floor",
        vec3(0.0, -0.5, 0.0),
        vec3(span, 1.0, span),
        textures::floor_material(),
    ));

    let edge = tuning::ARENA_HALF + WALL_THICKNESS * 0.5;
    let wall_length = span + WALL_THICKNESS * 2.0;
    let height_center = WALL_HEIGHT * 0.5;
    geometry.push(spawn_block(
        world,
        "Wall",
        vec3(0.0, height_center, -edge),
        vec3(wall_length, WALL_HEIGHT, WALL_THICKNESS),
        textures::wall_material(),
    ));
    geometry.push(spawn_block(
        world,
        "Wall",
        vec3(0.0, height_center, edge),
        vec3(wall_length, WALL_HEIGHT, WALL_THICKNESS),
        textures::wall_material(),
    ));
    geometry.push(spawn_block(
        world,
        "Wall",
        vec3(-edge, height_center, 0.0),
        vec3(WALL_THICKNESS, WALL_HEIGHT, wall_length),
        textures::wall_material(),
    ));
    geometry.push(spawn_block(
        world,
        "Wall",
        vec3(edge, height_center, 0.0),
        vec3(WALL_THICKNESS, WALL_HEIGHT, wall_length),
        textures::wall_material(),
    ));

    for (cx, cy, cz, sx, sy, sz, kind) in level.blocks {
        geometry.push(spawn_block(
            world,
            "Block",
            vec3(*cx, *cy, *cz),
            vec3(*sx, *sy, *sz),
            material_for(*kind),
        ));
    }

    for (x, z, color) in level.beacons {
        let color = vec3(color[0], color[1], color[2]);
        geometry.push(spawn_block(
            world,
            "Beacon",
            vec3(*x, 2.5, *z),
            vec3(0.7, 5.0, 0.7),
            textures::beacon_material(color, 2.6),
        ));
        geometry.push(spawn_lamp(world, vec3(*x, 3.0, *z), color, 28.0, 15.0));
        geometry.push(spawn_embers(world, vec3(*x, 0.2, *z), color));
    }

    let exit_position = vec3(level.exit[0], 0.0, level.exit[1]);
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

    boomer_world.resources.level.geometry = geometry;
    boomer_world.resources.level.exit_entity = Some(exit);
    boomer_world.resources.level.exit_position = exit_position;
    boomer_world.resources.level.exit_active = false;
}

pub fn despawn(boomer_world: &mut BoomerWorld, world: &mut World) {
    for entity in boomer_world.resources.level.geometry.drain(..) {
        despawn_recursive_immediate(world, entity);
    }
    boomer_world.resources.level.exit_entity = None;
}

pub fn open_exit(boomer_world: &mut BoomerWorld, world: &mut World) {
    boomer_world.resources.level.exit_active = true;
    if let Some(exit) = boomer_world.resources.level.exit_entity {
        world
            .core
            .set_visibility(exit, Visibility { visible: true });
    }
    let position = boomer_world.resources.level.exit_position;
    let lamp = spawn_lamp(
        world,
        vec3(position.x, 2.5, position.z),
        vec3(0.3, 1.8, 0.7),
        60.0,
        18.0,
    );
    boomer_world.resources.level.geometry.push(lamp);
}

fn material_for(kind: BlockKind) -> Material {
    match kind {
        BlockKind::Wall => textures::wall_material(),
        BlockKind::Pillar => textures::pillar_material(),
        BlockKind::Cover => textures::platform_material(),
        BlockKind::Choke => textures::accent_material(),
        BlockKind::Monument => textures::pillar_material(),
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

fn spawn_embers(world: &mut World, position: Vec3, color: Vec3) -> Entity {
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
