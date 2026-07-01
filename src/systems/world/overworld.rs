//! Adventure open-world overworld: the engine's procedural clipmap terrain drives
//! an effectively unbounded landscape the player roams. Physics colliders stream
//! around the player each frame, so the first-person character controller walks the
//! hills with the exact movement it uses in the arenas. Bounded arena "cells" are
//! reached from out here, so dense navmesh combat stays inside them while the
//! overworld stays open.

use crate::content::BlockSpec;
use crate::systems::world::level;
use nalgebra_glm::Vec3;
use nightshade::ecs::graphics::resources::Fog;
use nightshade::ecs::grass::{GrassDomain, GrassTypeParams};
use nightshade::prelude::*;

const OVERWORLD_SEED: u32 = 0x0B12_5709;
const FLATTEN_RADIUS: f32 = 70.0;
const HEIGHT_MIN: f32 = -30.0;
const HEIGHT_MAX: f32 = 150.0;
const SNOW_HEIGHT: f32 = 96.0;
const COLLIDER_RADIUS: i32 = 5;
// Pull the procedural features in from continental scale (~4 km) to gameplay scale
// (~900 m) and give the ground real walking-scale relief instead of a flat plain.
const CONTINENTAL_FREQUENCY: f32 = 1.0 / 900.0;
const EROSION_FREQUENCY: f32 = 1.0 / 480.0;
const RIDGE_FREQUENCY: f32 = 1.0 / 180.0;
const DETAIL_AMPLITUDE: f32 = 7.0;
const FOG_COLOR: [f32; 3] = [0.46, 0.53, 0.66];
const FOG_START: f32 = 240.0;
const FOG_END: f32 = 1300.0;

/// Turn on the streamed terrain and return the sun plus the area's solid blocks as
/// free-standing props. Primes colliders at `spawn` so the player lands on ground.
pub fn enter(world: &mut World, blocks: &[BlockSpec], spawn: Vec3) -> Vec<Entity> {
    let terrain = &mut world.resources.terrain;
    terrain.enabled = true;
    terrain.seed = OVERWORLD_SEED;
    terrain.height_min = HEIGHT_MIN;
    terrain.height_max = HEIGHT_MAX;
    terrain.snow_height = SNOW_HEIGHT;
    terrain.continental_frequency = CONTINENTAL_FREQUENCY;
    terrain.erosion_frequency = EROSION_FREQUENCY;
    terrain.ridge_frequency = RIDGE_FREQUENCY;
    terrain.detail_amplitude = DETAIL_AMPLITUDE;
    terrain.origin_flatten_radius = FLATTEN_RADIUS;
    terrain.collider_radius = COLLIDER_RADIUS;
    terrain.revision += 1;

    world.resources.render_settings.fog = Some(Fog {
        color: FOG_COLOR,
        start: FOG_START,
        end: FOG_END,
    });

    world.resources.grass.enabled = true;
    world.resources.grass.domain = GrassDomain::Infinite;
    if world.resources.grass.types.is_empty() {
        world.resources.grass.types = vec![GrassTypeParams::meadow()];
        world.resources.grass.types_revision += 1;
    }

    terrain_collider_system(world, spawn);

    let mut geometry = vec![spawn_sun(world)];
    geometry.extend(level::spawn_props(world, blocks));
    geometry
}

/// Stream terrain colliders around `center` every frame while the overworld is live.
pub fn update(world: &mut World, center: Vec3) {
    if world.resources.terrain.enabled {
        terrain_collider_system(world, center);
    }
}

/// Turn the terrain off when leaving the overworld for a bounded cell or the title.
pub fn leave(world: &mut World) {
    if world.resources.terrain.enabled {
        world.resources.terrain.enabled = false;
        world.resources.terrain.revision += 1;
    }
    world.resources.grass.enabled = false;
}
