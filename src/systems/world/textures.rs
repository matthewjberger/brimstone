use crate::art;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::loading::{load_texture_pack_from_image_bytes, queue_decoded_texture};
use nightshade::ecs::material::components::{AlphaMode, Material, TextureTransform};
use nightshade::ecs::material::material_registry_insert;
use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::{
    SamplerSettings, TextureUsage, texture_cache_protect,
};

pub const FLOOR_TEXTURE: &str = "boom_floor";
pub const WALL_TEXTURE: &str = "boom_wall";
pub const PLATFORM_TEXTURE: &str = "boom_platform";
pub const PILLAR_TEXTURE: &str = "boom_pillar";
pub const ACCENT_TEXTURE: &str = "boom_accent";

pub const MAT_IMP_IDLE: &str = "boom_mat_imp_idle";
pub const MAT_IMP_HURT: &str = "boom_mat_imp_hurt";
pub const MAT_SWARM_IDLE: &str = "boom_mat_swarm_idle";
pub const MAT_SWARM_HURT: &str = "boom_mat_swarm_hurt";
pub const MAT_CASTER_IDLE: &str = "boom_mat_caster_idle";
pub const MAT_CASTER_HURT: &str = "boom_mat_caster_hurt";
pub const MAT_FIREBALL: &str = "boom_mat_fireball";
pub const MAT_MEDKIT: &str = "boom_mat_medkit";
pub const MAT_AMMO: &str = "boom_mat_ammo";
pub const MAT_EXIT: &str = "boom_mat_exit";

pub const BILLBOARD_MESH: &str = "boom_billboard";

const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[
    (
        FLOOR_TEXTURE,
        include_bytes!("../../../assets/textures/prototype/dark/texture_08.png") as &[u8],
    ),
    (
        WALL_TEXTURE,
        include_bytes!("../../../assets/textures/prototype/red/texture_05.png") as &[u8],
    ),
    (
        PLATFORM_TEXTURE,
        include_bytes!("../../../assets/textures/prototype/light/texture_06.png") as &[u8],
    ),
    (
        PILLAR_TEXTURE,
        include_bytes!("../../../assets/textures/prototype/purple/texture_01.png") as &[u8],
    ),
    (
        ACCENT_TEXTURE,
        include_bytes!("../../../assets/textures/prototype/green/texture_06.png") as &[u8],
    ),
];

pub fn load(world: &mut World) {
    load_texture_pack_from_image_bytes(
        world,
        PROTOTYPE_TEXTURES,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );

    upload_sprite(world, "boom_imp_idle", art::imp_idle());
    upload_sprite(world, "boom_imp_hurt", art::imp_hurt());
    upload_sprite(world, "boom_swarm_idle", art::swarmer_idle());
    upload_sprite(world, "boom_swarm_hurt", art::swarmer_hurt());
    upload_sprite(world, "boom_caster_idle", art::caster_idle());
    upload_sprite(world, "boom_caster_hurt", art::caster_hurt());
    upload_sprite(world, "boom_fireball", art::fireball());
    upload_sprite(world, "boom_medkit", art::medkit());
    upload_sprite(world, "boom_ammo", art::ammo_box());

    register_material(world, MAT_IMP_IDLE, sprite_material("boom_imp_idle"));
    register_material(world, MAT_IMP_HURT, hurt_material("boom_imp_hurt"));
    register_material(world, MAT_SWARM_IDLE, sprite_material("boom_swarm_idle"));
    register_material(world, MAT_SWARM_HURT, hurt_material("boom_swarm_hurt"));
    register_material(world, MAT_CASTER_IDLE, sprite_material("boom_caster_idle"));
    register_material(world, MAT_CASTER_HURT, hurt_material("boom_caster_hurt"));
    register_material(world, MAT_FIREBALL, glow_material("boom_fireball"));
    register_material(world, MAT_MEDKIT, sprite_material("boom_medkit"));
    register_material(world, MAT_AMMO, sprite_material("boom_ammo"));
    register_material(world, MAT_EXIT, beacon_material(vec3(0.3, 1.8, 0.7), 5.0));

    register_billboard_mesh(world);
}

