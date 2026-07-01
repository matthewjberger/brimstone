//! Hand-authored level definitions. Each level is a distinct *place* - its own
//! footprint, room/corridor architecture, height structure, spawn, and exit
//! gate - not a reskin of one arena. Walls (full height) partition space into
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
    /// Emissive structural block - reactor cores, altars, glowing machinery.
    Core,
}

impl BlockKind {
    /// The editor's placeable palette, in display order (Core is hand-authored only).
    pub const ALL: [BlockKind; 6] = [
        BlockKind::Wall,
        BlockKind::Pillar,
        BlockKind::Platform,
        BlockKind::Cover,
        BlockKind::Choke,
        BlockKind::Monument,
    ];

    /// Wire order for [`BlockKind::code`] / [`BlockKind::from_code`]. The single
    /// source of truth for on-disk block codes: both directions derive from it, so
    /// a new variant can never desync the two halves of the mapping. Append only -
    /// reordering rewrites the meaning of every saved level file.
    const ORDER: [BlockKind; 7] = [
        BlockKind::Wall,
        BlockKind::Pillar,
        BlockKind::Cover,
        BlockKind::Choke,
        BlockKind::Monument,
        BlockKind::Platform,
        BlockKind::Core,
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
        Self::ORDER
            .iter()
            .position(|&kind| kind == self)
            .map(|index| index as u8)
            .unwrap_or(0)
    }

    pub fn from_code(code: u8) -> BlockKind {
        Self::ORDER
            .get(code as usize)
            .copied()
            .unwrap_or(BlockKind::Wall)
    }
}

/// (cx, cy, cz, sx, sy, sz, kind)
pub type BlockSpec = (f32, f32, f32, f32, f32, f32, BlockKind);
/// (x, z, [r, g, b])
pub type BeaconSpec = (f32, f32, [f32; 3]);
/// (cx, cy, cz, sx, sy, sz, pitch_radians, yaw_radians) - a tilted slab you can
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
    /// Power core that unlocks the exit. `None` opens the exit on clear instead.
    pub key: Option<[f32; 3]>,
    pub blocks: &'static [BlockSpec],
    pub ramps: &'static [RampSpec],
    pub beacons: &'static [BeaconSpec],
    pub spawn_points: &'static [(f32, f32)],
    pub pads: &'static [(f32, f32)],
    pub roster: Roster,
}

/// An owned, mutable level - what the in-game editor builds and what custom
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

// Every level is a hub-and-spoke place: a central room with a doorway to each of
// four themed spoke rooms, big corner masses walling the spokes apart, and a
// power core sealed deep in one spoke that unlocks the exit. Walls are cy 5 / sy
// 10 (full height); block dims are full extents. `key` marks the core.

