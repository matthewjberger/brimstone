use crate::systems::world::textures;
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::ecs::physics::commands::spawn_static_physics_cube_with_material;
use nightshade::ecs::world::commands::spawn_light_entity;
use nightshade::prelude::*;

pub const PLAYER_SPAWN: Vec3 = Vec3::new(0.0, 1.2, 13.0);

const WALL_HEIGHT: f32 = 8.0;
const WALL_THICKNESS: f32 = 1.0;

/// Cover pillars: (x, z, height, half_extent). Placed off the cardinal lanes
/// so there are clear running lines between them, but they break sightlines.
const PILLARS: [(f32, f32, f32, f32); 6] = [
    (8.5, 5.0, 4.0, 0.9),
    (-7.5, 8.0, 2.6, 1.1),
    (-9.5, -5.5, 4.5, 0.8),
    (6.5, -9.0, 2.2, 1.2),
    (12.5, -3.0, 3.2, 0.9),
    (-12.0, 2.5, 3.6, 0.85),
];

/// Low peeking cover: (x, z, half_x, half_z).
const LOW_COVER: [(f32, f32, f32, f32); 4] = [
    (4.0, 8.5, 1.6, 0.7),
    (-4.5, -8.0, 1.6, 0.7),
    (9.0, 1.0, 0.7, 1.6),
    (-9.0, -1.0, 0.7, 1.6),
];

/// Internal choke walls that carve lanes: (cx, cz, half_x, half_z, height).
const CHOKE_WALLS: [(f32, f32, f32, f32, f32); 4] = [
    (5.5, 0.0, 0.5, 4.0, 2.6),
    (-5.5, 0.0, 0.5, 4.0, 2.6),
    (0.0, 6.0, 4.0, 0.5, 2.6),
    (0.0, -6.0, 4.0, 0.5, 2.6),
];

/// Glowing landmark beacons: (x, z, color). Bright HDR colors so they bloom and
/// give the player something to orient and kite around.
const BEACONS: [(f32, f32, [f32; 3]); 4] = [
    (5.0, 5.0, [0.2, 1.5, 1.8]),
    (-5.0, 5.0, [1.6, 0.3, 1.5]),
    (5.0, -5.0, [1.7, 0.8, 0.2]),
    (-5.0, -5.0, [0.3, 1.6, 0.5]),
];

pub fn build(world: &mut World) {
    let span = tuning::ARENA_HALF * 2.0;

    spawn_block(
        world,
        "Floor",
        vec3(0.0, -0.5, 0.0),
        vec3(span, 1.0, span),
        textures::floor_material(),
    );

    let edge = tuning::ARENA_HALF + WALL_THICKNESS * 0.5;
    let wall_length = span + WALL_THICKNESS * 2.0;
    let height_center = WALL_HEIGHT * 0.5;
    spawn_block(
        world,
        "WallN",
        vec3(0.0, height_center, -edge),
        vec3(wall_length, WALL_HEIGHT, WALL_THICKNESS),
        textures::wall_material(),
    );
    spawn_block(
        world,
        "WallS",
        vec3(0.0, height_center, edge),
        vec3(wall_length, WALL_HEIGHT, WALL_THICKNESS),
        textures::wall_material(),
    );
    spawn_block(
        world,
        "WallW",
        vec3(-edge, height_center, 0.0),
        vec3(WALL_THICKNESS, WALL_HEIGHT, wall_length),
        textures::wall_material(),
    );
    spawn_block(
        world,
        "WallE",
        vec3(edge, height_center, 0.0),
        vec3(WALL_THICKNESS, WALL_HEIGHT, wall_length),
        textures::wall_material(),
    );

    spawn_block(
        world,
        "Monument",
        vec3(0.0, 3.5, 0.0),
        vec3(3.0, 7.0, 3.0),
        textures::pillar_material(),
    );

    for (x, z, height, half) in PILLARS {
        spawn_block(
            world,
            "Pillar",
            vec3(x, height * 0.5, z),
            vec3(half * 2.0, height, half * 2.0),
            textures::pillar_material(),
        );
    }

    for (x, z, half_x, half_z) in LOW_COVER {
        spawn_block(
            world,
            "Cover",
            vec3(x, 0.45, z),
            vec3(half_x * 2.0, 0.9, half_z * 2.0),
            textures::platform_material(),
        );
    }

    for (cx, cz, half_x, half_z, height) in CHOKE_WALLS {
        spawn_block(
            world,
            "Choke",
            vec3(cx, height * 0.5, cz),
            vec3(half_x * 2.0, height, half_z * 2.0),
            textures::accent_material(),
        );
    }

    for (x, z, color) in BEACONS {
        let color = vec3(color[0], color[1], color[2]);
        spawn_block(
            world,
            "Beacon",
            vec3(x, 2.5, z),
            vec3(0.7, 5.0, 0.7),
            textures::beacon_material(color, 2.6),
        );
        spawn_lamp(world, vec3(x, 3.0, z), color, 30.0, 15.0);
        spawn_embers(world, vec3(x, 0.2, z), color);
    }

    spawn_sun(world);
    spawn_lamp(world, vec3(0.0, 8.0, 0.0), vec3(0.6, 0.6, 0.9), 40.0, 26.0);
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

fn spawn_lamp(world: &mut World, position: Vec3, color: Vec3, intensity: f32, range: f32) {
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
}

fn spawn_embers(world: &mut World, position: Vec3, color: Vec3) {
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
}
