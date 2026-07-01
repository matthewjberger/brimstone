use crate::ecs::BrimstoneWorld;
use crate::systems::common::{approach, combo_multiplier};
use crate::systems::world::audio;
use crate::systems::world::level::PLAYER_SPAWN;
use crate::tuning;
use nalgebra_glm::{Vec3, dot, quat_angle_axis, vec3};
use nightshade::ecs::camera::components::{PerspectiveCamera, Projection};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::physics::commands::spawn_first_person_player;
use nightshade::ecs::physics::resources::physics_world_cast_ray;
use nightshade::prelude::*;

const CAMERA_BASE_HEIGHT: f32 = 1.05;
const BOB_FREQUENCY: f32 = 9.0;
const BOB_VERTICAL: f32 = 0.05;
const BOB_HORIZONTAL: f32 = 0.035;
const SHAKE_Z: f32 = 0.6;

pub fn spawn(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let (player, camera) = spawn_first_person_player(world, PLAYER_SPAWN);
    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.engine_input_enabled = false;
        controller.max_speed = tuning::MAX_GROUND_SPEED;
        controller.acceleration = 0.0;
        controller.jump_impulse = tuning::JUMP_IMPULSE;
        controller.friction_rate = 0.0;
        controller.above_max_friction_rate = 0.0;
    }
    if let Some(camera_data) = world.core.get_camera_mut(camera) {
        camera_data.projection = Projection::Perspective(PerspectiveCamera {
            y_fov_rad: tuning::FOV_BASE_DEGREES.to_radians(),
            z_near: 0.01,
            z_far: None,
            aspect_ratio: None,
        });
    }
    brimstone_world.resources.player.player_entity = Some(player);
    brimstone_world.resources.player.camera_entity = Some(camera);
    world.resources.active_camera = Some(camera);
}

