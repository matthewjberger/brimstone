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

pub const MAT_FIREBALL: &str = "boom_mat_fireball";
pub const MAT_ROCKET: &str = "boom_mat_rocket";
pub const MAT_MEDKIT: &str = "boom_mat_medkit";
pub const MAT_AMMO: &str = "boom_mat_ammo";
pub const MAT_KEYCARD: &str = "boom_mat_keycard";
pub const MAT_EXIT: &str = "boom_mat_exit";
pub const PAD_MATERIAL: &str = "boom_mat_pad";
pub const MARKER_PLAYER: &str = "boom_mat_marker_player";
pub const MARKER_ENEMY: &str = "boom_mat_marker_enemy";
pub const MAT_GHOST: &str = "boom_mat_ghost";

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

    register_animated(world, "imp", art::imp_idle(), art::imp_hurt());
    register_animated(world, "swarm", art::swarmer_idle(), art::swarmer_hurt());
    register_animated(world, "caster", art::caster_idle(), art::caster_hurt());
    register_animated(world, "brute", art::brute_idle(), art::brute_hurt());
    register_animated(
        world,
        "gargoyle",
        art::gargoyle_idle(),
        art::gargoyle_hurt(),
    );

    upload_sprite(world, "boom_fireball", art::fireball());
    upload_sprite(world, "boom_rocket", art::rocket());
    upload_sprite(world, "boom_medkit", art::medkit());
    upload_sprite(world, "boom_ammo", art::ammo_box());
    upload_sprite(world, "boom_keycard", art::keycard());

    register_material(world, MAT_FIREBALL, glow_material("boom_fireball"));
    register_material(
        world,
        MAT_ROCKET,
        glow_material_tinted("boom_rocket", [0.4, 0.7, 1.0]),
    );
    register_material(world, MAT_MEDKIT, sprite_material("boom_medkit"));
    register_material(world, MAT_AMMO, sprite_material("boom_ammo"));
    register_material(
        world,
        MAT_KEYCARD,
        glow_material_tinted("boom_keycard", [1.0, 0.85, 0.2]),
    );
    register_material(world, MAT_EXIT, beacon_material(vec3(0.3, 1.8, 0.7), 5.0));
    register_material(
        world,
        PAD_MATERIAL,
        beacon_material(vec3(0.3, 1.4, 1.7), 4.0),
    );
    register_material(
        world,
        MARKER_PLAYER,
        beacon_material(vec3(0.3, 1.7, 0.5), 4.0),
    );
    register_material(
        world,
        MARKER_ENEMY,
        beacon_material(vec3(1.7, 0.3, 0.3), 4.0),
    );
    register_material(world, MAT_GHOST, ghost_material());

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

/// Register an enemy's animated frames: each frame gets a normal sprite
/// material and an emissive `_e` variant for elites, plus one shared hurt flash.
fn register_animated(world: &mut World, key: &str, base: art::Sprite, hurt: art::Sprite) {
    for index in 0..art::ANIM_FRAMES {
        let texture = format!("boom_{key}_f{index}");
        upload_sprite(world, &texture, art::frame(&base, index));
        register_material(
            world,
            &format!("boom_mat_{key}_f{index}"),
            sprite_material(&texture),
        );
        register_material(
            world,
            &format!("boom_mat_{key}_f{index}_e"),
            hurt_material(&texture),
        );
    }
    let hurt_texture = format!("boom_{key}_hurt");
    upload_sprite(world, &hurt_texture, hurt);
    register_material(
        world,
        &format!("boom_mat_{key}_hurt"),
        hurt_material(&hurt_texture),
    );
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

/// Translucent preview material for the editor's placement ghost.
fn ghost_material() -> Material {
    Material {
        base_color: [0.4, 0.8, 1.0, 0.35],
        emissive_factor: [0.3, 0.6, 0.9],
        emissive_strength: 1.2,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        ..Default::default()
    }
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
    glow_material_tinted(texture, [1.0, 0.6, 0.3])
}

fn glow_material_tinted(texture: &str, emissive: [f32; 3]) -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_texture: Some(texture.to_string()),
        emissive_texture: Some(texture.to_string()),
        emissive_factor: emissive,
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
