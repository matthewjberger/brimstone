//! Hand-authored level definitions. Each level is a distinct *place* — its own
//! footprint, room/corridor architecture, height structure, spawn, and exit
//! gate — not a reskin of one arena. Walls (full height) partition space into
//! rooms and corridors with doorway gaps; platforms make raised galleries and
//! catwalks reached by pads; the exit sits at the end of a navigable path.

use nightshade::prelude::Atmosphere;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockKind {
    #[default]
    Wall,
    Pillar,
    Cover,
    Choke,
    Monument,
    Platform,
    /// Emissive structural block — reactor cores, altars, glowing machinery.
    Core,
}

impl BlockKind {
    /// The editor's placeable palette (Core is hand-authored only).
    pub const ALL: [BlockKind; 6] = [
        BlockKind::Wall,
        BlockKind::Pillar,
        BlockKind::Platform,
        BlockKind::Cover,
        BlockKind::Choke,
        BlockKind::Monument,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BlockKind::Wall => "WALL",
            BlockKind::Pillar => "PILLAR",
            BlockKind::Cover => "COVER",
            BlockKind::Choke => "CHOKE",
            BlockKind::Monument => "MONUMENT",
            BlockKind::Platform => "PLATFORM",
            BlockKind::Core => "CORE",
        }
    }

    pub fn code(self) -> u8 {
        match self {
            BlockKind::Wall => 0,
            BlockKind::Pillar => 1,
            BlockKind::Cover => 2,
            BlockKind::Choke => 3,
            BlockKind::Monument => 4,
            BlockKind::Platform => 5,
            BlockKind::Core => 6,
        }
    }

    pub fn from_code(code: u8) -> BlockKind {
        match code {
            1 => BlockKind::Pillar,
            2 => BlockKind::Cover,
            3 => BlockKind::Choke,
            4 => BlockKind::Monument,
            5 => BlockKind::Platform,
            6 => BlockKind::Core,
            _ => BlockKind::Wall,
        }
    }
}

/// (cx, cy, cz, sx, sy, sz, kind)
pub type BlockSpec = (f32, f32, f32, f32, f32, f32, BlockKind);
/// (x, z, [r, g, b])
pub type BeaconSpec = (f32, f32, [f32; 3]);
/// (cx, cy, cz, sx, sy, sz, pitch_radians, yaw_radians) — a tilted slab you can
/// walk up. Keep pitch under ~27 degrees so the controller climbs without sliding.
pub type RampSpec = (f32, f32, f32, f32, f32, f32, f32, f32);

#[derive(Clone, Copy, Default)]
pub struct Roster {
    pub imps: u32,
    pub swarmers: u32,
    pub casters: u32,
    pub brutes: u32,
    pub gargoyles: u32,
    pub sentinels: u32,
}

pub struct Level {
    pub name: &'static str,
    pub atmosphere: Atmosphere,
    pub fog: [f32; 3],
    /// Footprint half-extents; the floor and perimeter walls are sized to these,
    /// so levels are different shapes and sizes, not one square box.
    pub half_x: f32,
    pub half_z: f32,
    pub spawn: [f32; 3],
    pub exit: [f32; 2],
    pub blocks: &'static [BlockSpec],
    pub ramps: &'static [RampSpec],
    pub beacons: &'static [BeaconSpec],
    pub spawn_points: &'static [(f32, f32)],
    pub pads: &'static [(f32, f32)],
    pub roster: Roster,
}

/// An owned, mutable level — what the in-game editor builds and what custom
/// play sessions run from.
#[derive(Clone)]
pub struct LevelData {
    pub name: String,
    pub atmosphere_index: u8,
    pub fog: [f32; 3],
    pub spawn: [f32; 3],
    pub exit: [f32; 2],
    pub blocks: Vec<BlockSpec>,
    pub pads: Vec<(f32, f32)>,
    pub spawn_points: Vec<(f32, f32)>,
    pub roster: Roster,
}

impl Default for LevelData {
    fn default() -> Self {
        Self {
            name: "CUSTOM".to_string(),
            atmosphere_index: 0,
            fog: [0.05, 0.03, 0.10],
            spawn: [0.0, 1.2, 16.0],
            exit: [0.0, -16.5],
            blocks: Vec::new(),
            pads: Vec::new(),
            spawn_points: Vec::new(),
            roster: Roster {
                imps: 6,
                swarmers: 5,
                casters: 2,
                brutes: 0,
                gargoyles: 1,
                sentinels: 0,
            },
        }
    }
}

