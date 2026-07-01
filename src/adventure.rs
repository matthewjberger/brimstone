//! Adventure mode: a free-roam RPG layer over the shooter. You wander a small
//! overworld of linked areas — a town of wandering billboard NPCs, the wilds, and
//! a dungeon — talk to quest-givers and a merchant, take on kill-quests, fight out
//! in the wilds with the full arsenal, and turn quests in for gold and loot.
//!
//! The world is built from the same block geometry, billboards, navmesh, and
//! combat systems as the arcade; this module owns the area data, the NPC
//! schedules, the dialogue / shop / inventory / quest-log screens, and the loop
//! that ties them together. NPCs live in [`AdventureState`] rather than the ECS so
//! the combat component world stays untouched.

use crate::content::BlockKind::{Core, Cover, Monument, Pillar, Wall};
use crate::content::BlockSpec;
use crate::ecs::{
    AdvPanel, AdventureHandles, AdventureNpc, AdventurePortal, BrimstoneWorld, EnemyKind,
    Interactable, Phase, QuestProgress, QuestState, Screen,
};
use crate::systems::common::{next_random, random_range};
use crate::systems::lifecycle;
use crate::systems::world::textures::{
    MAT_EXIT, MAT_NPC_ELDER, MAT_NPC_GUARD, MAT_NPC_MERCHANT, MAT_NPC_VILLAGER,
};
use crate::systems::world::{
    audio, billboard, enemies, fx, game, level, overworld, pickups, player, projectiles, scatter,
    weapon,
};
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::camera::systems::first_person_camera_look_system;
use nightshade::prelude::*;

const INTERACT_RANGE: f32 = 3.4;
const NPC_SPEED: f32 = 1.8;
const NPC_SCALE: Vec3 = Vec3::new(1.9, 2.5, 1.0);
const ENEMY_RESPAWN: f32 = 4.0;
const POTION_HEAL: f32 = 55.0;

// ============================================================================
// Content
// ============================================================================

struct ItemDef {
    name: &'static str,
    price: u32,
    sold: bool,
}

const ITEM_POTION: usize = 0;
const ITEM_HOLLOW_SIGIL: usize = 1;

