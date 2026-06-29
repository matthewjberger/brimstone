//! Hand-authored level definitions. Each level is a distinct layout with its
//! own geometry, sky, enemy roster, player spawn, and exit gate. The game
//! advances through them and loops with scaling difficulty.

use nightshade::prelude::Atmosphere;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Wall,
    Pillar,
    Cover,
    Choke,
    Monument,
}

/// (cx, cy, cz, sx, sy, sz, kind)
pub type BlockSpec = (f32, f32, f32, f32, f32, f32, BlockKind);
/// (x, z, [r, g, b])
pub type BeaconSpec = (f32, f32, [f32; 3]);

#[derive(Clone, Copy)]
pub struct Roster {
    pub imps: u32,
    pub swarmers: u32,
    pub casters: u32,
}

pub struct Level {
    pub name: &'static str,
    pub atmosphere: Atmosphere,
    pub fog: [f32; 3],
    pub spawn: [f32; 3],
    pub exit: [f32; 2],
    pub blocks: &'static [BlockSpec],
    pub beacons: &'static [BeaconSpec],
    pub spawn_points: &'static [(f32, f32)],
    pub roster: Roster,
}

pub fn count() -> usize {
    LEVELS.len()
}

pub fn level(index: usize) -> &'static Level {
    &LEVELS[index % LEVELS.len()]
}

use BlockKind::{Choke, Cover, Monument, Pillar, Wall};

const L1_BLOCKS: &[BlockSpec] = &[
    (0.0, 3.5, 0.0, 3.0, 7.0, 3.0, Monument),
    (9.0, 2.0, 6.0, 1.8, 4.0, 1.8, Pillar),
    (-8.0, 2.3, 7.5, 1.8, 4.6, 1.8, Pillar),
    (-10.5, 1.6, -6.0, 1.8, 3.2, 1.8, Pillar),
    (7.5, 1.6, -9.5, 1.8, 3.2, 1.8, Pillar),
    (4.0, 0.45, 9.5, 3.4, 0.9, 1.4, Cover),
    (-4.5, 0.45, -8.5, 3.4, 0.9, 1.4, Cover),
];
const L1_BEACONS: &[BeaconSpec] = &[
    (5.0, 5.0, [0.2, 1.5, 1.8]),
    (-5.0, 5.0, [1.6, 0.3, 1.5]),
    (5.0, -5.0, [1.7, 0.8, 0.2]),
    (-5.0, -5.0, [0.3, 1.6, 0.5]),
];
const L1_SPAWNS: &[(f32, f32)] = &[
    (0.0, -16.0),
    (14.0, -8.0),
    (-14.0, -8.0),
    (14.0, 8.0),
    (-14.0, 8.0),
];

const L2_BLOCKS: &[BlockSpec] = &[
    (5.5, 1.75, 0.0, 1.0, 3.5, 22.0, Wall),
    (-5.5, 1.75, 0.0, 1.0, 3.5, 22.0, Wall),
    (11.5, 1.75, 7.0, 7.0, 3.5, 1.0, Wall),
    (-11.5, 1.75, 7.0, 7.0, 3.5, 1.0, Wall),
    (11.5, 1.75, -7.0, 7.0, 3.5, 1.0, Wall),
    (-11.5, 1.75, -7.0, 7.0, 3.5, 1.0, Wall),
    (0.0, 0.5, 6.0, 2.2, 1.0, 1.0, Choke),
    (0.0, 0.5, -6.0, 2.2, 1.0, 1.0, Choke),
];
const L2_BEACONS: &[BeaconSpec] = &[
    (0.0, 14.0, [1.7, 0.5, 0.15]),
    (0.0, -14.0, [1.7, 0.5, 0.15]),
    (12.0, 0.0, [0.2, 1.4, 1.6]),
    (-12.0, 0.0, [0.2, 1.4, 1.6]),
];
const L2_SPAWNS: &[(f32, f32)] = &[
    (0.0, 17.0),
    (0.0, -17.0),
    (14.0, 11.0),
    (-14.0, 11.0),
    (14.0, -11.0),
    (-14.0, -11.0),
];