fn upload_sprite(world: &mut World, name: &str, sprite: art::Sprite) {
    queue_decoded_texture(
        world,
        name.to_string(),
        sprite.rgba,
        sprite.width,
        sprite.height,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
    texture_cache_protect(&mut world.resources.texture_cache, name.to_string());
}

fn register_material(world: &mut World, name: &str, material: Material) {
    material_registry_insert(
        &mut world.resources.assets.material_registry,
        name.to_string(),
        material,
    );
    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
        let protected = &mut world.resources.assets.material_registry.protected_indices;
        if !protected.contains(&index) {
            protected.push(index);
        }
    }
}

pub fn floor_material() -> Material {
    proto_material(FLOOR_TEXTURE, vec3(0.42, 0.40, 0.48), 0.92, 0.04, 6.0)
}

pub fn wall_material() -> Material {
    proto_material(WALL_TEXTURE, vec3(0.62, 0.34, 0.34), 0.86, 0.06, 4.0)
}

pub fn platform_material() -> Material {
    proto_material(PLATFORM_TEXTURE, vec3(0.55, 0.58, 0.66), 0.78, 0.08, 3.0)
}

pub fn pillar_material() -> Material {
    proto_material(PILLAR_TEXTURE, vec3(0.46, 0.34, 0.62), 0.74, 0.10, 2.5)
}

pub fn accent_material() -> Material {
    proto_material(ACCENT_TEXTURE, vec3(0.40, 0.60, 0.46), 0.7, 0.1, 2.0)
}

/// Solid emissive material for glowing landmark beacons.
pub fn beacon_material(color: Vec3, strength: f32) -> Material {
    Material {
        base_color: [color.x, color.y, color.z, 1.0],
        emissive_factor: [color.x, color.y, color.z],
        emissive_strength: strength,
        roughness: 0.35,
        metallic: 0.0,
        ..Default::default()
    }
}

fn proto_material(
    texture: &str,
    tint: Vec3,
    roughness: f32,
    metallic: f32,
    tiling: f32,
) -> Material {
    Material {
        base_color: [tint.x, tint.y, tint.z, 1.0],
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: [tiling, tiling],
            ..Default::default()
        },
        roughness,
        metallic,
        ..Default::default()
    }
}

/// Crisp unlit sprite. Reads as flat pixel art, no bloom smear.
fn sprite_material(texture: &str) -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_texture: Some(texture.to_string()),
        alpha_mode: AlphaMode::Mask,
        alpha_cutoff: 0.5,
        unlit: true,
        double_sided: true,
        ..Default::default()
    }
}

/// Hurt frame gets a light emissive lift so the hit flash pops for its brief
/// window, without the idle sprites glowing all the time.
fn hurt_material(texture: &str) -> Material {
    Material {
        emissive_texture: Some(texture.to_string()),
        emissive_factor: [1.0, 1.0, 1.0],
        emissive_strength: 1.4,
        ..sprite_material(texture)
    }
}

fn glow_material(texture: &str) -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_texture: Some(texture.to_string()),
        emissive_texture: Some(texture.to_string()),
        emissive_factor: [1.0, 0.6, 0.3],
        emissive_strength: 5.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        ..Default::default()
    }
}

fn register_billboard_mesh(world: &mut World) {
    let normal = vec3(0.0, 0.0, 1.0);
    let vertices = vec![
        Vertex::with_tex_coords(vec3(-0.5, 1.0, 0.0), normal, [0.0, 0.0]),
        Vertex::with_tex_coords(vec3(0.5, 1.0, 0.0), normal, [1.0, 0.0]),
        Vertex::with_tex_coords(vec3(0.5, 0.0, 0.0), normal, [1.0, 1.0]),
        Vertex::with_tex_coords(vec3(-0.5, 0.0, 0.0), normal, [0.0, 1.0]),
    ];
    let indices = vec![0, 2, 1, 0, 3, 2];
    let mesh = Mesh::new(vertices, indices);
    mesh_cache_insert(
        &mut world.resources.assets.mesh_cache,
        BILLBOARD_MESH.to_string(),
        mesh,
    );
}
