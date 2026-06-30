use crate::ecs::{BoomerWorld, ENEMY, EnemyState, WeaponKind};
use crate::systems::common::random_range;
use crate::systems::world::{audio, enemies, fx, projectiles};
use crate::tuning;
use nalgebra_glm::{Vec3, Vec4, dot, vec4};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::physics::resources::physics_world_cast_ray;
use nightshade::prelude::*;

struct WeaponStats {
    pellets: u32,
    spread: f32,
    damage: f32,
    cooldown: f32,
    knockback: f32,
    shake: f32,
    kick: f32,
    fov_pop: f32,
    tracer: Vec4,
}

fn weapon_stats(kind: WeaponKind) -> WeaponStats {
    match kind {
        WeaponKind::Shotgun => WeaponStats {
            pellets: tuning::SHOTGUN_PELLETS,
            spread: tuning::SHOTGUN_SPREAD,
            damage: tuning::SHOTGUN_DAMAGE,
            cooldown: tuning::SHOTGUN_COOLDOWN,
            knockback: tuning::SHOTGUN_KNOCKBACK,
            shake: tuning::SHOTGUN_SHAKE,
            kick: tuning::SHOTGUN_KICK,
            fov_pop: tuning::SHOTGUN_FOV_POP,
            tracer: vec4(2.4, 2.0, 1.1, 1.0),
        },
        WeaponKind::Nailgun => WeaponStats {
            pellets: 1,
            spread: tuning::NAIL_SPREAD,
            damage: tuning::NAIL_DAMAGE,
            cooldown: tuning::NAIL_COOLDOWN,
            knockback: tuning::NAIL_KNOCKBACK,
            shake: tuning::NAIL_SHAKE,
            kick: tuning::NAIL_KICK,
            fov_pop: tuning::NAIL_FOV_POP,
            tracer: vec4(1.0, 2.2, 2.6, 1.0),
        },
        WeaponKind::Rocket => WeaponStats {
            pellets: 0,
            spread: 0.0,
            damage: tuning::ROCKET_DAMAGE,
            cooldown: tuning::ROCKET_COOLDOWN,
            knockback: tuning::ROCKET_KNOCKBACK,
            shake: tuning::ROCKET_SHAKE,
            kick: tuning::ROCKET_KICK,
            fov_pop: tuning::ROCKET_FOV_POP,
            tracer: vec4(0.4, 0.7, 1.0, 1.0),
        },
        WeaponKind::Railgun => WeaponStats {
            pellets: 1,
            spread: 0.0,
            damage: tuning::RAIL_DAMAGE,
            cooldown: tuning::RAIL_COOLDOWN,
            knockback: tuning::RAIL_KNOCKBACK,
            shake: tuning::RAIL_SHAKE,
            kick: tuning::RAIL_KICK,
            fov_pop: tuning::RAIL_FOV_POP,
            tracer: vec4(2.6, 1.2, 2.8, 1.0),
        },
    }
}