pub fn atmosphere_for(index: u8) -> Atmosphere {
    match index % 3 {
        1 => Atmosphere::Sunset,
        2 => Atmosphere::Space,
        _ => Atmosphere::Nebula,
    }
}

pub fn count() -> usize {
    LEVELS.len()
}

pub fn level(index: usize) -> &'static Level {
    &LEVELS[index % LEVELS.len()]
}

use BlockKind::{Core, Cover, Monument, Pillar, Platform, Wall};

// ============================================================================
// L0 — FOUNDRY. A wide hall dominated by a central machine block you circulate
// around (left or right) from the south entry to the north gate. A low roof on
// the machine is sniper high-ground reached by side pads.
// ============================================================================
const L0_BLOCKS: &[BlockSpec] = &[
    (0.0, 1.5, 0.0, 8.0, 3.0, 6.0, Monument), // reactor housing, top 3
    (0.0, 4.0, 0.0, 3.0, 4.0, 3.0, Core),     // glowing core rising out of it
    (5.0, 2.0, 4.0, 1.2, 4.0, 1.2, Pillar),   // coolant pipes around the reactor
    (-5.0, 2.0, 4.0, 1.2, 4.0, 1.2, Pillar),
    (5.0, 2.0, -4.0, 1.2, 4.0, 1.2, Pillar),
    (-5.0, 2.0, -4.0, 1.2, 4.0, 1.2, Pillar),
    (15.5, 1.5, 0.0, 4.0, 3.0, 18.0, Platform), // east gantry, top 3
    (-15.5, 1.5, 0.0, 4.0, 3.0, 18.0, Platform), // west gantry, top 3
    (7.0, 0.6, 12.0, 4.0, 1.2, 1.4, Cover),     // entry cover
    (-7.0, 0.6, -12.0, 4.0, 1.2, 1.4, Cover),   // gate cover
];
const L0_BEACONS: &[BeaconSpec] = &[
    (0.0, 6.0, [1.9, 0.8, 0.25]),  // reactor uplight, front
    (0.0, -6.0, [1.9, 0.8, 0.25]), // reactor uplight, back
    (14.0, 8.0, [0.2, 1.2, 1.7]),
    (-14.0, -8.0, [0.2, 1.2, 1.7]),
];
const L0_SPAWNS: &[(f32, f32)] = &[
    (16.0, 10.0),
    (-16.0, 10.0),
    (16.0, -10.0),
    (-16.0, -10.0),
    (8.0, 0.0),
    (-8.0, 0.0),
];
const L0_PADS: &[(f32, f32)] = &[(12.0, 0.0), (-12.0, 0.0)];

// ============================================================================
// L1 — THE LOCKS. A long, narrow corridor chopped into chambers by full-height
// bulkheads, each with a doorway that alternates side to side, so you serpentine
// north toward the gate while the horde funnels through the gaps.
// ============================================================================
const L1_BLOCKS: &[BlockSpec] = &[
    (-3.0, 4.0, 13.0, 14.0, 8.0, 1.0, Wall), // gap east x[4,10]
    (3.0, 4.0, 5.0, 14.0, 8.0, 1.0, Wall),   // gap west x[-10,-4]
    (-3.0, 4.0, -3.0, 14.0, 8.0, 1.0, Wall), // gap east
    (3.0, 4.0, -11.0, 14.0, 8.0, 1.0, Wall), // gap west
    (-6.0, 4.0, -18.0, 8.0, 8.0, 1.0, Wall), // final wall, center gap x[-2,2]
    (6.0, 4.0, -18.0, 8.0, 8.0, 1.0, Wall),
    (-6.0, 0.6, 9.0, 2.4, 1.2, 2.4, Cover),
    (6.0, 0.6, 1.0, 2.4, 1.2, 2.4, Cover),
    (-6.0, 0.6, -7.0, 2.4, 1.2, 2.4, Cover),
];
const L1_BEACONS: &[BeaconSpec] = &[
    (7.0, 13.0, [0.3, 1.5, 1.6]),
    (-7.0, 5.0, [1.6, 0.4, 0.4]),
    (7.0, -3.0, [0.3, 1.5, 1.6]),
    (-7.0, -11.0, [1.6, 0.4, 0.4]),
];
const L1_SPAWNS: &[(f32, f32)] = &[
    (7.0, 9.0),
    (-7.0, 1.0),
    (7.0, -7.0),
    (-7.0, -14.0),
    (0.0, -20.0),
];
const L1_PADS: &[(f32, f32)] = &[];