// L0 - THE FOUNDRY: reactor-control hub; machine, coolant, turbine, entry spokes.
const L0_BLOCKS: &[BlockSpec] = &[
    (-9.5, 5.0, 18.0, 13.0, 10.0, 1.5, Wall), // hub S wall (entry doorway x[-3,3])
    (9.5, 5.0, 18.0, 13.0, 10.0, 1.5, Wall),
    (-9.5, 5.0, -14.0, 13.0, 10.0, 1.5, Wall), // hub N wall (turbine doorway)
    (9.5, 5.0, -14.0, 13.0, 10.0, 1.5, Wall),
    (-16.0, 5.0, 10.5, 1.5, 10.0, 15.0, Wall), // hub W wall (machine-bay doorway)
    (-16.0, 5.0, -8.5, 1.5, 10.0, 11.0, Wall),
    (16.0, 5.0, 10.5, 1.5, 10.0, 15.0, Wall), // hub E wall (coolant-bay doorway)
    (16.0, 5.0, -8.5, 1.5, 10.0, 11.0, Wall),
    (-33.0, 4.0, -28.0, 30.0, 8.0, 24.0, Monument), // corner machinery (walls spokes apart)
    (33.0, 4.0, -28.0, 30.0, 8.0, 24.0, Monument),
    (-33.0, 4.0, 30.0, 30.0, 8.0, 20.0, Monument),
    (33.0, 4.0, 30.0, 30.0, 8.0, 20.0, Monument),
    (0.0, 3.0, 2.0, 10.0, 6.0, 8.0, Monument), // reactor housing, top 6
    (0.0, 8.0, 2.0, 4.0, 5.0, 4.0, Core),      // reactor core, top 11
    (8.0, 3.0, 12.0, 1.8, 6.0, 1.8, Pillar),   // hub coolant pillars
    (-8.0, 3.0, 12.0, 1.8, 6.0, 1.8, Pillar),
    (-11.0, 4.0, -30.0, 7.0, 8.0, 7.0, Monument), // north spoke: turbine landmark
    (-11.0, 9.0, -30.0, 3.0, 4.0, 3.0, Core),
    (11.0, 1.0, -28.0, 5.0, 2.0, 4.0, Cover),
    (-40.0, 2.0, -6.0, 8.0, 4.0, 8.0, Platform), // west spoke: machine catwalk, top 4
    (-30.0, 2.5, -6.0, 2.0, 5.0, 2.0, Pillar),
    (-30.0, 0.8, 12.0, 4.0, 1.6, 3.0, Cover),
    (-40.0, 0.6, 8.0, 1.8, 1.2, 1.8, Core), // west spoke: power-core pedestal (key)
    (40.0, 3.0, 2.0, 2.2, 6.0, 2.2, Pillar), // east spoke: coolant towers
    (30.0, 3.0, -6.0, 2.2, 6.0, 2.2, Pillar),
    (30.0, 3.0, 10.0, 2.2, 6.0, 2.2, Pillar),
    (42.0, 2.0, 12.0, 6.0, 4.0, 6.0, Platform), // east spoke: raised deck, top 4
    (30.0, 0.8, 2.0, 4.0, 1.6, 3.0, Cover),
    (0.0, 0.8, 30.0, 6.0, 1.6, 2.0, Cover), // south entry cover
    (11.0, 0.8, 24.0, 3.0, 1.6, 3.0, Cover),
    (-11.0, 0.8, 24.0, 3.0, 1.6, 3.0, Cover),
];
const L0_BEACONS: &[BeaconSpec] = &[
    (0.0, 2.0, [2.2, 0.9, 0.25]),    // reactor
    (-40.0, 8.0, [2.0, 1.4, 0.3]),   // power core, gold
    (-11.0, -30.0, [1.9, 0.7, 0.2]), // turbine
    (30.0, 2.0, [0.2, 1.2, 1.8]),    // coolant, cyan
    (0.0, -37.0, [0.3, 1.7, 0.6]),   // exit, green
    (0.0, 32.0, [0.35, 0.45, 0.8]),  // entry, cold
];
const L0_SPAWNS: &[(f32, f32)] = &[
    (0.0, 8.0),
    (-38.0, 2.0),
    (-30.0, -8.0),
    (38.0, 2.0),
    (30.0, 10.0),
    (0.0, -22.0),
    (11.0, -32.0),
    (0.0, 26.0),
];
const L0_PADS: &[(f32, f32)] = &[(-40.0, -10.0), (42.0, 8.0)];

