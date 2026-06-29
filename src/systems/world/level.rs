use crate::systems::world::textures;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::physics::commands::spawn_static_physics_cube_with_material;
use nightshade::prelude::*;

pub const ARENA_HALF: f32 = 18.0;
pub const PLAYER_SPAWN: Vec3 = Vec3::new(0.0, 1.2, 0.0);

const WALL_HEIGHT: f32 = 7.0;
const WALL_THICKNESS: f32 = 1.0;

pub fn build(world: &mut World) {
    let span = ARENA_HALF * 2.0;

    spawn_static_physics_cube_with_material(
        world,
        vec3(0.0, -0.5, 0.0),
        vec3(span, 1.0, span),
        textures::floor_material(),
    );

    let edge = ARENA_HALF + WALL_THICKNESS * 0.5;
    let wall_length = span + WALL_THICKNESS * 2.0;
    let height_center = WALL_HEIGHT * 0.5;

    spawn_static_physics_cube_with_material(
        world,
        vec3(0.0, height_center, -edge),
        vec3(wall_length, WALL_HEIGHT, WALL_THICKNESS),
        textures::wall_material(),
    );
    spawn_static_physics_cube_with_material(
        world,
        vec3(0.0, height_center, edge),
        vec3(wall_length, WALL_HEIGHT, WALL_THICKNESS),
        textures::wall_material(),
    );
    spawn_static_physics_cube_with_material(
        world,
        vec3(-edge, height_center, 0.0),
        vec3(WALL_THICKNESS, WALL_HEIGHT, wall_length),
        textures::wall_material(),
    );
    spawn_static_physics_cube_with_material(
        world,
        vec3(edge, height_center, 0.0),
        vec3(WALL_THICKNESS, WALL_HEIGHT, wall_length),
        textures::wall_material(),
    );

    let pillar_offset = ARENA_HALF * 0.55;
    let pillar_positions = [
        vec3(-pillar_offset, 1.5, -pillar_offset),
        vec3(pillar_offset, 1.5, -pillar_offset),
        vec3(-pillar_offset, 1.5, pillar_offset),
        vec3(pillar_offset, 1.5, pillar_offset),
    ];
    for position in pillar_positions {
        spawn_static_physics_cube_with_material(
            world,
            position,
            vec3(2.0, 3.0, 2.0),
            textures::pillar_material(),
        );
    }
}
