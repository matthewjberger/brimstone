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

pub const FLOOR_TEXTURE: &str = "boomer_floor";
pub const WALL_TEXTURE: &str = "boomer_wall";

pub const MAT_IMP_IDLE: &str = "boomer_mat_imp_idle";
pub const MAT_IMP_ATTACK: &str = "boomer_mat_imp_attack";
pub const MAT_IMP_HURT: &str = "boomer_mat_imp_hurt";
pub const MAT_MEDKIT: &str = "boomer_mat_medkit";
pub const MAT_AMMO: &str = "boomer_mat_ammo";
pub const MAT_MUZZLE: &str = "boomer_mat_muzzle";
pub const MAT_SPARK: &str = "boomer_mat_spark";

pub const BILLBOARD_MESH: &str = "boomer_billboard";

const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[
    (
        FLOOR_TEXTURE,
        include_bytes!("../../../assets/textures/prototype/dark/texture_08.png") as &[u8],
    ),
    (
        WALL_TEXTURE,
        include_bytes!("../../../assets/textures/prototype/red/texture_05.png") as &[u8],
    ),
];

pub fn load(world: &mut World) {
    load_texture_pack_from_image_bytes(
        world,
        PROTOTYPE_TEXTURES,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );

    upload_sprite(world, "boomer_imp_idle", art::imp_idle());
    upload_sprite(world, "boomer_imp_attack", art::imp_attack());
    upload_sprite(world, "boomer_imp_hurt", art::imp_hurt());
    upload_sprite(world, "boomer_medkit", art::medkit());
    upload_sprite(world, "boomer_ammo", art::ammo_box());
    upload_sprite(world, "boomer_muzzle", art::muzzle());
    upload_sprite(world, "boomer_spark", art::spark());

    register_material(world, MAT_IMP_IDLE, sprite_material("boomer_imp_idle"));
    register_material(world, MAT_IMP_ATTACK, sprite_material("boomer_imp_attack"));
    register_material(world, MAT_IMP_HURT, sprite_material("boomer_imp_hurt"));
    register_material(world, MAT_MEDKIT, sprite_material("boomer_medkit"));
    register_material(world, MAT_AMMO, sprite_material("boomer_ammo"));
    register_material(world, MAT_MUZZLE, glow_material("boomer_muzzle"));
    register_material(world, MAT_SPARK, glow_material("boomer_spark"));

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
    }
}

pub fn floor_material() -> Material {
    proto_material(FLOOR_TEXTURE, vec3(0.60, 0.58, 0.64), 0.92, 0.02, 10.0)
}

pub fn wall_material() -> Material {
    proto_material(WALL_TEXTURE, vec3(0.74, 0.46, 0.44), 0.88, 0.04, 4.0)
}

pub fn pillar_material() -> Material {
    proto_material(WALL_TEXTURE, vec3(0.46, 0.30, 0.30), 0.80, 0.05, 2.0)
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

fn glow_material(texture: &str) -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_texture: Some(texture.to_string()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        emissive_factor: [1.0, 0.85, 0.5],
        emissive_strength: 3.0,
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