pub fn movement(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let Some(player) = brimstone_world.resources.player.player_entity else {
        return;
    };

    let state = &mut brimstone_world.resources.player;
    state.dash_cooldown = (state.dash_cooldown - delta).max(0.0);
    state.iframes = (state.iframes - delta).max(0.0);
    state.wall_run_cooldown = (state.wall_run_cooldown - delta).max(0.0);
    state.wall_run_timer = (state.wall_run_timer - delta).max(0.0);

    let keyboard = &world.resources.input.keyboard;
    let mut forward_input = axis(
        keyboard.is_key_pressed(KeyCode::KeyW),
        keyboard.is_key_pressed(KeyCode::KeyS),
    );
    let mut strafe_input = axis(
        keyboard.is_key_pressed(KeyCode::KeyD),
        keyboard.is_key_pressed(KeyCode::KeyA),
    );
    let grace = brimstone_world.resources.player.spawn_grace > 0;
    if grace {
        brimstone_world.resources.player.spawn_grace -= 1;
    }
    let mut jump = !grace && keyboard.just_pressed(KeyCode::Space);
    let mut dash_pressed = !grace
        && (keyboard.just_pressed(KeyCode::ControlLeft)
            || keyboard.just_pressed(KeyCode::ControlRight));

    if let Some(gamepad) = query_active_gamepad(world) {
        let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let magnitude = (stick_x * stick_x + stick_y * stick_y).sqrt();
        if magnitude > tuning::GAMEPAD_DEADZONE {
            let normalized =
                (magnitude - tuning::GAMEPAD_DEADZONE) / (1.0 - tuning::GAMEPAD_DEADZONE);
            forward_input += stick_y * normalized / magnitude;
            strafe_input += stick_x * normalized / magnitude;
        }
    }
    jump = jump
        || (!grace
            && world
                .resources
                .input
                .gamepad
                .just_pressed_buttons
                .contains(&gilrs::Button::South));
    dash_pressed = dash_pressed
        || (!grace
            && world
                .resources
                .input
                .gamepad
                .just_pressed_buttons
                .contains(&gilrs::Button::East));

    let (forward, right) = camera_basis(brimstone_world, world);
    let mut wishdir = forward * forward_input + right * strafe_input;
    let input_length = wishdir.norm();
    let input_scale = input_length.min(1.0);
    if input_length > 1e-3 {
        wishdir /= input_length;
    } else {
        wishdir = Vec3::zeros();
    }

    let player_position = position(brimstone_world, world);

    let Some(controller) = world.core.get_character_controller(player) else {
        return;
    };
    let grounded = controller.grounded;
    let mut velocity = controller.velocity;

    let dashing = brimstone_world.resources.player.dash_timer > 0.0;
    if dashing {
        brimstone_world.resources.player.dash_timer -= delta;
        let dash_dir = brimstone_world.resources.player.dash_dir;
        velocity.x = dash_dir.x * tuning::DASH_SPEED;
        velocity.z = dash_dir.z * tuning::DASH_SPEED;
    } else if dash_pressed && brimstone_world.resources.player.dash_cooldown <= 0.0 {
        let dash_dir = if wishdir.norm() > 0.1 {
            wishdir
        } else {
            forward
        };
        brimstone_world.resources.player.dash_dir = dash_dir;
        brimstone_world.resources.player.dash_timer = tuning::DASH_TIME;
        brimstone_world.resources.player.dash_cooldown = tuning::DASH_COOLDOWN;
        brimstone_world.resources.player.iframes = tuning::DASH_IFRAMES;
        brimstone_world.resources.game.shake += tuning::DASH_SHAKE;
        audio::play(brimstone_world, world, audio::DASH, 0.7);
        velocity.x = dash_dir.x * tuning::DASH_SPEED;
        velocity.z = dash_dir.z * tuning::DASH_SPEED;
    } else {
        let multiplier = combo_multiplier(brimstone_world.resources.game.combo);
        let ground_speed = tuning::MOVE_SPEED
            * (1.0 + tuning::COMBO_SPEED_PER_STEP * (multiplier.saturating_sub(1)) as f32);
        let mut horizontal = vec3(velocity.x, 0.0, velocity.z);
        let ground_move = grounded && !jump;
        if ground_move {
            horizontal = apply_friction(horizontal, delta);
        }
        let (accel, wishspeed) = if ground_move {
            (tuning::GROUND_ACCEL, ground_speed * input_scale)
        } else {
            (tuning::AIR_ACCEL, tuning::AIR_SPEED_CAP * input_scale)
        };
        horizontal = accelerate(horizontal, wishdir, wishspeed, accel, delta);
        let speed = horizontal.norm();
        if speed > tuning::MAX_GROUND_SPEED {
            horizontal *= tuning::MAX_GROUND_SPEED / speed;
        }
        velocity.x = horizontal.x;
        velocity.z = horizontal.z;
    }

    let prev_wall_side = brimstone_world.resources.player.wall_run_side;
    let wall_jumped = apply_wallrun(
        brimstone_world,
        world,
        &mut velocity,
        &WallrunInput {
            player,
            grounded,
            dashing,
            forward,
            right,
            position: player_position,
            jump,
            delta,
        },
    );
    if prev_wall_side == 0 && brimstone_world.resources.player.wall_run_side != 0 {
        audio::play(brimstone_world, world, audio::DASH, 0.5);
    }

    if grounded && jump && !wall_jumped {
        velocity.y = tuning::JUMP_IMPULSE;
    }

    let launched = grounded && pad_launch(brimstone_world, player_position);
    if launched {
        velocity.y = velocity.y.max(tuning::PAD_IMPULSE);
        brimstone_world.resources.game.shake += tuning::DASH_SHAKE;
        audio::play(brimstone_world, world, audio::PAD, 0.6);
    }

    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.velocity = velocity;
    }
}

/// Accelerate toward `wishdir` only up to `wishspeed` measured *along* that
/// direction — the projection cap that makes strafe-jumping build real speed.
fn accelerate(velocity: Vec3, wishdir: Vec3, wishspeed: f32, accel: f32, delta: f32) -> Vec3 {
    if wishdir.norm() < 1e-3 || wishspeed <= 0.0 {
        return velocity;
    }
    let current_speed = dot(&velocity, &wishdir);
    let add_speed = wishspeed - current_speed;
    if add_speed <= 0.0 {
        return velocity;
    }
    let accel_speed = (accel * wishspeed * delta).min(add_speed);
    velocity + wishdir * accel_speed
}