const ITEMS: &[ItemDef] = &[
    ItemDef {
        name: "Health Draught",
        price: 18,
        sold: true,
    },
    ItemDef {
        name: "Hollow Sigil",
        price: 0,
        sold: false,
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum NpcRole {
    Villager,
    Merchant,
    QuestGiver,
}

struct NpcDef {
    name: &'static str,
    material: &'static str,
    role: NpcRole,
    quest: Option<usize>,
    line: &'static str,
}

const NPC_ELDER: usize = 0;
const NPC_CAPTAIN: usize = 1;
const NPC_MERCHANT: usize = 2;
const NPC_VILLAGER: usize = 3;
const NPC_FARMER: usize = 4;

const NPCS: &[NpcDef] = &[
    NpcDef {
        name: "Elder Maru",
        material: MAT_NPC_ELDER,
        role: NpcRole::QuestGiver,
        quest: Some(QUEST_CULL),
        line: "Rivermoor bleeds, traveler. The Mistfen crawls with fiends.",
    },
    NpcDef {
        name: "Captain Brann",
        material: MAT_NPC_GUARD,
        role: NpcRole::QuestGiver,
        quest: Some(QUEST_HOLLOW),
        line: "Something old stirs in Ember Hollow. I'd not go alone.",
    },
    NpcDef {
        name: "Merchant Vex",
        material: MAT_NPC_MERCHANT,
        role: NpcRole::Merchant,
        quest: None,
        line: "Coin for cures, friend? You'll want them where you're headed.",
    },
    NpcDef {
        name: "Villager",
        material: MAT_NPC_VILLAGER,
        role: NpcRole::Villager,
        quest: None,
        line: "Stay inside the walls after dusk. That's my advice.",
    },
    NpcDef {
        name: "Farmer Cole",
        material: MAT_NPC_VILLAGER,
        role: NpcRole::Villager,
        quest: None,
        line: "Lost three sheep to the fen this week. Three!",
    },
];

#[derive(Clone, Copy)]
enum QuestObjective {
    Kill { area: usize, count: u32 },
    Boss { area: usize },
}

struct QuestDef {
    title: &'static str,
    summary: &'static str,
    objective: QuestObjective,
    reward_gold: u32,
    reward_item: Option<usize>,
}

const QUEST_CULL: usize = 0;
const QUEST_HOLLOW: usize = 1;

const QUESTS: &[QuestDef] = &[
    QuestDef {
        title: "Culling the Fen",
        summary: "Slay 8 fiends out in the Mistfen.",
        objective: QuestObjective::Kill {
            area: AREA_WILDS,
            count: 8,
        },
        reward_gold: 70,
        reward_item: Some(ITEM_POTION),
    },
    QuestDef {
        title: "The Hollow's Heart",
        summary: "Descend into Ember Hollow and destroy the Warlord within.",
        objective: QuestObjective::Boss { area: AREA_HOLLOW },
        reward_gold: 220,
        reward_item: Some(ITEM_HOLLOW_SIGIL),
    },
];

type NpcSpawn = (usize, f32, f32);
type PortalSpec = (f32, f32, usize, &'static str);

struct AreaDef {
    name: &'static str,
    atmosphere: Atmosphere,
    fog: [f32; 3],
    half_x: f32,
    half_z: f32,
    spawn: [f32; 3],
    blocks: &'static [BlockSpec],
    lights: &'static [(f32, f32, [f32; 3])],
    npcs: &'static [NpcSpawn],
    portals: &'static [PortalSpec],
    enemies: &'static [EnemyKind],
    enemy_cap: usize,
    elite: bool,
    /// When true this area is the streamed open-world overworld (terrain, no walls)
    /// rather than a bounded arena cell.
    overworld: bool,
}

const AREA_TOWN: usize = 0;
const AREA_WILDS: usize = 1;
const AREA_HOLLOW: usize = 2;

const TOWN_BLOCKS: &[BlockSpec] = &[
    (-18.0, 3.0, -12.0, 9.0, 6.0, 9.0, Monument),
    (17.0, 3.0, -14.0, 10.0, 6.0, 8.0, Monument),
    (-21.0, 3.5, 14.0, 8.0, 7.0, 10.0, Wall),
    (19.0, 3.5, 16.0, 9.0, 7.0, 9.0, Monument),
    (0.0, 1.6, 0.0, 3.0, 3.2, 3.0, Core),
    (-7.0, 0.6, 9.0, 2.2, 1.2, 2.2, Cover),
    (9.0, 0.6, -5.0, 2.2, 1.2, 2.2, Cover),
];
const TOWN_LIGHTS: &[(f32, f32, [f32; 3])] = &[
    (0.0, 0.0, [1.8, 1.2, 0.5]),
    (-12.0, 6.0, [1.2, 0.9, 0.6]),
    (12.0, -8.0, [1.2, 0.9, 0.6]),
    (0.0, -36.0, [0.4, 1.4, 0.7]),
];
const TOWN_NPCS: &[NpcSpawn] = &[
    (NPC_ELDER, 2.0, -3.0),
    (NPC_MERCHANT, -6.0, 2.0),
    (NPC_CAPTAIN, 0.0, -30.0),
    (NPC_VILLAGER, 8.0, 6.0),
    (NPC_FARMER, -9.0, -6.0),
];
const TOWN_PORTALS: &[PortalSpec] = &[(0.0, -38.0, AREA_WILDS, "the Mistfen")];

const WILDS_BLOCKS: &[BlockSpec] = &[
    (-14.0, 2.0, -10.0, 4.0, 4.0, 4.0, Pillar),
    (13.0, 2.6, 9.0, 5.0, 5.2, 5.0, Monument),
    (0.0, 1.4, -18.0, 6.0, 2.8, 4.0, Cover),
    (22.0, 2.0, 22.0, 4.0, 4.0, 4.0, Pillar),
    (-24.0, 2.2, 17.0, 5.0, 4.4, 5.0, Monument),
    (8.0, 0.9, 2.0, 3.0, 1.8, 3.0, Cover),
    (-20.0, 2.0, -20.0, 4.0, 4.0, 4.0, Pillar),
];
const WILDS_LIGHTS: &[(f32, f32, [f32; 3])] = &[
    (0.0, 40.0, [0.4, 1.3, 0.7]),
    (38.0, 0.0, [1.6, 0.5, 0.4]),
    (-10.0, -10.0, [0.5, 0.6, 1.2]),
];
const WILDS_PORTALS: &[PortalSpec] = &[
    (0.0, 42.0, AREA_TOWN, "Rivermoor"),
    (40.0, 0.0, AREA_HOLLOW, "Ember Hollow"),
];
const WILDS_ENEMIES: &[EnemyKind] = &[
    EnemyKind::Imp,
    EnemyKind::Swarmer,
    EnemyKind::Caster,
    EnemyKind::Gargoyle,
];

const HOLLOW_BLOCKS: &[BlockSpec] = &[
    (0.0, 3.0, 0.0, 4.0, 6.0, 4.0, Core),
    (-11.0, 2.6, -9.0, 3.0, 5.2, 3.0, Pillar),
    (11.0, 2.6, -9.0, 3.0, 5.2, 3.0, Pillar),
    (-11.0, 2.6, 9.0, 3.0, 5.2, 3.0, Pillar),
    (11.0, 2.6, 9.0, 3.0, 5.2, 3.0, Pillar),
    (0.0, 1.0, -18.0, 8.0, 2.0, 3.0, Cover),
];
const HOLLOW_LIGHTS: &[(f32, f32, [f32; 3])] =
    &[(0.0, 0.0, [2.0, 0.7, 0.25]), (0.0, 24.0, [0.4, 1.2, 0.7])];
const HOLLOW_PORTALS: &[PortalSpec] = &[(0.0, 26.0, AREA_WILDS, "the Mistfen")];

/// Dungeon entrances scattered across the overworld terrain around town, each a
/// gate into a bounded arena cell. Seated on the ground once the terrain beneath
/// them streams in. (x, z, target area, label).
const OVERWORLD_POIS: &[PortalSpec] = &[
    (200.0, 120.0, AREA_WILDS, "the Mistfen"),
    (-180.0, 150.0, AREA_HOLLOW, "Ember Hollow"),
    (40.0, 240.0, AREA_WILDS, "the Fen Ruin"),
    (-150.0, -190.0, AREA_HOLLOW, "the Sunken Barrow"),
    (230.0, -110.0, AREA_WILDS, "the Reedmire"),
    (-330.0, -50.0, AREA_HOLLOW, "the Ashen Spire"),
    (120.0, -320.0, AREA_WILDS, "the Drowned Hold"),
    (-70.0, 400.0, AREA_HOLLOW, "the Frost Cairn"),
];
const HOLLOW_ENEMIES: &[EnemyKind] = &[
    EnemyKind::Brute,
    EnemyKind::Caster,
    EnemyKind::Gargoyle,
    EnemyKind::Imp,
];

const AREAS: &[AreaDef] = &[
    AreaDef {
        name: "RIVERMOOR",
        atmosphere: Atmosphere::Sunset,
        fog: [0.10, 0.07, 0.05],
        half_x: 42.0,
        half_z: 42.0,
        spawn: [0.0, 3.0, 30.0],
        blocks: TOWN_BLOCKS,
        lights: TOWN_LIGHTS,
        npcs: TOWN_NPCS,
        portals: TOWN_PORTALS,
        enemies: &[],
        enemy_cap: 0,
        elite: false,
        overworld: true,
    },
    AreaDef {
        name: "THE MISTFEN",
        atmosphere: Atmosphere::Nebula,
        fog: [0.05, 0.08, 0.06],
        half_x: 46.0,
        half_z: 46.0,
        spawn: [0.0, 1.2, 38.0],
        blocks: WILDS_BLOCKS,
        lights: WILDS_LIGHTS,
        npcs: &[],
        portals: WILDS_PORTALS,
        enemies: WILDS_ENEMIES,
        enemy_cap: 8,
        elite: false,
        overworld: false,
    },
    AreaDef {
        name: "EMBER HOLLOW",
        atmosphere: Atmosphere::Space,
        fog: [0.10, 0.03, 0.03],
        half_x: 30.0,
        half_z: 30.0,
        spawn: [0.0, 1.2, 22.0],
        blocks: HOLLOW_BLOCKS,
        lights: HOLLOW_LIGHTS,
        npcs: &[],
        portals: HOLLOW_PORTALS,
        enemies: HOLLOW_ENEMIES,
        enemy_cap: 7,
        elite: true,
        overworld: false,
    },
];

// ============================================================================
// Lifecycle
// ============================================================================

/// Begin a fresh adventure run from the title screen.
pub fn open(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let uptime = world.resources.window.timing.uptime_milliseconds;
    let adventure = &mut brimstone_world.resources.adventure;
    *adventure = Default::default();
    adventure.active = true;
    adventure.rng = 0x2545_f491_4f6c_dd1d ^ (uptime | 1);
    adventure.gold = 30;
    adventure.items.push((ITEM_POTION, 2));

    brimstone_world.resources.stats = Default::default();
    brimstone_world.resources.weapon = Default::default();
    brimstone_world.resources.game.phase = Phase::Playing;

    // Clear the arcade level so its geometry doesn't coexist with the overworld.
    game::teardown_world(brimstone_world, world);
    load_area(brimstone_world, world, AREA_TOWN);
    brimstone_world.resources.adventure.intro_done = true;
    brimstone_world
        .resources
        .adventure
        .notify("Rivermoor endures, but the Mistfen festers. Seek Elder Maru.");
    lifecycle::enter(brimstone_world, world, Screen::Adventure);
}

/// Leave adventure mode back to the title screen, restoring the arcade world so
/// the other modes are ready again.
pub fn leave(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    teardown(brimstone_world, world);
    overworld::leave(world);
    brimstone_world.resources.adventure.active = false;
    game::start_at(brimstone_world, world, 0);
    lifecycle::enter(brimstone_world, world, Screen::Title);
}

fn teardown(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    enemies::despawn_all(brimstone_world, world);
    pickups::despawn_all(brimstone_world, world);
    projectiles::despawn_all(brimstone_world, world);
    scatter::clear(&mut brimstone_world.resources.adventure.scatter, world);
    let npcs = std::mem::take(&mut brimstone_world.resources.adventure.npcs);
    for npc in npcs {
        despawn_recursive_immediate(world, npc.entity);
    }
    let geometry = std::mem::take(&mut brimstone_world.resources.adventure.geometry);
    for entity in geometry {
        despawn_recursive_immediate(world, entity);
    }
    brimstone_world.resources.adventure.pending_portals.clear();
}

fn load_area(brimstone_world: &mut BrimstoneWorld, world: &mut World, area_index: usize) {
    teardown(brimstone_world, world);
    let area = &AREAS[area_index];

    level::apply_environment(world, area.atmosphere, area.fog);
    let mut geometry = if area.overworld {
        overworld::enter(
            world,
            area.blocks,
            vec3(area.spawn[0], area.spawn[1], area.spawn[2]),
        )
    } else {
        overworld::leave(world);
        level::build_arena(world, area.blocks, area.half_x, area.half_z)
    };
    for (x, z, color) in area.lights {
        geometry.push(level::spawn_accent_light(
            world,
            vec3(*x, 3.0, *z),
            vec3(color[0], color[1], color[2]),
        ));
    }

    let mut npcs: Vec<AdventureNpc> = Vec::new();
    for (kind, x, z) in area.npcs {
        let home = vec3(*x, 0.0, *z);
        let entity = billboard::spawn(world, NPCS[*kind].material, home, NPC_SCALE);
        npcs.push(AdventureNpc {
            kind: *kind,
            entity,
            position: home,
            home,
            target: home,
            wait: 1.0,
            anim: 0.0,
            shown: 0,
        });
    }

    let portal_color = vec3(0.3, 1.7, 0.7);
    let mut portals: Vec<AdventurePortal> = Vec::new();
    let mut pending_portals: Vec<(f32, f32, usize, String)> = Vec::new();
    if area.overworld {
        // Seat portals on the streamed terrain lazily, once each one's ground
        // height is known (see `spawn_ready_portals`).
        for (x, z, target, label) in area.portals.iter().chain(OVERWORLD_POIS) {
            pending_portals.push((*x, *z, *target, label.to_string()));
        }
    } else {
        for (x, z, target, label) in area.portals {
            let position = vec3(*x, 0.0, *z);
            // A real stretched cube, not a billboard, so the gate stays solid from
            // every side, with a light and a column of rising embers for presence.
            geometry.push(level::spawn_marker(
                world,
                vec3(*x, 2.4, *z),
                vec3(2.8, 4.8, 2.8),
                MAT_EXIT,
            ));
            geometry.push(level::spawn_accent_light(
                world,
                vec3(*x, 2.6, *z),
                portal_color,
            ));
            geometry.push(level::spawn_embers(world, vec3(*x, 0.3, *z), portal_color));
            portals.push(AdventurePortal {
                position,
                target_area: *target,
                label: label.to_string(),
            });
        }
    }

    let spawn = vec3(area.spawn[0], area.spawn[1], area.spawn[2]);
    player::teleport(brimstone_world, world, spawn);

    let adventure = &mut brimstone_world.resources.adventure;
    adventure.area = area_index;
    adventure.npcs = npcs;
    adventure.portals = portals;
    adventure.pending_portals = pending_portals;
    adventure.geometry = geometry;
    adventure.spawn_point = spawn;
    adventure.banner = 3.0;
    adventure.enemy_timer = 1.5;
    adventure.interactable = Interactable::None;
    adventure.panel = AdvPanel::None;
    adventure.last_kills = brimstone_world.resources.game.kills;
    adventure.boss_active = false;

    if !area.enemies.is_empty() {
        for _ in 0..area.enemy_cap.min(4) {
            spawn_wild_enemy(brimstone_world, world);
        }
    }
    maybe_spawn_boss(brimstone_world, world, area_index);
}

/// Seat any overworld portal whose terrain has streamed in: place its gate,
/// light, and embers on the ground and register it for interaction. Pending
/// entries whose ground is not known yet are retried on later frames.
fn spawn_ready_portals(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if brimstone_world
        .resources
        .adventure
        .pending_portals
        .is_empty()
    {
        return;
    }
    let portal_color = vec3(0.3, 1.7, 0.7);
    let mut pending = std::mem::take(&mut brimstone_world.resources.adventure.pending_portals);
    pending.retain(|(x, z, target, label)| {
        let Some(ground) = world.resources.terrain_render.sample_height(*x, *z) else {
            return true;
        };
        let mut parts = level::spawn_poi_landmark(world, vec3(*x, ground, *z), portal_color);
        let adventure = &mut brimstone_world.resources.adventure;
        adventure.geometry.append(&mut parts);
        adventure.portals.push(AdventurePortal {
            position: vec3(*x, ground, *z),
            target_area: *target,
            label: label.clone(),
        });
        false
    });
    brimstone_world.resources.adventure.pending_portals = pending;
}

/// Spawn the area's warlord set-piece if a boss objective for it is active.
fn maybe_spawn_boss(brimstone_world: &mut BrimstoneWorld, world: &mut World, area_index: usize) {
    let wants_boss = brimstone_world
        .resources
        .adventure
        .quests
        .iter()
        .any(|progress| {
            progress.state == QuestState::Active
                && matches!(
                    QUESTS[progress.quest].objective,
                    QuestObjective::Boss { area } if area == area_index
                )
        });
    if !wants_boss {
        return;
    }
    enemies::spawn(
        brimstone_world,
        world,
        EnemyKind::Brute,
        true,
        true,
        vec3(0.0, 0.0, -10.0),
    );
    brimstone_world.resources.adventure.boss_active = true;
    brimstone_world
        .resources
        .adventure
        .notify("The Warlord of Ember Hollow stirs...");
}

// ============================================================================
// Per-frame update
// ============================================================================

pub fn update(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if !matches!(brimstone_world.resources.screen.current, Screen::Adventure) {
        return;
    }
    if AREAS[brimstone_world.resources.adventure.area].overworld {
        let center = player::position(brimstone_world, world);
        overworld::update(world, center);
        scatter::update(
            &mut brimstone_world.resources.adventure.scatter,
            world,
            center,
        );
        spawn_ready_portals(brimstone_world, world);
    }
    let delta = world.resources.window.timing.delta_time.clamp(0.0, 0.1);
    tick_timers(brimstone_world, delta);

    if brimstone_world.resources.adventure.panel != AdvPanel::None {
        handle_panel_input(brimstone_world, world);
        fx::tick(brimstone_world, world);
        update_ui(brimstone_world, world);
        return;
    }

    brimstone_world.resources.player.sim_active = true;
    player::pre_look(brimstone_world, world);
    first_person_camera_look_system(world);
    player::movement(brimstone_world, world);
    weapon::update(brimstone_world, world);
    enemies::update(brimstone_world, world);
    projectiles::update(brimstone_world, world);
    pickups::update(brimstone_world, world);
    player::apply_camera_feel(brimstone_world, world);
    billboard::update(brimstone_world, world);
    fx::tick(brimstone_world, world);
    update_vfx_system(world);
    crate::systems::world::viewmodel::update(brimstone_world, world);

    update_npcs(brimstone_world, world, delta);
    maintain_enemies(brimstone_world, world, delta);
    credit_kills(brimstone_world);
    check_boss(brimstone_world);
    detect_interactable(brimstone_world, world);
    handle_interact_key(brimstone_world, world);
    handle_panel_keys(brimstone_world, world);
    revive_if_dead(brimstone_world, world);

    update_ui(brimstone_world, world);
}

fn tick_timers(brimstone_world: &mut BrimstoneWorld, delta: f32) {
    let adventure = &mut brimstone_world.resources.adventure;
    adventure.banner = (adventure.banner - delta).max(0.0);
    adventure.notice_timer = (adventure.notice_timer - delta).max(0.0);
}

fn update_npcs(brimstone_world: &mut BrimstoneWorld, world: &mut World, delta: f32) {
    let camera = billboard::camera_position(brimstone_world, world);
    let adventure = &mut brimstone_world.resources.adventure;
    let rng = &mut adventure.rng;
    for npc in adventure.npcs.iter_mut() {
        let mut to_target = npc.target - npc.position;
        to_target.y = 0.0;
        if to_target.norm() > 0.35 {
            npc.position += to_target.normalize() * NPC_SPEED * delta;
        } else {
            npc.wait -= delta;
            if npc.wait <= 0.0 {
                let angle = random_range(rng, 0.0, std::f32::consts::TAU);
                let distance = random_range(rng, 1.5, 6.5);
                npc.target = npc.home + vec3(angle.cos() * distance, 0.0, angle.sin() * distance);
                npc.wait = random_range(rng, 2.0, 5.0);
            }
        }
        billboard::face(world, npc.entity, npc.position, camera);
    }
}

fn maintain_enemies(brimstone_world: &mut BrimstoneWorld, world: &mut World, delta: f32) {
    let area = &AREAS[brimstone_world.resources.adventure.area];
    if area.enemies.is_empty() {
        return;
    }
    brimstone_world.resources.adventure.enemy_timer -= delta;
    if brimstone_world.resources.adventure.enemy_timer > 0.0 {
        return;
    }
    brimstone_world.resources.adventure.enemy_timer = ENEMY_RESPAWN;
    if enemies::total_count(brimstone_world) < area.enemy_cap {
        spawn_wild_enemy(brimstone_world, world);
    }
}

fn spawn_wild_enemy(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let area = &AREAS[brimstone_world.resources.adventure.area];
    if area.enemies.is_empty() {
        return;
    }
    let rng = &mut brimstone_world.resources.adventure.rng;
    let pick =
        (random_range(rng, 0.0, area.enemies.len() as f32) as usize).min(area.enemies.len() - 1);
    let kind = area.enemies[pick];
    let bound_x = area.half_x - 4.0;
    let bound_z = area.half_z - 4.0;
    let x = random_range(rng, -bound_x, bound_x);
    let z = random_range(rng, -bound_z, bound_z);
    let elite = area.elite && next_random(rng) < 0.3;
    enemies::spawn(brimstone_world, world, kind, elite, false, vec3(x, 0.0, z));
}

/// Credit kill-quest progress from the global kill counter delta.
fn credit_kills(brimstone_world: &mut BrimstoneWorld) {
    let kills = brimstone_world.resources.game.kills;
    let adventure = &mut brimstone_world.resources.adventure;
    let delta = kills.saturating_sub(adventure.last_kills);
    adventure.last_kills = kills;
    if delta == 0 {
        return;
    }
    let area = adventure.area;
    let mut completed: Vec<usize> = Vec::new();
    for progress in adventure.quests.iter_mut() {
        if progress.state != QuestState::Active {
            continue;
        }
        let QuestObjective::Kill {
            area: quest_area,
            count,
        } = QUESTS[progress.quest].objective
        else {
            continue;
        };
        if quest_area != area {
            continue;
        }
        progress.count = (progress.count + delta).min(count);
        if progress.count >= count {
            progress.state = QuestState::ReadyToTurnIn;
            completed.push(progress.quest);
        }
    }
    for quest in completed {
        adventure.notify(format!("Objective complete: {}", QUESTS[quest].title));
    }
}

/// Mark a boss objective ready once its warlord has fallen.
fn check_boss(brimstone_world: &mut BrimstoneWorld) {
    if !brimstone_world.resources.adventure.boss_active
        || brimstone_world.resources.game.boss_entity.is_some()
    {
        return;
    }
    let area = brimstone_world.resources.adventure.area;
    brimstone_world.resources.adventure.boss_active = false;
    let mut completed: Option<usize> = None;
    for progress in brimstone_world.resources.adventure.quests.iter_mut() {
        if progress.state == QuestState::Active
            && matches!(
                QUESTS[progress.quest].objective,
                QuestObjective::Boss { area: quest_area } if quest_area == area
            )
        {
            progress.state = QuestState::ReadyToTurnIn;
            completed = Some(progress.quest);
        }
    }
    if let Some(quest) = completed {
        brimstone_world.resources.adventure.notify(format!(
            "Warlord slain! '{}' complete.",
            QUESTS[quest].title
        ));
    }
}

fn detect_interactable(brimstone_world: &mut BrimstoneWorld, world: &World) {
    let player_position = player::position(brimstone_world, world);
    let adventure = &mut brimstone_world.resources.adventure;
    let mut best: (f32, Interactable) = (INTERACT_RANGE, Interactable::None);
    for (index, npc) in adventure.npcs.iter().enumerate() {
        let distance = ground_distance(player_position, npc.position);
        if distance < best.0 {
            best = (distance, Interactable::Npc(index));
        }
    }
    for (index, portal) in adventure.portals.iter().enumerate() {
        let distance = ground_distance(player_position, portal.position);
        if distance < best.0 {
            best = (distance, Interactable::Portal(index));
        }
    }
    adventure.interactable = best.1;
}

fn ground_distance(a: Vec3, b: Vec3) -> f32 {
    let mut offset = a - b;
    offset.y = 0.0;
    offset.norm()
}

fn handle_interact_key(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if !world.resources.input.keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    match brimstone_world.resources.adventure.interactable {
        Interactable::Npc(index) => {
            if let Some(npc) = brimstone_world.resources.adventure.npcs.get(index) {
                let kind = npc.kind;
                brimstone_world.resources.adventure.dialogue_npc = kind;
                brimstone_world.resources.adventure.panel = AdvPanel::Dialogue;
                audio::play(brimstone_world, world, audio::PICKUP, 0.4);
            }
        }
        Interactable::Portal(index) => {
            if let Some(portal) = brimstone_world.resources.adventure.portals.get(index) {
                let target = portal.target_area;
                audio::play(brimstone_world, world, audio::CLEAR, 0.5);
                load_area(brimstone_world, world, target);
            }
        }
        Interactable::None => {}
    }
}

fn handle_panel_keys(brimstone_world: &mut BrimstoneWorld, world: &World) {
    let keyboard = &world.resources.input.keyboard;
    if keyboard.just_pressed(KeyCode::KeyI) {
        brimstone_world.resources.adventure.panel = AdvPanel::Inventory;
    } else if keyboard.just_pressed(KeyCode::KeyJ) {
        brimstone_world.resources.adventure.panel = AdvPanel::Quests;
    }
}

fn revive_if_dead(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if brimstone_world.resources.stats.health > 0.0
        && matches!(brimstone_world.resources.game.phase, Phase::Playing)
    {
        return;
    }
    brimstone_world.resources.stats.health = brimstone_world.resources.stats.max_health;
    brimstone_world.resources.game.phase = Phase::Playing;
    let spawn = brimstone_world.resources.adventure.spawn_point;
    player::teleport(brimstone_world, world, spawn);
    brimstone_world
        .resources
        .adventure
        .notify("You were struck down, and wake at the gate.");
}

// ============================================================================
// Panels (keyboard-driven menus)
// ============================================================================

fn number_pressed(world: &World) -> Option<usize> {
    let keyboard = &world.resources.input.keyboard;
    for (slot, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
    ]
    .into_iter()
    .enumerate()
    {
        if keyboard.just_pressed(key) {
            return Some(slot);
        }
    }
    None
}

fn handle_panel_input(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let keyboard = &world.resources.input.keyboard;
    let close = keyboard.just_pressed(KeyCode::Escape);
    let toggle = keyboard.just_pressed(KeyCode::KeyI) || keyboard.just_pressed(KeyCode::KeyJ);
    let number = number_pressed(world);

    match brimstone_world.resources.adventure.panel {
        AdvPanel::Dialogue => {
            if close {
                brimstone_world.resources.adventure.panel = AdvPanel::None;
            } else if number == Some(0) {
                dialogue_action(brimstone_world, world);
            }
        }
        AdvPanel::Shop => {
            if close {
                brimstone_world.resources.adventure.panel = AdvPanel::Dialogue;
            } else if let Some(slot) = number {
                buy_item(brimstone_world, world, slot);
            }
        }
        AdvPanel::Inventory => {
            if close || toggle {
                brimstone_world.resources.adventure.panel = AdvPanel::None;
            } else if number == Some(0) {
                use_potion(brimstone_world, world);
            }
        }
        AdvPanel::Quests => {
            if close || toggle {
                brimstone_world.resources.adventure.panel = AdvPanel::None;
            }
        }
        AdvPanel::None => {}
    }
}

/// The single primary action a dialogue offers (key 1), if any.
fn dialogue_action(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    let def = &NPCS[brimstone_world.resources.adventure.dialogue_npc];
    match def.role {
        NpcRole::Merchant => {
            brimstone_world.resources.adventure.panel = AdvPanel::Shop;
        }
        NpcRole::QuestGiver => {
            let Some(quest) = def.quest else { return };
            match quest_state(brimstone_world, quest) {
                QuestState::Available => {
                    brimstone_world
                        .resources
                        .adventure
                        .quests
                        .push(QuestProgress {
                            quest,
                            state: QuestState::Active,
                            count: 0,
                        });
                    let title = QUESTS[quest].title;
                    brimstone_world
                        .resources
                        .adventure
                        .notify(format!("Quest accepted: {title}"));
                    brimstone_world.resources.adventure.panel = AdvPanel::None;
                    audio::play(brimstone_world, world, audio::CLEAR, 0.5);
                }
                QuestState::ReadyToTurnIn => {
                    turn_in_quest(brimstone_world, world, quest);
                    brimstone_world.resources.adventure.panel = AdvPanel::None;
                }
                _ => {}
            }
        }
        NpcRole::Villager => {}
    }
}

fn turn_in_quest(brimstone_world: &mut BrimstoneWorld, world: &mut World, quest: usize) {
    if let Some(progress) = brimstone_world.resources.adventure.quest_mut(quest) {
        progress.state = QuestState::Done;
    }
    let reward = &QUESTS[quest];
    brimstone_world.resources.adventure.gold += reward.reward_gold;
    if let Some(item) = reward.reward_item {
        brimstone_world.resources.adventure.add_item(item, 1);
    }
    let title = reward.title;
    let gold = reward.reward_gold;
    brimstone_world
        .resources
        .adventure
        .notify(format!("'{title}' complete! +{gold} gold"));
    audio::play(brimstone_world, world, audio::CLEAR, 0.8);
}

fn buy_item(brimstone_world: &mut BrimstoneWorld, world: &mut World, slot: usize) {
    let Some(item) = shop_items().get(slot).copied() else {
        return;
    };
    let price = ITEMS[item].price;
    if brimstone_world.resources.adventure.gold < price {
        brimstone_world
            .resources
            .adventure
            .notify("Not enough gold.");
        audio::play(brimstone_world, world, audio::EMPTY, 0.5);
        return;
    }
    brimstone_world.resources.adventure.gold -= price;
    brimstone_world.resources.adventure.add_item(item, 1);
    let name = ITEMS[item].name;
    brimstone_world
        .resources
        .adventure
        .notify(format!("Bought {name}."));
    audio::play(brimstone_world, world, audio::PICKUP, 0.6);
}

fn use_potion(brimstone_world: &mut BrimstoneWorld, world: &mut World) {
    if brimstone_world.resources.stats.health >= brimstone_world.resources.stats.max_health {
        brimstone_world
            .resources
            .adventure
            .notify("Already at full health.");
        return;
    }
    if !brimstone_world
        .resources
        .adventure
        .remove_item(ITEM_POTION, 1)
    {
        brimstone_world
            .resources
            .adventure
            .notify("No draughts left.");
        audio::play(brimstone_world, world, audio::EMPTY, 0.5);
        return;
    }
    let max = brimstone_world.resources.stats.max_health;
    brimstone_world.resources.stats.health =
        (brimstone_world.resources.stats.health + POTION_HEAL).min(max);
    brimstone_world
        .resources
        .adventure
        .notify("You drink a Health Draught.");
    audio::play(brimstone_world, world, audio::PICKUP, 0.7);
}

fn quest_state(brimstone_world: &BrimstoneWorld, quest: usize) -> QuestState {
    brimstone_world
        .resources
        .adventure
        .quest(quest)
        .map(|progress| progress.state)
        .unwrap_or(QuestState::Available)
}

fn shop_items() -> Vec<usize> {
    (0..ITEMS.len()).filter(|item| ITEMS[*item].sold).collect()
}

// ============================================================================
// UI
// ============================================================================

pub fn build_ui(tree: &mut UiTreeBuilder) -> AdventureHandles {
    use crate::theme::*;

    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();

    let mut handles = AdventureHandles {
        root,
        ..Default::default()
    };

    tree.in_parent(root, |tree| {
        let info = tree
            .add_node()
            .window(Ab(vec2(24.0, 20.0)), Ab(vec2(320.0, 86.0)), Anchor::TopLeft)
            .with_rect(8.0, 1.5, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .entity();
        tree.in_parent(info, |tree| {
            handles.area_label = label(tree, vec2(14.0, 9.0), 18.0, ACCENT);
            handles.health_label = label(tree, vec2(14.0, 34.0), 15.0, HEALTH);
            handles.gold_label = label(tree, vec2(14.0, 56.0), 15.0, AMMO);
        });

        let quest = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-24.0, 20.0)),
                Ab(vec2(360.0, 60.0)),
                Anchor::TopRight,
            )
            .with_rect(8.0, 1.5, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .entity();
        tree.in_parent(quest, |tree| {
            handles.quest_label = tree
                .add_node()
                .window(Ab(vec2(14.0, 10.0)), Ab(vec2(332.0, 40.0)), Anchor::TopLeft)
                .with_text("", 14.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_COLOR)
                .entity();
        });

        handles.crosshair = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(7.0, 7.0)), Anchor::Center)
            .with_rect(3.5, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(CROSSHAIR)
            .entity();

        handles.prompt_label = tree
            .add_node()
            .window(Rl(vec2(50.0, 64.0)), Ab(vec2(640.0, 26.0)), Anchor::Center)
            .with_text("", 18.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.0)
            .with_visible(false)
            .color_raw::<UiBase>(ACCENT_HOT)
            .entity();

        handles.notice_label = tree
            .add_node()
            .window(Rl(vec2(50.0, 86.0)), Ab(vec2(760.0, 24.0)), Anchor::Center)
            .with_text("", 16.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 1.5)
            .with_visible(false)
            .color_raw::<UiBase>(WHITE)
            .entity();

        handles.banner_label = tree
            .add_node()
            .window(Rl(vec2(50.0, 24.0)), Ab(vec2(760.0, 56.0)), Anchor::Center)
            .with_text("", 46.0)
            .text_center()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 0.9), 2.5)
            .with_visible(false)
            .color_raw::<UiBase>(ACCENT)
            .entity();

        handles.panel_root = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(720.0, 420.0)), Anchor::Center)
            .with_rect(10.0, 2.0, PANEL_BORDER)
            .color_raw::<UiBase>(vec4(0.05, 0.04, 0.06, 0.95))
            .with_shadow(vec4(0.0, 0.0, 0.0, 0.5), vec2(0.0, 4.0), 18.0, 0.0)
            .with_visible(false)
            .entity();
        tree.in_parent(handles.panel_root, |tree| {
            handles.panel_title = tree
                .add_node()
                .window(Ab(vec2(28.0, 22.0)), Ab(vec2(664.0, 34.0)), Anchor::TopLeft)
                .with_text("", 26.0)
                .text_left()
                .color_raw::<UiBase>(ACCENT)
                .entity();
            handles.panel_body = tree
                .add_node()
                .window(
                    Ab(vec2(28.0, 72.0)),
                    Ab(vec2(664.0, 326.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 18.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_COLOR)
                .entity();
        });
    });

    handles
}