// ============================================================================
// L2 — THE GALLERY. A central pit ringed by a raised balcony walkway. Melee
// boils in the pit; casters and sentinels hold the balcony. Side pads kick you
// up to the high ring; the gate sits in a gap in the south balcony.
// ============================================================================
const L2_BLOCKS: &[BlockSpec] = &[
    (0.0, 1.5, 14.0, 30.0, 3.0, 4.0, Platform), // north balcony, top 3
    (-9.0, 1.5, -14.0, 12.0, 3.0, 4.0, Platform), // south balcony (split for gate gap)
    (9.0, 1.5, -14.0, 12.0, 3.0, 4.0, Platform),
    (14.0, 1.5, 0.0, 4.0, 3.0, 24.0, Platform), // east balcony
    (-14.0, 1.5, 0.0, 4.0, 3.0, 24.0, Platform), // west balcony
    (0.0, 0.5, 0.0, 4.0, 1.0, 4.0, Cover),      // pit cover
    (8.0, 0.5, -8.0, 2.0, 1.0, 2.0, Cover),     // clear of pad/spawn corners
    (-8.0, 0.5, 8.0, 2.0, 1.0, 2.0, Cover),
];
const L2_BEACONS: &[BeaconSpec] = &[
    (0.0, 0.0, [0.4, 0.7, 1.8]),
    (12.0, 12.0, [1.5, 0.4, 1.3]),
    (-12.0, -12.0, [1.5, 0.4, 1.3]),
];
// Pit-only (|x|,|z| < 12) so enemies never spawn inside the raised balconies.
const L2_SPAWNS: &[(f32, f32)] = &[
    (0.0, 10.0),
    (10.0, 0.0),
    (-10.0, 0.0),
    (0.0, -9.0),
    (8.0, 8.0),
    (-8.0, -8.0),
];
const L2_PADS: &[(f32, f32)] = &[(8.0, 8.0), (-8.0, -8.0)];

// ============================================================================
// L3 — SPIRE HALL. A tall central tower ringed by ledges at climbing heights,
// reached only by pads. The fight goes vertical; gargoyles own the air. Gate on
// the ground at the north edge.
// ============================================================================
const L3_BLOCKS: &[BlockSpec] = &[
    (0.0, 2.0, 0.0, 5.0, 4.0, 5.0, Monument), // spire base, top 4
    (0.0, 6.0, 0.0, 3.0, 4.0, 3.0, Monument), // spire shaft, top 8
    (0.0, 9.5, 0.0, 1.8, 3.0, 1.8, Core),     // glowing crown, top 11
    (6.5, 0.75, 0.0, 4.0, 1.5, 4.0, Platform), // top 1.5
    (-6.5, 1.5, 0.0, 4.0, 3.0, 4.0, Platform), // top 3.0
    (0.0, 2.25, 6.5, 4.0, 4.5, 4.0, Platform), // top 4.5
    (0.0, 1.1, -6.5, 4.0, 2.2, 4.0, Platform), // top 2.2
    (13.0, 2.5, 13.0, 1.6, 5.0, 1.6, Pillar),
    (-13.0, 2.5, -13.0, 1.6, 5.0, 1.6, Pillar),
];
const L3_BEACONS: &[BeaconSpec] = &[
    (0.0, 0.0, [0.7, 0.4, 1.8]),
    (11.0, -11.0, [0.2, 1.5, 1.4]),
    (-11.0, 11.0, [1.6, 0.7, 0.2]),
];
const L3_SPAWNS: &[(f32, f32)] = &[
    (14.0, 14.0),
    (-14.0, -14.0),
    (14.0, -14.0),
    (-14.0, 14.0),
    (0.0, 14.0),
    (14.0, 0.0),
];
const L3_PADS: &[(f32, f32)] = &[(6.5, 6.5), (-6.5, -6.5), (0.0, 9.0)];