fn apply_friction(velocity: Vec3, delta: f32) -> Vec3 {
    let speed = velocity.norm();
    if speed < 1e-3 {
        return Vec3::zeros();
    }
    let control = speed.max(tuning::STOP_SPEED);
    let drop = control * tuning::GROUND_FRICTION * delta;
    velocity * (speed - drop).max(0.0) / speed
}

fn pad_launch(brimstone_world: &BrimstoneWorld, player_position: Vec3) -> bool {
    brimstone_world.resources.level.pads.iter().any(|pad| {
        let dx = player_position.x - pad.x;
        let dz = player_position.z - pad.z;
        (dx * dx + dz * dz).sqrt() < tuning::PAD_RADIUS
    })
}

struct WallrunInput {
    player: Entity,
    grounded: bool,
    dashing: bool,
    forward: Vec3,
    right: Vec3,
    position: Vec3,
    jump: bool,
    delta: f32,
}

/// Stick to a wall mid-air and run along it, with a slow controlled fall and an
/// explosive wall-jump off it. Mirrors the nightshade movement demo's wallrun.
fn apply_wallrun(
    brimstone_world: &mut BrimstoneWorld,
    world: &World,
    velocity: &mut Vec3,
    input: &WallrunInput,
) -> bool {
    if input.grounded || input.dashing {
        brimstone_world.resources.player.wall_run_side = 0;
        return false;
    }

    let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();

    if brimstone_world.resources.player.wall_run_side != 0 && input.jump {
        let normal = brimstone_world.resources.player.wall_run_normal;
        let mut flat = vec3(normal.x, 0.0, normal.z);
        flat = if flat.norm() > 1e-3 {
            flat.normalize()
        } else {
            -input.forward
        };
        velocity.x =
            flat.x * tuning::WALL_JUMP_LATERAL + input.forward.x * tuning::WALL_JUMP_FORWARD;
        velocity.z =
            flat.z * tuning::WALL_JUMP_LATERAL + input.forward.z * tuning::WALL_JUMP_FORWARD;
        velocity.y = tuning::WALL_JUMP_VERTICAL;
        let state = &mut brimstone_world.resources.player;
        state.wall_run_side = 0;
        state.wall_run_timer = 0.0;
        state.wall_run_cooldown = tuning::WALL_RUN_COOLDOWN;
        return true;
    }

    if brimstone_world.resources.player.wall_run_cooldown > 0.0 || speed < tuning::WALL_RUN_MIN_SPEED {
        clear_wallrun(brimstone_world);
        return false;
    }

    let origin = input.position + vec3(0.0, 0.3, 0.0);
    let ignore = Some(input.player);
    let (side, normal) = if let Some(normal) = cast_wall(world, origin, input.right, ignore) {
        (1i8, normal)
    } else if let Some(normal) = cast_wall(world, origin, -input.right, ignore) {
        (-1i8, normal)
    } else {
        clear_wallrun(brimstone_world);
        return false;
    };

    let changed = brimstone_world.resources.player.wall_run_side != side;
    if changed {
        brimstone_world.resources.player.wall_run_timer = tuning::WALL_RUN_DURATION;
    }
    brimstone_world.resources.player.wall_run_side = side;
    brimstone_world.resources.player.wall_run_normal = normal;

    if brimstone_world.resources.player.wall_run_timer <= 0.0 {
        clear_wallrun(brimstone_world);
        return false;
    }

    let along = wall_along(normal, input.forward);
    if changed {
        let entry = speed.max(tuning::WALL_RUN_SPEED) + tuning::WALL_RUN_FORWARD_BOOST;
        velocity.x = along.x * entry;
        velocity.z = along.z * entry;
        if velocity.y < 0.0 {
            velocity.y = velocity.y.max(tuning::WALL_RUN_FALL_RATE);
        }
    } else {
        let run = speed.max(tuning::WALL_RUN_SPEED);
        velocity.x = along.x * run - normal.x * tuning::WALL_RUN_STICK * input.delta;
        velocity.z = along.z * run - normal.z * tuning::WALL_RUN_STICK * input.delta;
        velocity.y = tuning::WALL_RUN_FALL_RATE;
    }
    false
}

