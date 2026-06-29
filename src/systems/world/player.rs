use crate::ecs::BoomerWorld;
use crate::systems::world::level::PLAYER_SPAWN;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::physics::commands::spawn_first_person_player;
use nightshade::prelude::*;

const MAX_SPEED: f32 = 9.0;
const SPRINT_MULTIPLIER: f32 = 1.45;
const GROUND_ACCEL: f32 = 13.0;
const AIR_ACCEL: f32 = 4.0;
const GROUND_FRICTION: f32 = 11.0;
const JUMP_IMPULSE: f32 = 6.6;
const GAMEPAD_DEADZONE: f32 = 0.15;

pub fn spawn(boomer_world: &mut BoomerWorld, world: &mut World) {
    let (player, camera) = spawn_first_person_player(world, PLAYER_SPAWN);
    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.engine_input_enabled = false;
        controller.max_speed = MAX_SPEED;
        controller.acceleration = 0.0;
        controller.jump_impulse = JUMP_IMPULSE;
        controller.friction_rate = 0.0;
        controller.above_max_friction_rate = 0.0;
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

    if let Some(gamepad) = query_active_gamepad(world) {
        let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let magnitude = (stick_x * stick_x + stick_y * stick_y).sqrt();
        if magnitude > GAMEPAD_DEADZONE {
            let normalized = (magnitude - GAMEPAD_DEADZONE) / (1.0 - GAMEPAD_DEADZONE);
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

    let (forward, right) = camera_basis(boomer_world, world);
    let mut move_direction = forward * forward_input + right * strafe_input;
    let magnitude = move_direction.norm();
    if magnitude > 1.0 {
        move_direction /= magnitude;
    } else if magnitude <= 1e-3 {
        move_direction = Vec3::zeros();
    }

    let Some(controller) = world.core.get_character_controller(player) else {
        return;
    };
    let grounded = controller.grounded;
    let mut velocity = controller.velocity;

    let target_speed = if sprint {
        MAX_SPEED * SPRINT_MULTIPLIER
    } else {
        MAX_SPEED
    };
    let mut horizontal = vec3(velocity.x, 0.0, velocity.z);
    if move_direction.norm() > 0.01 {
        let accel = if grounded { GROUND_ACCEL } else { AIR_ACCEL };
        let target = move_direction * target_speed;
        horizontal += (target - horizontal) * (accel * delta).min(1.0);
    } else if grounded {
        let friction = (1.0 - GROUND_FRICTION * delta).max(0.0);
        horizontal *= friction;
    }
    velocity.x = horizontal.x;
    velocity.z = horizontal.z;
    if grounded && jump {
        velocity.y = JUMP_IMPULSE;
    }

    if let Some(controller) = world.core.get_character_controller_mut(player) {
        controller.velocity = velocity;
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
