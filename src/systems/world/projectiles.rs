use crate::ecs::{CobaltWorld, ENEMY, EnemyState, Projectile};
use crate::systems::world::textures::{MAT_FIREBALL, MAT_ROCKET};
use crate::systems::world::{audio, billboard, enemies, fx, game, player};
use crate::tuning;
use nalgebra_glm::{Vec3, dot, vec3};
use nightshade::ecs::physics::resources::physics_world_cast_ray;
use nightshade::ecs::world::commands::{EcsCommand, queue_ecs_command};
use nightshade::prelude::*;

const PLAYER_HIT_PADDING: f32 = 0.55;

struct Blast {
    position: Vec3,
    hostile: bool,
    splash_radius: f32,
    direct: Option<Entity>,
    hit_player: bool,
    damage: f32,
}

pub fn spawn(
    cobalt_world: &mut CobaltWorld,
    world: &mut World,
    origin: Vec3,
    target: Vec3,
    damage: f32,
) {
    let mut direction = target - origin;
    if direction.norm() < 1e-3 {
        direction = vec3(0.0, 0.0, 1.0);
    }
    let velocity = direction.normalize() * tuning::FIREBALL_SPEED;
    let entity = billboard::spawn(
        world,
        MAT_FIREBALL,
        origin,
        vec3(tuning::FIREBALL_SCALE, tuning::FIREBALL_SCALE, 1.0),
    );
    cobalt_world.resources.projectiles.items.push(Projectile {
        entity,
        position: origin,
        velocity,
        lifetime: tuning::FIREBALL_LIFETIME,
        damage,
        hostile: true,
        splash_radius: 0.0,
    });
}

pub fn spawn_rocket(
    cobalt_world: &mut CobaltWorld,
    world: &mut World,
    origin: Vec3,
    forward: Vec3,
) {
    let mut direction = forward;
    if direction.norm() < 1e-3 {
        direction = vec3(0.0, 0.0, 1.0);
    }
    let velocity = direction.normalize() * tuning::ROCKET_SPEED;
    let entity = billboard::spawn(
        world,
        MAT_ROCKET,
        origin,
        vec3(tuning::ROCKET_SCALE, tuning::ROCKET_SCALE, 1.0),
    );
    cobalt_world.resources.projectiles.items.push(Projectile {
        entity,
        position: origin,
        velocity,
        lifetime: tuning::ROCKET_LIFETIME,
        damage: tuning::ROCKET_DAMAGE,
        hostile: false,
        splash_radius: tuning::ROCKET_SPLASH_RADIUS,
    });
}

pub fn update(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let player_entity = cobalt_world.resources.player.player_entity;
    let player_center = player::position(cobalt_world, world);

    let enemy_targets: Vec<(Entity, Vec3)> = cobalt_world
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

    let mut items = std::mem::take(&mut cobalt_world.resources.projectiles.items);
    let mut removed: Vec<(Entity, Blast)> = Vec::new();

    items.retain_mut(|item| {
        item.lifetime -= delta;
        let start = item.position;
        let travel = item.velocity * delta;
        let step = travel.norm().max(1e-4);
        let direction = item.velocity / item.velocity.norm().max(1e-4);
        let next = start + travel;
        item.position = next;

        let blast = |position, direct, hit_player| Blast {
            position,
            hostile: item.hostile,
            splash_radius: item.splash_radius,
            direct,
            hit_player,
            damage: item.damage,
        };

        if item.lifetime <= 0.0 {
            removed.push((item.entity, blast(next, None, false)));
            return false;
        }

        if item.hostile {
            let to_player = player_center - next;
            if to_player.norm() < tuning::FIREBALL_RADIUS + PLAYER_HIT_PADDING {
                removed.push((item.entity, blast(next, None, true)));
                return false;
            }
        } else {
            for (enemy_entity, center) in &enemy_targets {
                if let Some(distance) =
                    ray_sphere(start, direction, *center, tuning::ENEMY_HIT_RADIUS)
                    && distance <= step
                {
                    let point = start + direction * distance;
                    removed.push((item.entity, blast(point, Some(*enemy_entity), false)));
                    return false;
                }
            }
        }

        if let Some(hit) = physics_world_cast_ray(
            &world.resources.physics,
            start,
            direction,
            step,
            player_entity,
        ) && hit.distance <= step
        {
            removed.push((item.entity, blast(hit.point, None, false)));
            return false;
        }

        true
    });

    cobalt_world.resources.projectiles.items = items;

    for (entity, blast) in removed {
        queue_ecs_command(world, EcsCommand::DespawnRecursive { entity });
        if blast.hostile {
            fx::hit(cobalt_world, world, blast.position, vec3(1.0, 0.5, 0.2));
            if blast.hit_player {
                game::damage_player(cobalt_world, world, blast.damage);
            }
        } else {
            explode(cobalt_world, world, &blast);
        }
    }
}

fn explode(cobalt_world: &mut CobaltWorld, world: &mut World, blast: &Blast) {
    let radius = blast.splash_radius.max(0.1);
    fx::death(
        cobalt_world,
        world,
        blast.position,
        vec3(0.5, 0.75, 1.0),
        80,
    );
    fx::hit(cobalt_world, world, blast.position, vec3(0.6, 0.85, 1.0));
    cobalt_world.resources.game.shake += tuning::ROCKET_SHAKE;
    cobalt_world.resources.game.hitstop = cobalt_world
        .resources
        .game
        .hitstop
        .max(tuning::ROCKET_HITSTOP);
    audio::play(cobalt_world, world, audio::EXPLOSION, 1.0);

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

    for (enemy_entity, center) in targets {
        let mut offset = center - blast.position;
        let distance = offset.norm();
        if distance >= radius {
            continue;
        }
        let falloff = (1.0 - distance / radius).clamp(0.0, 1.0);
        let mut damage = tuning::ROCKET_SPLASH_DAMAGE * falloff;
        if blast.direct == Some(enemy_entity) {
            damage += tuning::ROCKET_DAMAGE;
        }
        if offset.norm() < 1e-3 {
            offset = vec3(0.0, 1.0, 0.0);
        }
        let knockback = offset.normalize() * tuning::ROCKET_KNOCKBACK * falloff;
        enemies::damage(cobalt_world, world, enemy_entity, damage, center, knockback);
    }

    rocket_jump(cobalt_world, world, blast.position, radius);
}

/// Catch yourself in the blast and ride it — the genre-defining rocket-jump.
fn rocket_jump(cobalt_world: &mut CobaltWorld, world: &mut World, position: Vec3, radius: f32) {
    let Some(player) = cobalt_world.resources.player.player_entity else {
        return;
    };
    let player_center = player::position(cobalt_world, world);
    let mut away = player_center - position;
    let distance = away.norm();
    if distance >= radius {
        return;
    }
    let falloff = (1.0 - distance / radius).clamp(0.0, 1.0);
    if away.norm() < 1e-3 {
        away = vec3(0.0, 1.0, 0.0);
    }
    away = away.normalize();
    away.y = away.y.max(0.45);
    away = away.normalize();
    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.velocity += away * tuning::ROCKET_SELF_PUSH * falloff;
    }
    game::damage_player(cobalt_world, world, tuning::ROCKET_SELF_DAMAGE * falloff);
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

pub fn despawn_all(cobalt_world: &mut CobaltWorld, world: &mut World) {
    for projectile in cobalt_world.resources.projectiles.items.drain(..) {
        queue_ecs_command(
            world,
            EcsCommand::DespawnRecursive {
                entity: projectile.entity,
            },
        );
    }
}