// L1 - THE SPIRE: atrium hub with a climbable spire; four raised shrine spokes.
const L1_BLOCKS: &[BlockSpec] = &[
    (-9.0, 5.0, 15.0, 12.0, 10.0, 1.5, Wall), // hub walls, doorway each side
    (9.0, 5.0, 15.0, 12.0, 10.0, 1.5, Wall),
    (-9.0, 5.0, -15.0, 12.0, 10.0, 1.5, Wall),
    (9.0, 5.0, -15.0, 12.0, 10.0, 1.5, Wall),
    (-15.0, 5.0, 9.0, 1.5, 10.0, 12.0, Wall),
    (-15.0, 5.0, -9.0, 1.5, 10.0, 12.0, Wall),
    (15.0, 5.0, 9.0, 1.5, 10.0, 12.0, Wall),
    (15.0, 5.0, -9.0, 1.5, 10.0, 12.0, Wall),
    (-30.5, 4.0, -30.5, 27.0, 8.0, 27.0, Monument), // corner monoliths
    (30.5, 4.0, -30.5, 27.0, 8.0, 27.0, Monument),
    (-30.5, 4.0, 30.5, 27.0, 8.0, 27.0, Monument),
    (30.5, 4.0, 30.5, 27.0, 8.0, 27.0, Monument),
    (0.0, 4.0, 0.0, 7.0, 8.0, 7.0, Monument), // spire base, top 8
    (0.0, 11.0, 0.0, 4.5, 6.0, 4.5, Monument), // spire shaft, top 14
    (0.0, 16.0, 0.0, 2.5, 4.0, 2.5, Core),    // crown, top 18
    (9.0, 1.5, 0.0, 4.0, 3.0, 4.0, Platform), // ascent ledges
    (0.0, 2.5, 9.0, 4.0, 5.0, 4.0, Platform),
    (-9.0, 1.5, 0.0, 4.0, 3.0, 4.0, Platform),
    (0.0, 2.0, -30.0, 10.0, 4.0, 8.0, Platform), // north shrine dais, top 4
    (0.0, 0.6, -38.0, 1.8, 1.2, 1.8, Core),      // power-core pedestal (key)
    (-10.0, 2.5, -24.0, 2.0, 5.0, 2.0, Pillar),
    (10.0, 2.5, -24.0, 2.0, 5.0, 2.0, Pillar),
    (-34.0, 2.0, 0.0, 8.0, 4.0, 16.0, Platform), // west shrine balcony, top 4
    (-24.0, 2.5, 8.0, 2.0, 5.0, 2.0, Pillar),
    (-24.0, 2.5, -8.0, 2.0, 5.0, 2.0, Pillar),
    (34.0, 2.0, 0.0, 8.0, 4.0, 16.0, Platform), // east shrine balcony, top 4
    (24.0, 2.5, 8.0, 2.0, 5.0, 2.0, Pillar),
    (24.0, 2.5, -8.0, 2.0, 5.0, 2.0, Pillar),
    (0.0, 0.8, 32.0, 6.0, 1.6, 2.0, Cover), // south entry cover
    (11.0, 0.8, 24.0, 3.0, 1.6, 3.0, Cover),
    (-11.0, 0.8, 24.0, 3.0, 1.6, 3.0, Cover),
];
const L1_BEACONS: &[BeaconSpec] = &[
    (0.0, 0.0, [0.7, 0.35, 1.9]),   // spire crown
    (0.0, -38.0, [2.0, 1.4, 0.3]),  // power core, gold
    (-34.0, 0.0, [0.3, 1.4, 1.3]),  // west shrine
    (34.0, 0.0, [1.7, 0.7, 0.2]),   // east shrine
    (-40.0, 0.0, [0.3, 1.7, 0.6]),  // exit, green
    (0.0, 32.0, [0.35, 0.45, 0.8]), // entry
];
const L1_SPAWNS: &[(f32, f32)] = &[
    (0.0, 8.0),
    (0.0, -26.0),
    (-32.0, 8.0),
    (-32.0, -10.0),
    (32.0, 8.0),
    (32.0, -10.0),
    (0.0, 28.0),
    (10.0, -20.0),
];
const L1_PADS: &[(f32, f32)] = &[
    (9.0, 9.0),
    (-9.0, -9.0),
    (-30.0, 8.0),
    (30.0, -8.0),
    (0.0, -24.0),
];

// L2 - THE WARRENS: crossroads hub; four burial-chamber spokes, core in the east.
const L2_BLOCKS: &[BlockSpec] = &[
    (-8.5, 5.0, 14.0, 11.0, 10.0, 1.5, Wall), // hub walls, doorway each side
    (8.5, 5.0, 14.0, 11.0, 10.0, 1.5, Wall),
    (-8.5, 5.0, -14.0, 11.0, 10.0, 1.5, Wall),
    (8.5, 5.0, -14.0, 11.0, 10.0, 1.5, Wall),
    (-14.0, 5.0, 8.5, 1.5, 10.0, 11.0, Wall),
    (-14.0, 5.0, -8.5, 1.5, 10.0, 11.0, Wall),
    (14.0, 5.0, 8.5, 1.5, 10.0, 11.0, Wall),
    (14.0, 5.0, -8.5, 1.5, 10.0, 11.0, Wall),
    (-32.0, 4.0, -28.0, 32.0, 8.0, 24.0, Monument), // corner tombs
    (32.0, 4.0, -28.0, 32.0, 8.0, 24.0, Monument),
    (-32.0, 4.0, 28.0, 32.0, 8.0, 20.0, Monument),
    (32.0, 4.0, 28.0, 32.0, 8.0, 20.0, Monument),
    (0.0, 2.0, 0.0, 6.0, 4.0, 6.0, Monument), // central tomb, top 4
    (0.0, 5.0, 0.0, 2.5, 3.0, 2.5, Core),
    (-9.0, 4.0, -30.0, 7.0, 8.0, 7.0, Monument), // north spoke: ossuary landmark
    (-9.0, 0.8, -22.0, 4.0, 1.6, 3.0, Cover),
    (9.0, 0.8, -22.0, 4.0, 1.6, 3.0, Cover),
    (0.0, 0.8, 30.0, 6.0, 1.6, 2.0, Cover), // south entry cover
    (11.0, 0.8, 22.0, 3.0, 1.6, 3.0, Cover),
    (-11.0, 0.8, 22.0, 3.0, 1.6, 3.0, Cover),
    (-34.0, 2.5, 0.0, 2.5, 5.0, 2.5, Pillar), // west chamber
    (-40.0, 2.5, -8.0, 2.5, 5.0, 2.5, Pillar),
    (-40.0, 2.5, 8.0, 2.5, 5.0, 2.5, Pillar),
    (-28.0, 0.8, 0.0, 4.0, 1.6, 3.0, Cover),
    (34.0, 2.5, 0.0, 2.5, 5.0, 2.5, Pillar), // east chamber (holds the core)
    (40.0, 2.5, -8.0, 2.5, 5.0, 2.5, Pillar),
    (40.0, 2.5, 8.0, 2.5, 5.0, 2.5, Pillar),
    (38.0, 0.6, -2.0, 1.8, 1.2, 1.8, Core), // power-core pedestal (key)
];
const L2_BEACONS: &[BeaconSpec] = &[
    (0.0, 0.0, [1.5, 0.5, 1.6]),    // central tomb
    (38.0, -2.0, [2.0, 1.4, 0.3]),  // power core, gold (east)
    (-40.0, 0.0, [0.4, 0.6, 1.0]),  // west chamber
    (-9.0, -30.0, [1.6, 0.5, 0.3]), // ossuary
    (7.0, -36.0, [0.3, 1.7, 0.6]),  // exit, green
    (0.0, 30.0, [0.35, 0.45, 0.8]), // entry
];
const L2_SPAWNS: &[(f32, f32)] = &[
    (0.0, 6.0),
    (-34.0, 0.0),
    (-40.0, -8.0),
    (34.0, 0.0),
    (40.0, 8.0),
    (0.0, -22.0),
    (0.0, 24.0),
    (9.0, -30.0),
];
const L2_PADS: &[(f32, f32)] = &[];

