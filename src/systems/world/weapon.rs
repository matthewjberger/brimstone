use crate::ecs::{CobaltWorld, ENEMY, EnemyState, WeaponKind, WeaponState};
use crate::systems::common::random_range;
use crate::systems::world::{audio, enemies, fx, projectiles, viewmodel};
use crate::tuning;
use nalgebra_glm::{Vec3, Vec4, dot, vec3, vec4};
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
        WeaponKind::Pistol => WeaponStats {
            pellets: 1,
            spread: tuning::PISTOL_SPREAD,
            damage: tuning::PISTOL_DAMAGE,
            cooldown: tuning::PISTOL_COOLDOWN,
            knockback: tuning::PISTOL_KNOCKBACK,
            shake: tuning::PISTOL_SHAKE,
            kick: tuning::PISTOL_KICK,
            fov_pop: tuning::PISTOL_FOV_POP,
            tracer: vec4(2.2, 1.9, 1.4, 1.0),
        },
        WeaponKind::Tesla => WeaponStats {
            pellets: 0,
            spread: 0.0,
            damage: tuning::TESLA_TICK_DAMAGE,
            cooldown: tuning::TESLA_TICK,
            knockback: tuning::TESLA_KNOCKBACK,
            shake: tuning::TESLA_SHAKE,
            kick: 0.0,
            fov_pop: 0.0,
            tracer: vec4(0.6, 1.1, 2.8, 1.0),
        },
    }
}

pub fn update(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    cobalt_world.resources.weapon.cooldown =
        (cobalt_world.resources.weapon.cooldown - delta).max(0.0);
    cobalt_world.resources.weapon.hit_marker =
        (cobalt_world.resources.weapon.hit_marker - delta).max(0.0);
    cobalt_world.resources.weapon.recoil =
        (cobalt_world.resources.weapon.recoil - delta * 7.0).max(0.0);

    switch_weapons(cobalt_world, world);
    auto_equip_sidearm(&mut cobalt_world.resources.weapon);

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

    // The tesla is a held beam, not a discrete shot: it runs its own logic every
    // frame (to keep the arc alive and to stop it the instant you release).
    if cobalt_world.resources.weapon.current == WeaponKind::Tesla {
        tesla_beam(cobalt_world, world, firing);
        return;
    }

    if !firing || cobalt_world.resources.weapon.cooldown > 0.0 {
        return;
    }

    let kind = cobalt_world.resources.weapon.current;

    if !kind.infinite() && cobalt_world.resources.weapon.ammo(kind) == 0 {
        cobalt_world.resources.weapon.cooldown = 0.2;
        audio::play(cobalt_world, world, audio::EMPTY, 0.5);
        return;
    }

    let stats = weapon_stats(kind);

    if !kind.infinite() {
        *cobalt_world.resources.weapon.ammo_mut(kind) -= 1;
    }
    cobalt_world.resources.weapon.cooldown = stats.cooldown;
    cobalt_world.resources.weapon.recoil = 1.0;
    cobalt_world.resources.game.shake += stats.shake;
    cobalt_world.resources.game.cam_kick += stats.kick;
    cobalt_world.resources.game.fov_pop = cobalt_world.resources.game.fov_pop.max(stats.fov_pop);
    let (sound, sound_volume) = match kind {
        WeaponKind::Shotgun => (audio::SHOTGUN, 0.9),
        WeaponKind::Nailgun => (audio::NAILGUN, 0.4),
        WeaponKind::Rocket => (audio::ROCKET, 0.85),
        WeaponKind::Railgun => (audio::RAILGUN, 0.8),
        WeaponKind::Pistol => (audio::NAILGUN, 0.5),
        WeaponKind::Tesla => (audio::RAILGUN, 0.7),
    };
    audio::play(cobalt_world, world, sound, sound_volume);

    let Some((origin, forward, right, up)) = camera_frame(cobalt_world, world) else {
        return;
    };
    let muzzle = origin + forward * 0.6 - up * 0.12 + right * 0.12;
    fx::muzzle(
        cobalt_world,
        world,
        muzzle,
        forward,
        vec3(stats.tracer.x, stats.tracer.y, stats.tracer.z),
    );

    if matches!(kind, WeaponKind::Rocket) {
        projectiles::spawn_rocket(cobalt_world, world, muzzle, forward);
        return;
    }

    let targets: Vec<(Entity, Vec3, f32)> = cobalt_world
        .query_entities(ENEMY)
        .filter_map(|game_entity| {
            let enemy = cobalt_world.get_enemy(game_entity)?;
            if enemy.state == EnemyState::Dying {
                None
            } else {
                let (center, radius) = enemies::hit_sphere(enemy);
                Some((game_entity, center, radius))
            }
        })
        .collect();

    let player = cobalt_world.resources.player.player_entity;
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
        for (game_entity, target, radius) in &targets {
            if let Some(distance) = ray_sphere(origin, direction, *target, *radius)
                && distance < wall_distance
                && distance < tuning::WEAPON_RANGE
            {
                hits.push((*game_entity, origin + direction * distance));
            }
        }

        let end = origin + direction * wall_distance;
        fx::tracer(cobalt_world, world, muzzle, end, stats.tracer);
        let hit_anything = !hits.is_empty();
        for (game_entity, point) in hits {
            enemies::damage(
                cobalt_world,
                world,
                game_entity,
                stats.damage,
                point,
                direction * stats.knockback,
            );
        }
        if hit_anything {
            cobalt_world.resources.weapon.hit_marker = 0.12;
            cobalt_world.resources.game.hitstop = cobalt_world
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
                    &mut cobalt_world.resources.game.random_state,
                    -jitter,
                    jitter,
                ),
                random_range(
                    &mut cobalt_world.resources.game.random_state,
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
        for (game_entity, target, radius) in &targets {
            if let Some(distance) = ray_sphere(origin, direction, *target, *radius)
                && distance < wall_distance
                && distance < tuning::WEAPON_RANGE
                && best.map(|(_, current)| distance < current).unwrap_or(true)
            {
                best = Some((*game_entity, distance));
            }
        }

        if let Some((game_entity, distance)) = best {
            let point = origin + direction * distance;
            fx::tracer(cobalt_world, world, muzzle, point, stats.tracer);
            enemies::damage(
                cobalt_world,
                world,
                game_entity,
                stats.damage,
                point,
                direction * stats.knockback,
            );
            connected = true;
        } else {
            let end = origin + direction * wall_distance;
            fx::tracer(cobalt_world, world, muzzle, end, stats.tracer);
        }
    }

    if connected {
        cobalt_world.resources.weapon.hit_marker = 0.12;
        if matches!(kind, WeaponKind::Shotgun) {
            cobalt_world.resources.game.hitstop = cobalt_world
                .resources
                .game
                .hitstop
                .max(tuning::HITSTOP_SHOTGUN);
        }
    }
}

