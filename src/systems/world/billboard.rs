use crate::ecs::{CobaltWorld, ENEMY, ENGINE_ENTITY, PICKUP};
use crate::systems::world::textures::BILLBOARD_MESH;
use nalgebra_glm::{Vec3, quat_angle_axis, vec3};
use nightshade::prelude::*;

pub fn spawn(world: &mut World, material: &str, position: Vec3, scale: Vec3) -> Entity {
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY,
        1,
    )[0];

    world.core.set_name(entity, Name("Billboard".to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: nalgebra_glm::quat_identity(),
            scale,
        },
    );
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_render_mesh(entity, RenderMesh::new(BILLBOARD_MESH));
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material.to_string()));
    world
        .core
        .set_bounding_volume(entity, BoundingVolume::from_mesh_type("Cube"));
    world
        .core
        .set_visibility(entity, Visibility { visible: true });
    world.resources.mesh_render_state.mark_entity_added(entity);
    entity
}

pub fn camera_position(cobalt_world: &CobaltWorld, world: &World) -> Vec3 {
    cobalt_world
        .resources
        .player
        .camera_entity
        .and_then(|camera| world.core.get_global_transform(camera))
        .map(|transform| transform.translation())
        .unwrap_or_else(|| vec3(0.0, 1.6, 0.0))
}

pub fn face(world: &mut World, entity: Entity, position: Vec3, camera_position: Vec3) {
    let mut direction = camera_position - position;
    direction.y = 0.0;
    let yaw = if direction.norm() > 1e-4 {
        direction.x.atan2(direction.z)
    } else {
        0.0
    };
    let rotation = quat_angle_axis(yaw, &vec3(0.0, 1.0, 0.0));
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.rotation = rotation;
    }
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
}

pub fn update(cobalt_world: &mut CobaltWorld, world: &mut World) {
    let camera = camera_position(cobalt_world, world);

    let enemies: Vec<(Entity, Vec3)> = cobalt_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine = cobalt_world.get_engine_entity(game_entity)?.0;
            let position = cobalt_world.get_enemy(game_entity)?.position;
            Some((engine, position))
        })
        .collect();
    for (engine, position) in enemies {
        face(world, engine, position, camera);
    }

    let pickups: Vec<(Entity, Vec3)> = cobalt_world
        .query_entities(PICKUP | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine = cobalt_world.get_engine_entity(game_entity)?.0;
            let position = cobalt_world.get_pickup(game_entity)?.position;
            Some((engine, position))
        })
        .collect();
    for (engine, position) in pickups {
        face(world, engine, position, camera);
    }

    let projectiles: Vec<(Entity, Vec3)> = cobalt_world
        .resources
        .projectiles
        .items
        .iter()
        .map(|projectile| (projectile.entity, projectile.position))
        .collect();
    for (entity, position) in projectiles {
        face(world, entity, position, camera);
    }
}