// L3 - THE CRUCIBLE: cathedral nave hub; chapel spokes, throne hall, boss.
const L3_BLOCKS: &[BlockSpec] = &[
    (-8.5, 5.0, 16.0, 11.0, 10.0, 1.5, Wall), // hub walls, doorway each side
    (8.5, 5.0, 16.0, 11.0, 10.0, 1.5, Wall),
    (-8.5, 5.0, -16.0, 11.0, 10.0, 1.5, Wall),
    (8.5, 5.0, -16.0, 11.0, 10.0, 1.5, Wall),
    (-14.0, 5.0, 10.0, 1.5, 10.0, 12.0, Wall),
    (-14.0, 5.0, -10.0, 1.5, 10.0, 12.0, Wall),
    (14.0, 5.0, 10.0, 1.5, 10.0, 12.0, Wall),
    (14.0, 5.0, -10.0, 1.5, 10.0, 12.0, Wall),
    (-30.0, 4.0, -32.0, 28.0, 8.0, 28.0, Monument), // corner masses
    (30.0, 4.0, -32.0, 28.0, 8.0, 28.0, Monument),
    (-30.0, 4.0, 32.0, 28.0, 8.0, 28.0, Monument),
    (30.0, 4.0, 32.0, 28.0, 8.0, 28.0, Monument),
    (-8.0, 3.5, 10.0, 2.5, 7.0, 2.5, Pillar), // nave colonnade
    (8.0, 3.5, 10.0, 2.5, 7.0, 2.5, Pillar),
    (-8.0, 3.5, 0.0, 2.5, 7.0, 2.5, Pillar),
    (8.0, 3.5, 0.0, 2.5, 7.0, 2.5, Pillar),
    (-8.0, 3.5, -10.0, 2.5, 7.0, 2.5, Pillar),
    (8.0, 3.5, -10.0, 2.5, 7.0, 2.5, Pillar),
    (0.0, 1.5, -30.0, 16.0, 3.0, 7.0, Platform), // throne dais, top 3
    (0.0, 4.5, -34.0, 5.0, 6.0, 3.0, Monument),  // throne back
    (-9.0, 5.0, -32.0, 3.5, 10.0, 3.5, Monument), // monoliths
    (9.0, 5.0, -32.0, 3.5, 10.0, 3.5, Monument),
    (0.0, 7.0, -34.0, 2.2, 3.0, 2.2, Core), // throne crown glow
    (0.0, 0.8, 34.0, 6.0, 1.6, 2.0, Cover), // south entry cover
    (11.0, 0.8, 26.0, 3.0, 1.6, 3.0, Cover),
    (-11.0, 0.8, 26.0, 3.0, 1.6, 3.0, Cover),
    (-34.0, 2.0, 0.0, 8.0, 4.0, 10.0, Platform), // west chapel dais, top 4
    (-38.0, 0.6, 0.0, 1.8, 1.2, 1.8, Core),      // power-core pedestal (key)
    (-24.0, 3.5, 8.0, 2.5, 7.0, 2.5, Pillar),
    (-24.0, 3.5, -8.0, 2.5, 7.0, 2.5, Pillar),
    (34.0, 2.0, 0.0, 8.0, 4.0, 10.0, Platform), // east chapel dais, top 4
    (38.0, 4.0, 0.0, 3.0, 4.0, 3.0, Monument),  // east altar
    (24.0, 3.5, 8.0, 2.5, 7.0, 2.5, Pillar),
    (24.0, 3.5, -8.0, 2.5, 7.0, 2.5, Pillar),
];
const L3_BEACONS: &[BeaconSpec] = &[
    (0.0, -34.0, [2.3, 0.5, 0.18]), // throne, red
    (-38.0, 0.0, [2.0, 1.4, 0.3]),  // power core, gold (west chapel)
    (38.0, 0.0, [0.4, 0.5, 0.9]),   // east chapel
    (0.0, -42.0, [0.3, 1.7, 0.6]),  // exit, green
    (0.0, 34.0, [0.4, 0.45, 0.8]),  // entry
    (0.0, 5.0, [0.5, 0.4, 0.9]),    // nave
];
const L3_SPAWNS: &[(f32, f32)] = &[
    (0.0, -30.0),
    (0.0, 6.0),
    (-34.0, 8.0),
    (-24.0, -8.0),
    (34.0, 8.0),
    (24.0, -8.0),
    (0.0, 28.0),
    (0.0, -18.0),
];
const L3_PADS: &[(f32, f32)] = &[(-30.0, -8.0), (30.0, 8.0)];