// ============================================================================
// L4 — THE WARRENS. A spine corridor with sealed side rooms off it; the keycard
// is locked away in the west room behind a doorway. Grab it, then run the spine
// to the north gate.
// ============================================================================
const L4_BLOCKS: &[BlockSpec] = &[
    // West chamber (keycard vault): enclosed pocket off the spine's left, one
    // doorway at z[8,11]. South wall seals it; the spine divider is the east wall.
    (-11.0, 4.0, 3.0, 14.0, 8.0, 1.0, Wall), // south wall, x[-18,-4]
    (-4.0, 4.0, 5.5, 1.0, 8.0, 5.0, Wall),   // spine divider z[3,8]
    (-4.0, 4.0, 13.0, 1.0, 8.0, 4.0, Wall),  // spine divider z[11,15]
    (-12.0, 0.5, 8.0, 1.8, 1.0, 1.8, Core),  // keycard shrine pedestal
    // East chamber (ambush pocket): mirror, doorway at z[-11,-8].
    (11.0, 4.0, -3.0, 14.0, 8.0, 1.0, Wall), // north wall, x[4,18]
    (4.0, 4.0, -5.5, 1.0, 8.0, 5.0, Wall),   // spine divider z[-8,-3]
    (4.0, 4.0, -13.0, 1.0, 8.0, 4.0, Wall),  // spine divider z[-15,-11]
    // Spine baffles: stagger the central run so it weaves instead of a straight shot.
    (1.8, 0.9, 4.0, 3.6, 1.8, 1.0, Cover),
    (-1.8, 0.9, -4.0, 3.6, 1.8, 1.0, Cover),
    (-12.0, 0.6, 12.0, 3.0, 1.2, 1.4, Cover), // cover inside the vault
    (12.0, 0.6, -12.0, 3.0, 1.2, 1.4, Cover), // cover inside the ambush pocket
];
const L4_BEACONS: &[BeaconSpec] = &[
    (-12.0, 8.0, [1.9, 0.7, 0.2]),  // keycard shrine, hot gold
    (-4.0, 9.5, [0.3, 1.3, 1.6]),   // vault doorway, cyan
    (4.0, -9.5, [1.7, 0.4, 0.3]),   // ambush doorway, red
    (0.0, 5.0, [0.4, 0.5, 0.95]),   // spine waypoints
    (0.0, -5.0, [0.4, 0.5, 0.95]),
];
const L4_SPAWNS: &[(f32, f32)] = &[
    (0.0, 11.0),
    (0.0, -11.0),
    (-12.0, 6.0),
    (12.0, -6.0),
    (14.0, 12.0),
    (-14.0, -12.0),
];
const L4_PADS: &[(f32, f32)] = &[];

// ============================================================================
// L5 — THE CRUCIBLE. The core: a broad arena with two flanking alcoves for
// cover and pickups, a low cover ring in the middle, and pillars to break the
// warlord's sightlines. No exit until the floor is clear.
// ============================================================================
// A cathedral nave: you fight north up a colonnade toward a raised throne
// where the warlord looms, flanked by two great monoliths, the gate beyond it.
const L5_BLOCKS: &[BlockSpec] = &[
    (0.0, 1.0, -12.0, 9.0, 2.0, 4.0, Platform), // throne stage, top 2
    (-7.5, 3.0, -13.0, 3.0, 6.0, 3.0, Monument), // left monolith
    (7.5, 3.0, -13.0, 3.0, 6.0, 3.0, Monument), // right monolith
    (-6.5, 2.5, -2.0, 1.8, 5.0, 1.8, Pillar),   // nave colonnade
    (6.5, 2.5, -2.0, 1.8, 5.0, 1.8, Pillar),
    (-6.5, 2.5, 5.0, 1.8, 5.0, 1.8, Pillar),
    (6.5, 2.5, 5.0, 1.8, 5.0, 1.8, Pillar),
    (14.5, 0.5, 2.0, 5.0, 1.0, 4.0, Cover), // east aisle cover
    (-14.5, 0.5, 2.0, 5.0, 1.0, 4.0, Cover), // west aisle cover
    (0.0, 0.6, 11.0, 5.0, 1.2, 2.0, Cover), // cover by the entrance
];
const L5_BEACONS: &[BeaconSpec] = &[
    (0.0, -12.0, [2.2, 0.5, 0.18]), // throne, hot red
    (-7.5, -13.0, [1.9, 0.6, 0.15]),
    (7.5, -13.0, [1.9, 0.6, 0.15]),
    (12.0, 11.0, [0.3, 0.4, 0.75]), // entrance, cold
    (-12.0, 11.0, [0.3, 0.4, 0.75]),
];
const L5_SPAWNS: &[(f32, f32)] = &[
    (0.0, -13.0), // the throne (boss)
    (15.0, -8.0),
    (-15.0, -8.0),
    (13.0, 4.0),
    (-13.0, 4.0),
    (0.0, 13.0),
];
const L5_PADS: &[(f32, f32)] = &[];