fn label(tree: &mut UiTreeBuilder, offset: Vec2, size: f32, color: Vec4) -> Entity {
    tree.add_node()
        .window(Ab(offset), Ab(vec2(300.0, 22.0)), Anchor::TopLeft)
        .with_text("", size)
        .text_left()
        .color_raw::<UiBase>(color)
        .entity()
}

fn update_ui(brimstone_world: &BrimstoneWorld, world: &mut World) {
    let handles = brimstone_world.resources.ui_handles.adventure;
    let adventure = &brimstone_world.resources.adventure;
    let area = &AREAS[adventure.area];

    ui_set_text(world, handles.area_label, area.name);
    ui_set_text(
        world,
        handles.health_label,
        &format!(
            "HP  {:.0} / {:.0}",
            brimstone_world.resources.stats.health.max(0.0),
            brimstone_world.resources.stats.max_health
        ),
    );
    ui_set_text(
        world,
        handles.gold_label,
        &format!("GOLD  {}", adventure.gold),
    );
    ui_set_text(world, handles.quest_label, &active_quest_text(adventure));

    let crosshair_on = adventure.panel == AdvPanel::None;
    ui_set_visible(world, handles.crosshair, crosshair_on);

    let prompt = prompt_text(adventure);
    ui_set_visible(
        world,
        handles.prompt_label,
        crosshair_on && !prompt.is_empty(),
    );
    if !prompt.is_empty() {
        ui_set_text(world, handles.prompt_label, &prompt);
    }

    let show_notice = adventure.notice_timer > 0.0;
    ui_set_visible(world, handles.notice_label, show_notice);
    if show_notice {
        ui_set_text(world, handles.notice_label, &adventure.notice);
    }

    let show_banner = adventure.banner > 0.0;
    ui_set_visible(world, handles.banner_label, show_banner);
    if show_banner {
        ui_set_text(world, handles.banner_label, area.name);
    }

    let show_panel = adventure.panel != AdvPanel::None;
    ui_set_visible(world, handles.panel_root, show_panel);
    if show_panel {
        let (title, body) = panel_text(brimstone_world);
        ui_set_text(world, handles.panel_title, &title);
        ui_set_text(world, handles.panel_body, &body);
    }
}