const LEVELS: &[Level] = &[
    Level {
        name: "THE FOUNDRY",
        atmosphere: Atmosphere::Sunset,
        fog: [0.10, 0.05, 0.02],
        half_x: 48.0,
        half_z: 40.0,
        spawn: [0.0, 1.2, 36.0],
        exit: [0.0, -37.0],
        key: Some([-40.0, 0.0, 8.0]),
        blocks: L0_BLOCKS,
        ramps: &[],
        beacons: L0_BEACONS,
        spawn_points: L0_SPAWNS,
        pads: L0_PADS,
        roster: Roster {
            imps: 10,
            swarmers: 8,
            casters: 3,
            brutes: 1,
            gargoyles: 1,
            sentinels: 2,
        },
    },
    Level {
        name: "THE SPIRE",
        atmosphere: Atmosphere::Nebula,
        fog: [0.06, 0.03, 0.10],
        half_x: 44.0,
        half_z: 44.0,
        spawn: [0.0, 1.2, 38.0],
        exit: [-40.0, 0.0],
        key: Some([0.0, 0.0, -38.0]),
        blocks: L1_BLOCKS,
        ramps: &[],
        beacons: L1_BEACONS,
        spawn_points: L1_SPAWNS,
        pads: L1_PADS,
        roster: Roster {
            imps: 6,
            swarmers: 6,
            casters: 4,
            brutes: 1,
            gargoyles: 6,
            sentinels: 3,
        },
    },
    Level {
        name: "THE WARRENS",
        atmosphere: Atmosphere::Space,
        fog: [0.02, 0.03, 0.08],
        half_x: 48.0,
        half_z: 40.0,
        spawn: [0.0, 1.2, 36.0],
        exit: [7.0, -36.0],
        key: Some([38.0, 0.0, -2.0]),
        blocks: L2_BLOCKS,
        ramps: &[],
        beacons: L2_BEACONS,
        spawn_points: L2_SPAWNS,
        pads: L2_PADS,
        roster: Roster {
            imps: 9,
            swarmers: 8,
            casters: 4,
            brutes: 2,
            gargoyles: 1,
            sentinels: 3,
        },
    },
    Level {
        name: "THE CRUCIBLE",
        atmosphere: Atmosphere::Nebula,
        fog: [0.10, 0.03, 0.04],
        half_x: 44.0,
        half_z: 46.0,
        spawn: [0.0, 1.2, 40.0],
        exit: [0.0, -42.0],
        key: Some([-38.0, 0.0, 0.0]),
        blocks: L3_BLOCKS,
        ramps: &[],
        beacons: L3_BEACONS,
        spawn_points: L3_SPAWNS,
        pads: L3_PADS,
        roster: Roster {
            imps: 12,
            swarmers: 10,
            casters: 4,
            brutes: 3,
            gargoyles: 3,
            sentinels: 3,
        },
    },
];
