use crate::ecs::BoomerWorld;
use crate::systems::common::approach;
use crate::systems::world::audio;
use crate::systems::world::level::PLAYER_SPAWN;
use crate::tuning;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::camera::components::{PerspectiveCamera, Projection};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::physics::commands::spawn_first_person_player;
use nightshade::prelude::*;

const CAMERA_BASE_HEIGHT: f32 = 1.3;
const BOB_FREQUENCY: f32 = 9.0;
const BOB_VERTICAL: f32 = 0.05;
const BOB_HORIZONTAL: f32 = 0.035;
const SHAKE_Z: f32 = 0.6;

pub fn spawn(boomer_world: &mut BoomerWorld, world: &mut World) {
    let (player, camera) = spawn_first_person_player(world, PLAYER_SPAWN);
    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.engine_input_enabled = false;
        controller.max_speed = tuning::MOVE_SPEED;
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
    boomer_world.resources.player.player_entity = Some(player);
    boomer_world.resources.player.camera_entity = Some(camera);
    world.resources.active_camera = Some(camera);
}

pub fn movement(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let Some(player) = boomer_world.resources.player.player_entity else {
        return;
    };

    let state = &mut boomer_world.resources.player;
    state.dash_cooldown = (state.dash_cooldown - delta).max(0.0);
    state.iframes = (state.iframes - delta).max(0.0);

    let keyboard = &world.resources.input.keyboard;
    let mut forward_input = axis(
        keyboard.is_key_pressed(KeyCode::KeyW),
        keyboard.is_key_pressed(KeyCode::KeyS),
    );
    let mut strafe_input = axis(
        keyboard.is_key_pressed(KeyCode::KeyD),
        keyboard.is_key_pressed(KeyCode::KeyA),
    );
    let mut jump = keyboard.just_pressed(KeyCode::Space);
    let mut sprint =
        keyboard.is_key_pressed(KeyCode::ShiftLeft) || keyboard.is_key_pressed(KeyCode::ShiftRight);
    let mut dash_pressed =
        keyboard.just_pressed(KeyCode::ControlLeft) || keyboard.just_pressed(KeyCode::ControlRight);

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
        sprint = sprint || gamepad.is_pressed(gilrs::Button::LeftTrigger);
    }
    jump = jump
        || world
            .resources
            .input
            .gamepad
            .just_pressed_buttons
            .contains(&gilrs::Button::South);
    dash_pressed = dash_pressed
        || world
            .resources
            .input
            .gamepad
            .just_pressed_buttons
            .contains(&gilrs::Button::East);

    let (forward, right) = camera_basis(boomer_world, world);
    let mut move_direction = forward * forward_input + right * strafe_input;
    let input_magnitude = move_direction.norm();
    if input_magnitude > 1.0 {
        move_direction /= input_magnitude;
    } else if input_magnitude <= 1e-3 {
        move_direction = Vec3::zeros();
    }

    let Some(controller) = world.core.get_character_controller(player) else {
        return;
    };
    let grounded = controller.grounded;
    let mut velocity = controller.velocity;

    let dashing = boomer_world.resources.player.dash_timer > 0.0;
    if dashing {
        boomer_world.resources.player.dash_timer -= delta;
        let dash_dir = boomer_world.resources.player.dash_dir;
        velocity.x = dash_dir.x * tuning::DASH_SPEED;
        velocity.z = dash_dir.z * tuning::DASH_SPEED;
    } else if dash_pressed && boomer_world.resources.player.dash_cooldown <= 0.0 {
        let dash_dir = if move_direction.norm() > 0.1 {
            move_direction
        } else {
            forward
        };
        boomer_world.resources.player.dash_dir = dash_dir;
        boomer_world.resources.player.dash_timer = tuning::DASH_TIME;
        boomer_world.resources.player.dash_cooldown = tuning::DASH_COOLDOWN;
        boomer_world.resources.player.iframes = tuning::DASH_IFRAMES;
        boomer_world.resources.game.shake += tuning::DASH_SHAKE;
        audio::play(boomer_world, world, audio::DASH, 0.7);
        velocity.x = dash_dir.x * tuning::DASH_SPEED;
        velocity.z = dash_dir.z * tuning::DASH_SPEED;
    } else {
        let target_speed = if sprint {
            tuning::MOVE_SPEED * tuning::SPRINT_MULTIPLIER
        } else {
            tuning::MOVE_SPEED
        };
        let mut horizontal = vec3(velocity.x, 0.0, velocity.z);
        if move_direction.norm() > 0.01 {
            let accel = if grounded {
                tuning::GROUND_ACCEL
            } else {
                tuning::AIR_ACCEL
            };
            let target = move_direction * target_speed;
            horizontal += (target - horizontal) * (accel * delta).min(1.0);
        } else if grounded {
            let friction = (1.0 - tuning::GROUND_FRICTION * delta).max(0.0);
            horizontal *= friction;
        }
        velocity.x = horizontal.x;
        velocity.z = horizontal.z;
    }

    if grounded && jump {
        velocity.y = tuning::JUMP_IMPULSE;
    }

    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.velocity = velocity;
    }
}

pub fn apply_camera_feel(boomer_world: &mut BoomerWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    let Some(camera) = boomer_world.resources.player.camera_entity else {
        return;
    };
    let Some(player) = boomer_world.resources.player.player_entity else {
        return;
    };

    let game = &mut boomer_world.resources.game;
    game.shake = approach(game.shake, 0.0, tuning::SHAKE_DECAY * delta);
    game.cam_kick = approach(game.cam_kick, 0.0, tuning::KICK_DECAY * delta);
    game.fov_pop = approach(game.fov_pop, 0.0, tuning::FOV_POP_DECAY * delta);
    game.damage_flash = (game.damage_flash - delta).max(0.0);
    let shake = game.shake.min(1.4);
    let cam_kick = game.cam_kick;
    let fov_pop = game.fov_pop;

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
    }
    world
        .core
        .set_local_transform_dirty(camera, LocalTransformDirty);

    if let Some(camera_data) = world.core.get_camera_mut(camera)
        && let Projection::Perspective(ref mut perspective) = camera_data.projection
    {
        perspective.y_fov_rad = (tuning::FOV_BASE_DEGREES + fov_pop).to_radians();
    }
}

pub fn position(boomer_world: &BoomerWorld, world: &World) -> Vec3 {
    boomer_world
        .resources
        .player
        .player_entity
        .and_then(|player| world.core.get_local_transform(player))
        .map(|transform| transform.translation)
        .unwrap_or(PLAYER_SPAWN)
}

pub fn reset(boomer_world: &BoomerWorld, world: &mut World) {
    let Some(player) = boomer_world.resources.player.player_entity else {
        return;
    };
    if let Some(transform) = world.core.get_local_transform_mut(player) {
        transform.translation = PLAYER_SPAWN;
    }
    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.velocity = Vec3::zeros();
    }
    if let Some(interpolation) = world.core.get_physics_interpolation_mut(player) {
        interpolation.previous_translation = PLAYER_SPAWN;
        interpolation.current_translation = PLAYER_SPAWN;
    }
    mark_local_transform_dirty(world, player);
}

fn axis(positive: bool, negative: bool) -> f32 {
    (positive as i32 - negative as i32) as f32
}

fn camera_basis(boomer_world: &BoomerWorld, world: &World) -> (Vec3, Vec3) {
    let camera = boomer_world.resources.player.camera_entity;
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