fn clear_wallrun(brimstone_world: &mut BrimstoneWorld) {
    if brimstone_world.resources.player.wall_run_side != 0 {
        brimstone_world.resources.player.wall_run_side = 0;
        brimstone_world.resources.player.wall_run_cooldown = tuning::WALL_RUN_COOLDOWN;
    }
}

fn cast_wall(world: &World, origin: Vec3, direction: Vec3, ignore: Option<Entity>) -> Option<Vec3> {
    physics_world_cast_ray(
        &world.resources.physics,
        origin,
        direction,
        tuning::WALL_DETECT_DISTANCE,
        ignore,
    )
    .filter(|hit| hit.normal.y.abs() < 0.35 && hit.normal.norm() > 0.1)
    .map(|hit| hit.normal)
}

/// The horizontal direction along a wall (perpendicular to its normal) that
/// points the same way the player is looking.
fn wall_along(normal: Vec3, forward: Vec3) -> Vec3 {
    let mut flat = vec3(normal.x, 0.0, normal.z);
    if flat.norm() < 1e-4 {
        return forward;
    }
    flat = flat.normalize();
    let along = vec3(-flat.z, 0.0, flat.x);
    if dot(&along, &forward) >= dot(&-along, &forward) {
        along
    } else {
        -along
    }
}

/// Strip the wallrun camera roll before the look system reads the rotation, so
/// yaw/pitch stay clean and the roll is re-applied fresh each frame.
pub fn pre_look(brimstone_world: &BrimstoneWorld, world: &mut World) {
    let tilt = brimstone_world.resources.player.wall_run_tilt;
    if tilt.abs() < 1e-5 {
        return;
    }
    let Some(camera) = brimstone_world.resources.player.camera_entity else {
        return;
    };
    if let Some(transform) = world.core.get_local_transform_mut(camera) {
        transform.rotation *= quat_angle_axis(-tilt, &vec3(0.0, 0.0, 1.0));
    }
    world
        .core
        .set_local_transform_dirty(camera, LocalTransformDirty);
}