const L3_BLOCKS: &[BlockSpec] = &[
    (9.0, 2.5, 0.0, 1.5, 5.0, 1.5, Pillar),
    (6.4, 2.5, 6.4, 1.5, 5.0, 1.5, Pillar),
    (0.0, 2.5, 9.0, 1.5, 5.0, 1.5, Pillar),
    (-6.4, 2.5, 6.4, 1.5, 5.0, 1.5, Pillar),
    (-9.0, 2.5, 0.0, 1.5, 5.0, 1.5, Pillar),
    (-6.4, 2.5, -6.4, 1.5, 5.0, 1.5, Pillar),
    (0.0, 2.5, -9.0, 1.5, 5.0, 1.5, Pillar),
    (6.4, 2.5, -6.4, 1.5, 5.0, 1.5, Pillar),
    (0.0, 0.5, 0.0, 2.4, 1.0, 2.4, Cover),
];
const L3_BEACONS: &[BeaconSpec] = &[
    (0.0, 0.0, [0.3, 0.6, 1.8]),
    (13.0, 13.0, [1.5, 0.3, 1.4]),
    (-13.0, -13.0, [1.5, 0.3, 1.4]),
];
const L3_SPAWNS: &[(f32, f32)] = &[
    (15.0, 0.0),
    (-15.0, 0.0),
    (0.0, 15.0),
    (0.0, -15.0),
    (11.0, 11.0),
    (-11.0, -11.0),
];

const L4_BLOCKS: &[BlockSpec] = &[
    (11.0, 1.75, 0.0, 7.0, 3.5, 1.0, Wall),
    (-11.0, 1.75, 0.0, 7.0, 3.5, 1.0, Wall),
    (0.0, 1.75, 11.0, 1.0, 3.5, 7.0, Wall),
    (0.0, 1.75, -11.0, 1.0, 3.5, 7.0, Wall),
    (0.0, 2.0, 0.0, 2.2, 4.0, 2.2, Monument),
    (12.5, 0.45, 12.5, 1.4, 0.9, 1.4, Choke),
    (-12.5, 0.45, 12.5, 1.4, 0.9, 1.4, Choke),
    (12.5, 0.45, -12.5, 1.4, 0.9, 1.4, Choke),
    (-12.5, 0.45, -12.5, 1.4, 0.9, 1.4, Choke),
];
const L4_BEACONS: &[BeaconSpec] = &[
    (6.0, 6.0, [1.6, 0.3, 0.3]),
    (-6.0, 6.0, [1.6, 0.6, 0.2]),
    (6.0, -6.0, [1.6, 0.6, 0.2]),
    (-6.0, -6.0, [1.6, 0.3, 0.3]),
];
const L4_SPAWNS: &[(f32, f32)] = &[
    (16.0, 16.0),
    (-16.0, 16.0),
    (16.0, -16.0),
    (-16.0, -16.0),
    (16.0, 0.0),
    (-16.0, 0.0),
];

const LEVELS: &[Level] = &[
    Level {
        name: "ARRIVAL",
        atmosphere: Atmosphere::Nebula,
        fog: [0.05, 0.02, 0.10],
        spawn: [0.0, 1.2, 14.0],
        exit: [0.0, -16.5],
        blocks: L1_BLOCKS,
        beacons: L1_BEACONS,
        spawn_points: L1_SPAWNS,
        roster: Roster {
            imps: 7,
            swarmers: 4,
            casters: 1,
        },
    },
    Level {
        name: "GAUNTLET",
        atmosphere: Atmosphere::Sunset,
        fog: [0.12, 0.04, 0.02],
        spawn: [0.0, 1.2, 16.0],
        exit: [0.0, -16.5],
        blocks: L2_BLOCKS,
        beacons: L2_BEACONS,
        spawn_points: L2_SPAWNS,
        roster: Roster {
            imps: 6,
            swarmers: 7,
            casters: 2,
        },
    },
    Level {
        name: "COLONNADE",
        atmosphere: Atmosphere::Space,
        fog: [0.02, 0.03, 0.08],
        spawn: [0.0, 1.2, 15.5],
        exit: [0.0, -16.5],
        blocks: L3_BLOCKS,
        beacons: L3_BEACONS,
        spawn_points: L3_SPAWNS,
        roster: Roster {
            imps: 4,
            swarmers: 5,
            casters: 5,
        },
    },
    Level {
        name: "CRUCIBLE",
        atmosphere: Atmosphere::Nebula,
        fog: [0.10, 0.03, 0.04],
        spawn: [0.0, 1.2, 16.5],
        exit: [0.0, -16.5],
        blocks: L4_BLOCKS,
        beacons: L4_BEACONS,
        spawn_points: L4_SPAWNS,
        roster: Roster {
            imps: 9,
            swarmers: 8,
            casters: 3,
        },
    },
];