/// Tesla cannon: a press-and-hold beam. While the trigger is held it locks a
/// visible lightning arc onto the nearest enemy ahead and chains onward to the
/// nearest few, redrawing the bolt every frame so it crackles continuously.
/// Damage and ammo are spent in rapid ticks (not every frame) so hit feedback
/// fires at a sane rate; releasing the trigger stops it instantly.
fn tesla_beam(cobalt_world: &mut CobaltWorld, world: &mut World, firing: bool) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    cobalt_world.resources.weapon.tesla_tick =
        (cobalt_world.resources.weapon.tesla_tick - delta).max(0.0);

    if !firing {
        return;
    }

    if cobalt_world.resources.weapon.ammo(WeaponKind::Tesla) == 0 {
        if cobalt_world.resources.weapon.tesla_tick <= 0.0 {
            cobalt_world.resources.weapon.tesla_tick = 0.3;
            audio::play(cobalt_world, world, audio::EMPTY, 0.4);
        }
        return;
    }

    let Some((origin, forward, right, up)) = camera_frame(cobalt_world, world) else {
        return;
    };
    let muzzle = viewmodel::muzzle(cobalt_world, origin, forward, right, up);

    let targets: Vec<(Entity, Vec3)> = cobalt_world
        .query_entities(ENEMY)
        .filter_map(|game_entity| {
            let enemy = cobalt_world.get_enemy(game_entity)?;
            if enemy.state == EnemyState::Dying {
                None
            } else {
                Some((game_entity, enemies::center(enemy)))
            }
        })
        .collect();

    let primary = targets
        .iter()
        .copied()
        .filter(|(_, point)| {
            let to_target = point - origin;
            let distance = to_target.norm();
            distance > 0.1
                && distance < tuning::TESLA_RANGE
                && dot(&(to_target / distance), &forward) > 0.35
        })
        .min_by(|a, b| (a.1 - origin).norm().total_cmp(&(b.1 - origin).norm()));
    let Some(primary) = primary else {
        return;
    };

    let mut hits: Vec<(Entity, Vec3)> = vec![primary];
    let mut current = primary.1;
    while hits.len() <= tuning::TESLA_CHAINS {
        let next = targets
            .iter()
            .copied()
            .filter(|(entity, _)| !hits.iter().any(|(hit, _)| hit == entity))
            .filter(|(_, point)| (point - current).norm() < tuning::TESLA_CHAIN_RANGE)
            .min_by(|a, b| (a.1 - current).norm().total_cmp(&(b.1 - current).norm()));
        let Some(next) = next else { break };
        hits.push(next);
        current = next.1;
    }

    let color = vec3(0.55, 1.0, 2.8);
    let mut from = muzzle;
    for (_, point) in &hits {
        fx::lightning(cobalt_world, world, from, *point, color, 0.06);
        from = *point;
    }
    cobalt_world.resources.weapon.recoil = 0.35;

    if cobalt_world.resources.weapon.tesla_tick > 0.0 {
        return;
    }
    cobalt_world.resources.weapon.tesla_tick = tuning::TESLA_TICK;
    let remaining = cobalt_world
        .resources
        .weapon
        .ammo(WeaponKind::Tesla)
        .saturating_sub(1);
    *cobalt_world.resources.weapon.ammo_mut(WeaponKind::Tesla) = remaining;
    for (entity, point) in &hits {
        enemies::damage(
            cobalt_world,
            world,
            *entity,
            tuning::TESLA_TICK_DAMAGE,
            *point,
            forward * tuning::TESLA_KNOCKBACK,
        );
    }
    cobalt_world.resources.weapon.hit_marker = 0.12;
    cobalt_world.resources.game.shake += tuning::TESLA_SHAKE;
    audio::play(cobalt_world, world, audio::RAILGUN, 0.3);
}