pub fn apply_camera_feel(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let Some(camera) = brimstone_world.resources.player.camera_entity else {
        return;
    };
    let Some(player) = brimstone_world.resources.player.player_entity else {
        return;
    };

    let active = brimstone_world.resources.player.sim_active;
    let wall_side = brimstone_world.resources.player.wall_run_side;

    let game = &mut brimstone_world.resources.game;
    game.shake = approach(game.shake, 0.0, tuning::SHAKE_DECAY * delta);
    game.cam_kick = approach(game.cam_kick, 0.0, tuning::KICK_DECAY * delta);
    game.fov_pop = approach(game.fov_pop, 0.0, tuning::FOV_POP_DECAY * delta);
    game.damage_flash = (game.damage_flash - delta).max(0.0);
    let shake = game.shake.min(1.4);
    let cam_kick = game.cam_kick;
    let fov_pop = game.fov_pop;

    // Only evolve and re-apply the camera roll while the sim is live, so the
    // roll baked into the rotation always equals what `pre_look` removes.
    let tilt = if active {
        let target_tilt = match wall_side {
            1 => tuning::WALL_RUN_CAMERA_TILT,
            -1 => -tuning::WALL_RUN_CAMERA_TILT,
            _ => 0.0,
        };
        let tilt_lerp = (delta * tuning::WALL_RUN_TILT_LERP).clamp(0.0, 1.0);
        let current_tilt = brimstone_world.resources.player.wall_run_tilt;
        let next = current_tilt + (target_tilt - current_tilt) * tilt_lerp;
        brimstone_world.resources.player.wall_run_tilt = next;
        next
    } else {
        brimstone_world.resources.player.wall_run_tilt
    };
    let wall_fov = if active && wall_side != 0 {
        tuning::WALL_RUN_FOV_POP
    } else {
        0.0
    };

    let elapsed = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
    let speed = world
        .core
        .get_character_controller(player)
        .map(|controller| {
            let velocity = controller.velocity;
            (velocity.x * velocity.x + velocity.z * velocity.z).sqrt()
        })
        .unwrap_or(0.0);
    let grounded = world
        .core
        .get_character_controller(player)
        .map(|controller| controller.grounded)
        .unwrap_or(false);
    let bob_intensity = if grounded {
        (speed / tuning::MOVE_SPEED).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let shake_x = (elapsed * tuning::SHAKE_FREQ_X).sin() * shake * tuning::SHAKE_AMPLITUDE;
    let shake_y = (elapsed * tuning::SHAKE_FREQ_Y).cos() * shake * tuning::SHAKE_AMPLITUDE;
    let shake_z =
        (elapsed * tuning::SHAKE_FREQ_X * 0.7).sin() * shake * tuning::SHAKE_AMPLITUDE * SHAKE_Z;
    let bob_v = (elapsed * BOB_FREQUENCY).sin() * BOB_VERTICAL * bob_intensity;
    let bob_h = (elapsed * BOB_FREQUENCY * 0.5).cos() * BOB_HORIZONTAL * bob_intensity;

    if let Some(transform) = world.core.get_local_transform_mut(camera) {
        transform.translation = vec3(
            shake_x + bob_h,
            CAMERA_BASE_HEIGHT + shake_y + bob_v - cam_kick,
            shake_z,
        );
        if active && tilt.abs() > 1e-5 {
            transform.rotation *= quat_angle_axis(tilt, &vec3(0.0, 0.0, 1.0));
        }
    }
    world
        .core
        .set_local_transform_dirty(camera, LocalTransformDirty);

    if let Some(camera_data) = world.core.get_camera_mut(camera)
        && let Projection::Perspective(ref mut perspective) = camera_data.projection
    {
        perspective.y_fov_rad = (tuning::FOV_BASE_DEGREES + fov_pop + wall_fov).to_radians();
    }
}

pub fn position(brimstone_world: &BrimstoneWorld, world: &World) -> Vec3 {
    brimstone_world
        .resources
        .player
        .player_entity
        .and_then(|player| world.core.get_local_transform(player))
        .map(|transform| transform.translation)
        .unwrap_or(PLAYER_SPAWN)
}

pub fn teleport(brimstone_world: &BrimstoneWorld, world: &mut World, position: Vec3) {
    let Some(player) = brimstone_world.resources.player.player_entity else {
        return;
    };
    if let Some(transform) = world.core.get_local_transform_mut(player) {
        transform.translation = position;
    }
    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.velocity = Vec3::zeros();
    }
    if let Some(interpolation) = world.core.get_physics_interpolation_mut(player) {
        interpolation.previous_translation = position;
        interpolation.current_translation = position;
    }
    mark_local_transform_dirty(world, player);

    if let Some(camera) = brimstone_world.resources.player.camera_entity {
        if let Some(transform) = world.core.get_local_transform_mut(camera) {
            transform.rotation = nalgebra_glm::quat_identity();
        }
        world
            .core
            .set_local_transform_dirty(camera, LocalTransformDirty);
    }
}

fn axis(positive: bool, negative: bool) -> f32 {
    (positive as i32 - negative as i32) as f32
}

fn camera_basis(brimstone_world: &BrimstoneWorld, world: &World) -> (Vec3, Vec3) {
    let camera = brimstone_world.resources.player.camera_entity;
    let Some(transform) = camera.and_then(|camera| world.core.get_global_transform(camera)) else {
        return (vec3(0.0, 0.0, -1.0), vec3(1.0, 0.0, 0.0));
    };
    let mut forward = transform.forward_vector();
    forward.y = 0.0;
    let mut right = transform.right_vector();
    right.y = 0.0;
    let forward = if forward.norm() > 1e-3 {
        forward.normalize()
    } else {
        vec3(0.0, 0.0, -1.0)
    };
    let right = if right.norm() > 1e-3 {
        right.normalize()
    } else {
        vec3(1.0, 0.0, 0.0)
    };
    (forward, right)
}