fn active_quest_text(adventure: &crate::ecs::AdventureState) -> String {
    for progress in &adventure.quests {
        if progress.state == QuestState::Active {
            let quest = &QUESTS[progress.quest];
            let detail = match quest.objective {
                QuestObjective::Kill { count, .. } => {
                    format!("{} ({}/{})", quest.summary, progress.count, count)
                }
                QuestObjective::Boss { .. } => quest.summary.to_string(),
            };
            return format!("QUEST  {}\n  {}", quest.title, detail);
        }
        if progress.state == QuestState::ReadyToTurnIn {
            return format!(
                "QUEST  {}\n  Ready to turn in!",
                QUESTS[progress.quest].title
            );
        }
    }
    "No active quest.\n  Press J for the quest log.".to_string()
}

fn prompt_text(adventure: &crate::ecs::AdventureState) -> String {
    match adventure.interactable {
        Interactable::Npc(index) => adventure
            .npcs
            .get(index)
            .map(|npc| format!("[E] Speak with {}", NPCS[npc.kind].name))
            .unwrap_or_default(),
        Interactable::Portal(index) => adventure
            .portals
            .get(index)
            .map(|portal| format!("[E] Travel to {}", portal.label))
            .unwrap_or_default(),
        Interactable::None => String::new(),
    }
}