fn switch_weapons(cobalt_world: &mut CobaltWorld, world: &World) {
    let keyboard = &world.resources.input.keyboard;
    let direct = [
        (KeyCode::Digit1, WeaponKind::Shotgun),
        (KeyCode::Digit2, WeaponKind::Nailgun),
        (KeyCode::Digit3, WeaponKind::Rocket),
        (KeyCode::Digit4, WeaponKind::Railgun),
        (KeyCode::Digit5, WeaponKind::Pistol),
        (KeyCode::Digit6, WeaponKind::Tesla),
    ];
    for (key, weapon) in direct {
        if keyboard.just_pressed(key) {
            cobalt_world.resources.weapon.current = weapon;
            return;
        }
    }

    let gamepad = &world.resources.input.gamepad.just_pressed_buttons;
    let scroll = world.resources.input.mouse.wheel_delta.y;
    let current = cobalt_world.resources.weapon.current;
    if gamepad.contains(&gilrs::Button::DPadUp) || scroll > 0.5 {
        cobalt_world.resources.weapon.current = cycle_weapon(current, 1);
    } else if gamepad.contains(&gilrs::Button::DPadDown) || scroll < -0.5 {
        cobalt_world.resources.weapon.current = cycle_weapon(current, -1);
    }
}

/// Step through every weapon (sidearm included) by `step` slots, wrapping around.
fn cycle_weapon(current: WeaponKind, step: i32) -> WeaponKind {
    let count = WeaponKind::ALL.len() as i32;
    let next = (current.index() as i32 + step).rem_euclid(count);
    WeaponKind::ALL[next as usize]
}

/// Keep a usable weapon in hand: if the held gun is empty and so is every other
/// pool, drop to the infinite sidearm so the player can always fight. This is
/// what makes an ammo soft-lock impossible — they never get stuck holding a dead
/// weapon with no fallback.
fn auto_equip_sidearm(weapon: &mut WeaponState) {
    let current = weapon.current;
    if current.infinite() || weapon.ammo(current) > 0 {
        return;
    }
    let all_dry = WeaponKind::ALL
        .iter()
        .filter(|kind| !kind.infinite())
        .all(|&kind| weapon.ammo(kind) == 0);
    if all_dry {
        weapon.current = WeaponKind::Pistol;
    }
}

fn camera_frame(cobalt_world: &CobaltWorld, world: &World) -> Option<(Vec3, Vec3, Vec3, Vec3)> {
    let camera = cobalt_world.resources.player.camera_entity?;
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
