use crate::ecs::{BoomerWorld, ENEMY, EnemyState};
use crate::systems::world::{audio, enemies, flash};
use nalgebra_glm::{Vec3, dot};
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::physics::resources::physics_world_cast_ray;
use nightshade::prelude::*;

const FIRE_COOLDOWN: f32 = 0.55;
const PELLET_DAMAGE: f32 = 10.0;
const RANGE: f32 = 60.0;
const SPREAD: f32 = 0.055;

const PATTERN: [(f32, f32); 7] = [
    (0.0, 0.0),
    (SPREAD, 0.0),
    (-SPREAD, 0.0),
    (0.0, SPREAD),
    (0.0, -SPREAD),
    (SPREAD * 0.7, SPREAD * 0.7),
    (-SPREAD * 0.7, -SPREAD * 0.7),
];

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    boomer_world.resources.weapon.cooldown =
        (boomer_world.resources.weapon.cooldown - delta).max(0.0);

    let gamepad_fire = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::RightTrigger2);
    let fire = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::LEFT_JUST_PRESSED)
        || gamepad_fire;
    if !fire || boomer_world.resources.weapon.cooldown > 0.0 {
        return;
    }

    if boomer_world.resources.weapon.ammo == 0 {
        boomer_world.resources.weapon.cooldown = 0.2;
        audio::play(boomer_world, world, audio::EMPTY, 0.6);
        return;
    }

    boomer_world.resources.weapon.ammo -= 1;
    boomer_world.resources.weapon.cooldown = FIRE_COOLDOWN;
    audio::play(boomer_world, world, audio::SHOOT, 1.0);

    let Some((origin, forward, right, up)) = camera_frame(boomer_world, world) else {
        return;
    };

    flash::muzzle(
        boomer_world,
        world,
        origin + forward * 0.8 - up * 0.18 + right * 0.12,
    );

    let targets: Vec<(Entity, Vec3)> = boomer_world
        .query_entities(ENEMY)
        .filter_map(|game_entity| {
            let enemy = boomer_world.get_enemy(game_entity)?;
            if enemy.state == EnemyState::Dying {
                None
            } else {
                Some((game_entity, enemies::center(enemy)))
            }
        })
        .collect();

    let player = boomer_world.resources.player.player_entity;
    for (offset_x, offset_y) in PATTERN {
        let direction = (forward + right * offset_x + up * offset_y).normalize();
        let wall_distance =
            physics_world_cast_ray(&world.resources.physics, origin, direction, RANGE, player)
                .map(|hit| hit.distance)
                .unwrap_or(RANGE);

        let mut best: Option<(Entity, f32)> = None;
        for (game_entity, center) in &targets {
            if let Some(distance) = ray_sphere(origin, direction, *center, enemies::HIT_RADIUS)
                && distance < wall_distance
                && distance < RANGE
                && best.map(|(_, current)| distance < current).unwrap_or(true)
            {
                best = Some((*game_entity, distance));
            }
        }

        if let Some((game_entity, distance)) = best {
            let point = origin + direction * distance;
            enemies::damage(boomer_world, world, game_entity, PELLET_DAMAGE, point);
        }
    }
}

fn camera_frame(boomer_world: &BoomerWorld, world: &World) -> Option<(Vec3, Vec3, Vec3, Vec3)> {
    let camera = boomer_world.resources.player.camera_entity?;
    let transform = world.core.get_global_transform(camera)?;
    Some((
        transform.translation(),
        transform.forward_vector().normalize(),
        transform.right_vector().normalize(),
        transform.up_vector().normalize(),
    ))
}

fn ray_sphere(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let to_center = origin - center;
    let projection = dot(&direction, &to_center);
    let discriminant = projection * projection - (dot(&to_center, &to_center) - radius * radius);
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.sqrt();
    let near = -projection - root;
    if near > 0.0 {
        return Some(near);
    }
    let far = -projection + root;
    if far > 0.0 {
        return Some(far);
    }
    None
}