fn panel_text(brimstone_world: &BrimstoneWorld) -> (String, String) {
    let adventure = &brimstone_world.resources.adventure;
    match adventure.panel {
        AdvPanel::Dialogue => {
            let kind = adventure.dialogue_npc;
            let def = &NPCS[kind];
            let mut body = format!("\"{}\"\n\n", def.line);
            match def.role {
                NpcRole::Merchant => body.push_str("[1] Trade\n[Esc] Leave"),
                NpcRole::QuestGiver => {
                    let quest = def.quest.unwrap_or(0);
                    match quest_state(brimstone_world, quest) {
                        QuestState::Available => body.push_str(&format!(
                            "[1] Accept '{}'\n    {}\n[Esc] Leave",
                            QUESTS[quest].title, QUESTS[quest].summary
                        )),
                        QuestState::Active => {
                            body.push_str("Return when the deed is done.\n[Esc] Leave")
                        }
                        QuestState::ReadyToTurnIn => body.push_str(&format!(
                            "[1] Turn in '{}'  (+{} gold)\n[Esc] Leave",
                            QUESTS[quest].title, QUESTS[quest].reward_gold
                        )),
                        QuestState::Done => body.push_str("Well done, hero.\n[Esc] Leave"),
                    }
                }
                NpcRole::Villager => body.push_str("[Esc] Leave"),
            }
            (def.name.to_string(), body)
        }
        AdvPanel::Shop => {
            let mut body = format!("Your gold: {}\n\n", adventure.gold);
            for (slot, item) in shop_items().iter().enumerate() {
                body.push_str(&format!(
                    "[{}] {} - {} gold\n",
                    slot + 1,
                    ITEMS[*item].name,
                    ITEMS[*item].price
                ));
            }
            body.push_str("\n[Esc] Back");
            ("Merchant Vex".to_string(), body)
        }
        AdvPanel::Inventory => {
            let mut body = format!("Gold: {}\n\n", adventure.gold);
            if adventure.items.is_empty() {
                body.push_str("Your pack is empty.\n");
            }
            for (item, count) in &adventure.items {
                body.push_str(&format!("  {} x{}\n", ITEMS[*item].name, count));
            }
            body.push_str(&format!(
                "\n[1] Use Health Draught ({} held)\n[Esc] Close",
                adventure.item_count(ITEM_POTION)
            ));
            ("INVENTORY".to_string(), body)
        }
        AdvPanel::Quests => {
            let mut body = String::new();
            if adventure.quests.is_empty() {
                body.push_str("No quests yet. Seek out the townsfolk.\n");
            }
            for progress in &adventure.quests {
                let quest = &QUESTS[progress.quest];
                let status = match progress.state {
                    QuestState::Active => match quest.objective {
                        QuestObjective::Kill { count, .. } => {
                            format!("{}/{}", progress.count, count)
                        }
                        QuestObjective::Boss { .. } => "hunting".to_string(),
                    },
                    QuestState::ReadyToTurnIn => "READY".to_string(),
                    QuestState::Done => "DONE".to_string(),
                    QuestState::Available => "-".to_string(),
                };
                body.push_str(&format!(
                    "{}  [{}]\n   {}\n",
                    quest.title, status, quest.summary
                ));
            }
            body.push_str("\n[Esc] Close");
            ("QUEST LOG".to_string(), body)
        }
        AdvPanel::None => (String::new(), String::new()),
    }
}