const LEVELS: &[Level] = &[
    Level {
        name: "FOUNDRY",
        atmosphere: Atmosphere::Sunset,
        fog: [0.10, 0.05, 0.02],
        half_x: 19.0,
        half_z: 15.0,
        spawn: [0.0, 1.2, 12.0],
        exit: [0.0, -12.5],
        blocks: L0_BLOCKS,
        ramps: &[],
        beacons: L0_BEACONS,
        spawn_points: L0_SPAWNS,
        pads: L0_PADS,
        roster: Roster {
            imps: 7,
            swarmers: 5,
            casters: 2,
            brutes: 0,
            gargoyles: 0,
            sentinels: 1,
        },
    },
    Level {
        name: "THE LOCKS",
        atmosphere: Atmosphere::Space,
        fog: [0.02, 0.04, 0.10],
        half_x: 10.0,
        half_z: 24.0,
        spawn: [0.0, 1.2, 21.0],
        exit: [0.0, -21.5],
        blocks: L1_BLOCKS,
        ramps: &[],
        beacons: L1_BEACONS,
        spawn_points: L1_SPAWNS,
        pads: L1_PADS,
        roster: Roster {
            imps: 6,
            swarmers: 7,
            casters: 3,
            brutes: 1,
            gargoyles: 0,
            sentinels: 1,
        },
    },
    Level {
        name: "THE GALLERY",
        atmosphere: Atmosphere::Nebula,
        fog: [0.05, 0.03, 0.10],
        half_x: 17.0,
        half_z: 17.0,
        spawn: [0.0, 1.2, 10.0],
        exit: [0.0, -15.5],
        blocks: L2_BLOCKS,
        ramps: &[],
        beacons: L2_BEACONS,
        spawn_points: L2_SPAWNS,
        pads: L2_PADS,
        roster: Roster {
            imps: 5,
            swarmers: 5,
            casters: 4,
            brutes: 1,
            gargoyles: 2,
            sentinels: 3,
        },
    },
    Level {
        name: "SPIRE HALL",
        atmosphere: Atmosphere::Nebula,
        fog: [0.06, 0.03, 0.10],
        half_x: 17.0,
        half_z: 17.0,
        spawn: [0.0, 1.2, 14.0],
        exit: [0.0, -15.5],
        blocks: L3_BLOCKS,
        ramps: &[],
        beacons: L3_BEACONS,
        spawn_points: L3_SPAWNS,
        pads: L3_PADS,
        roster: Roster {
            imps: 4,
            swarmers: 5,
            casters: 3,
            brutes: 1,
            gargoyles: 5,
            sentinels: 2,
        },
    },
    Level {
        name: "THE WARRENS",
        atmosphere: Atmosphere::Space,
        fog: [0.02, 0.03, 0.08],
        half_x: 18.0,
        half_z: 16.0,
        spawn: [0.0, 1.2, 13.0],
        exit: [0.0, -13.5],
        blocks: L4_BLOCKS,
        ramps: &[],
        beacons: L4_BEACONS,
        spawn_points: L4_SPAWNS,
        pads: L4_PADS,
        roster: Roster {
            imps: 6,
            swarmers: 6,
            casters: 4,
            brutes: 1,
            gargoyles: 1,
            sentinels: 2,
        },
    },
    Level {
        name: "THE CRUCIBLE",
        atmosphere: Atmosphere::Nebula,
        fog: [0.10, 0.03, 0.04],
        half_x: 18.0,
        half_z: 18.0,
        spawn: [0.0, 1.2, 15.0],
        exit: [0.0, -15.5],
        blocks: L5_BLOCKS,
        ramps: &[],
        beacons: L5_BEACONS,
        spawn_points: L5_SPAWNS,
        pads: L5_PADS,
        roster: Roster {
            imps: 9,
            swarmers: 8,
            casters: 3,
            brutes: 2,
            gargoyles: 2,
            sentinels: 2,
        },
    },
];
