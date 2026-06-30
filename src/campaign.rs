//! The Story-mode campaign: an ordered run of themed missions over the hand-
//! built arenas, each with an objective and narrative framing, stitched
//! together by text cutscenes.

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Objective {
    /// Wipe out every wave, then the gate opens.
    #[default]
    Exterminate,
    /// The gate is open from the start — punch through the horde and escape.
    Reach,
    /// A warlord anchors the final wave; clear the floor to open the gate.
    Boss,
    /// The gate is locked until you find and grab the keycard across the level.
    Keycard,
}

impl Objective {
    pub fn label(self) -> &'static str {
        match self {
            Objective::Exterminate => "EXTERMINATE THE HORDE",
            Objective::Reach => "REACH THE GATE",
            Objective::Boss => "KILL THE WARLORD",
            Objective::Keycard => "RECOVER THE KEYCARD",
        }
    }
}

pub struct Mission {
    /// Index into the static `LEVELS` table whose geometry/theme this mission uses.
    pub level: usize,
    pub title: &'static str,
    pub objective: Objective,
    /// World position of the keycard for `Keycard` missions; ignored otherwise.
    pub key: [f32; 3],
    pub briefing: &'static str,
    pub debrief: &'static str,
}

pub fn count() -> usize {
    CAMPAIGN.len()
}

pub fn mission(index: usize) -> &'static Mission {
    &CAMPAIGN[index.min(CAMPAIGN.len() - 1)]
}

/// Opening cutscene, shown once before the first mission.
pub const INTRO: &[&str] = &[
    "The colony ship GEHENNA went dark over the ring-world eight days ago.\n\nYou are the only marine still breathing.",
    "Something down there turned the crew into the horde now boiling across the decks.\n\nCut a path to the core. Put it down. Get out.",
];

/// Closing cutscene, shown after the final mission.
pub const ENDING: &[&str] = &[
    "The overlord folds in on itself and the ring-world goes quiet.\n\nFor the first time in eight days, nothing is trying to kill you.",
    "GEHENNA drifts, dead and silent, and you are still breathing.\n\nRIP AND TEAR ACCOMPLISHED.",
];

pub const CAMPAIGN: &[Mission] = &[
    Mission {
        level: 0,
        title: "DROP ZONE",
        objective: Objective::Exterminate,
        key: [0.0, 0.0, 0.0],
        briefing: "You hit the hangar deck hard. The first wave is already swarming the beacons. Clear them out and find the way down.",
        debrief: "Hangar secured. The deck below is venting heat — the horde is thickest there.",
    },
    Mission {
        level: 1,
        title: "THE GAUNTLET",
        objective: Objective::Reach,
        key: [0.0, 0.0, 0.0],
        briefing: "The corridor ahead is a kill-channel and the bulkhead won't hold. Don't stand and fight — run the gauntlet and slam the gate behind you.",
        debrief: "Gate sealed. Whatever was herding them is close now.",
    },
    Mission {
        level: 2,
        title: "THE SANCTUM",
        objective: Objective::Keycard,
        key: [12.0, 0.0, -12.0],
        briefing: "The gate out of the sanctum is sealed. The casters guard the keycard in the far corner of the colonnade — go take it, then get to the gate.",
        debrief: "Keycard in hand, the sanctum falls behind you. The deck rises into an old monument hall.",
    },
    Mission {
        level: 4,
        title: "THE ZIGGURAT",
        objective: Objective::Boss,
        key: [0.0, 0.0, 0.0],
        briefing: "A warlord holds the high ground atop the ziggurat, screaming the horde into a frenzy. Climb it. End it.",
        debrief: "The warlord is meat. But the spire beyond is crawling with wings.",
    },
    Mission {
        level: 6,
        title: "ASCENT",
        objective: Objective::Reach,
        key: [0.0, 0.0, 0.0],
        briefing: "The gargoyles own the air around the spire. You can't win this one — you can only climb it. Reach the gate at the top.",
        debrief: "You break through the roost. Below you, the core chamber glows red.",
    },
    Mission {
        level: 3,
        title: "THE CRUCIBLE",
        objective: Objective::Boss,
        key: [0.0, 0.0, 0.0],
        briefing: "This is the core. The overlord is down there, and everything it has left is between you and it. No exit until it's dead.",
        debrief: "Silence.",
    },
];