pub fn update(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    boomer_world.resources.weapon.cooldown =
        (boomer_world.resources.weapon.cooldown - delta).max(0.0);
    boomer_world.resources.weapon.hit_marker =
        (boomer_world.resources.weapon.hit_marker - delta).max(0.0);

    switch_weapons(boomer_world, world);

    let mouse_fire = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::LEFT_CLICKED);
    let gamepad_fire = query_active_gamepad(world)
        .map(|gamepad| gamepad.is_pressed(gilrs::Button::RightTrigger2))
        .unwrap_or(false);
    let firing = mouse_fire || gamepad_fire;

    if !firing || boomer_world.resources.weapon.cooldown > 0.0 {
        return;
    }

    let kind = boomer_world.resources.weapon.current;

    if boomer_world.resources.weapon.ammo(kind) == 0 {
        boomer_world.resources.weapon.cooldown = 0.2;
        audio::play(boomer_world, world, audio::EMPTY, 0.5);
        return;
    }

    let stats = weapon_stats(kind);

    *boomer_world.resources.weapon.ammo_mut(kind) -= 1;
    boomer_world.resources.weapon.cooldown = stats.cooldown;
    boomer_world.resources.game.shake += stats.shake;
    boomer_world.resources.game.cam_kick += stats.kick;
    boomer_world.resources.game.fov_pop = boomer_world.resources.game.fov_pop.max(stats.fov_pop);
    let (sound, sound_volume) = match kind {
        WeaponKind::Shotgun => (audio::SHOTGUN, 0.9),
        WeaponKind::Nailgun => (audio::NAILGUN, 0.4),
        WeaponKind::Rocket => (audio::ROCKET, 0.85),
        WeaponKind::Railgun => (audio::RAILGUN, 0.8),
    };
    audio::play(boomer_world, world, sound, sound_volume);

    let Some((origin, forward, right, up)) = camera_frame(boomer_world, world) else {
        return;
    };
    let muzzle = origin + forward * 0.6 - up * 0.12 + right * 0.12;
    fx::muzzle(boomer_world, world, muzzle, forward);

    if matches!(kind, WeaponKind::Rocket) {
        projectiles::spawn_rocket(boomer_world, world, muzzle, forward);
        return;
    }

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
    let mut connected = false;

    if matches!(kind, WeaponKind::Railgun) {
        let direction = forward;
        let wall_distance = physics_world_cast_ray(
            &world.resources.physics,
            origin,
            direction,
            tuning::WEAPON_RANGE,
            player,
        )
        .map(|hit| hit.distance)
        .unwrap_or(tuning::WEAPON_RANGE);

        let mut hits: Vec<(Entity, Vec3)> = Vec::new();
        for (game_entity, target) in &targets {
            if let Some(distance) = ray_sphere(origin, direction, *target, tuning::ENEMY_HIT_RADIUS)
                && distance < wall_distance
                && distance < tuning::WEAPON_RANGE
            {
                hits.push((*game_entity, origin + direction * distance));
            }
        }

        let end = origin + direction * wall_distance;
        fx::tracer(boomer_world, world, muzzle, end, stats.tracer);
        let hit_anything = !hits.is_empty();
        for (game_entity, point) in hits {
            enemies::damage(
                boomer_world,
                world,
                game_entity,
                stats.damage,
                point,
                direction * stats.knockback,
            );
        }
        if hit_anything {
            boomer_world.resources.weapon.hit_marker = 0.12;
            boomer_world.resources.game.hitstop = boomer_world
                .resources
                .game
                .hitstop
                .max(tuning::RAIL_HITSTOP);
        }
        return;
    }

    for pellet in 0..stats.pellets {
        let (offset_x, offset_y) = if stats.pellets > 1 {
            let angle = pellet as f32 * 2.399_963_3 + 0.7;
            let radius = ((pellet as f32 + 1.0) / stats.pellets as f32).sqrt() * stats.spread;
            (radius * angle.cos(), radius * angle.sin())
        } else {
            let jitter = stats.spread;
            (
                random_range(
                    &mut boomer_world.resources.game.random_state,
                    -jitter,
                    jitter,
                ),
                random_range(
                    &mut boomer_world.resources.game.random_state,
                    -jitter,
                    jitter,
                ),
            )
        };
        let direction = (forward + right * offset_x + up * offset_y).normalize();

        let wall_distance = physics_world_cast_ray(
            &world.resources.physics,
            origin,
            direction,
            tuning::WEAPON_RANGE,
            player,
        )
        .map(|hit| hit.distance)
        .unwrap_or(tuning::WEAPON_RANGE);

        let mut best: Option<(Entity, f32)> = None;
        for (game_entity, target) in &targets {
            if let Some(distance) = ray_sphere(origin, direction, *target, tuning::ENEMY_HIT_RADIUS)
                && distance < wall_distance
                && distance < tuning::WEAPON_RANGE
                && best.map(|(_, current)| distance < current).unwrap_or(true)
            {
                best = Some((*game_entity, distance));
            }
        }

        if let Some((game_entity, distance)) = best {
            let point = origin + direction * distance;
            fx::tracer(boomer_world, world, muzzle, point, stats.tracer);
            enemies::damage(
                boomer_world,
                world,
                game_entity,
                stats.damage,
                point,
                direction * stats.knockback,
            );
            connected = true;
        } else {
            let end = origin + direction * wall_distance;
            fx::tracer(boomer_world, world, muzzle, end, stats.tracer);
        }
    }

    if connected {
        boomer_world.resources.weapon.hit_marker = 0.12;
        if matches!(kind, WeaponKind::Shotgun) {
            boomer_world.resources.game.hitstop = boomer_world
                .resources
                .game
                .hitstop
                .max(tuning::HITSTOP_SHOTGUN);
        }
    }
}

fn switch_weapons(boomer_world: &mut BoomerWorld, world: &World) {
    let keyboard = &world.resources.input.keyboard;
    let dpad_up = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::DPadUp);
    let dpad_down = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::DPadDown);
    if keyboard.just_pressed(KeyCode::Digit1) {
        boomer_world.resources.weapon.current = WeaponKind::Shotgun;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        boomer_world.resources.weapon.current = WeaponKind::Nailgun;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        boomer_world.resources.weapon.current = WeaponKind::Rocket;
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        boomer_world.resources.weapon.current = WeaponKind::Railgun;
    } else if dpad_up {
        boomer_world.resources.weapon.current = match boomer_world.resources.weapon.current {
            WeaponKind::Shotgun => WeaponKind::Nailgun,
            WeaponKind::Nailgun => WeaponKind::Rocket,
            WeaponKind::Rocket => WeaponKind::Railgun,
            WeaponKind::Railgun => WeaponKind::Shotgun,
        };
    } else if dpad_down {
        boomer_world.resources.weapon.current = match boomer_world.resources.weapon.current {
            WeaponKind::Shotgun => WeaponKind::Railgun,
            WeaponKind::Nailgun => WeaponKind::Shotgun,
            WeaponKind::Rocket => WeaponKind::Nailgun,
            WeaponKind::Railgun => WeaponKind::Rocket,
        };
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
