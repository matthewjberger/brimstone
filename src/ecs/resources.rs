use super::components::EnemyKind;
use crate::tuning;
use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;

/// A queued spawn: which enemy kind, whether it is an elite variant, and
/// whether it is the mission boss (the warlord).
pub type SpawnEntry = (EnemyKind, bool, bool);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Title,
    LevelSelect,
    InGame,
    Paused,
    Editor,
    Cutscene,
    MissionSelect,
}

/// What to do once the player clicks through the current cutscene.
#[derive(Clone, Copy, Default)]
pub enum StoryNext {
    #[default]
    Title,
    StartMission(usize),
}

#[derive(Clone, Default)]
pub struct StorySlide {
    pub title: String,
    pub body: String,
}

/// Story-mode campaign progress and the cutscene currently on screen.
#[derive(Default)]
pub struct StoryState {
    pub active: bool,
    pub mission: usize,
    /// Highest mission index unlocked for replay/continue (persisted to disk).
    pub unlocked: usize,
    pub loaded: bool,
    pub slides: Vec<StorySlide>,
    pub slide_index: usize,
    pub after: StoryNext,
    /// Characters of the current slide body revealed so far (typewriter effect).
    pub reveal: f32,
}

#[derive(Default)]
pub struct ScreenState {
    pub current: Screen,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    /// Wire order for [`Difficulty::code`] / [`Difficulty::from_code`]. The single
    /// source of truth for on-disk codes: both directions derive from it, so a new
    /// variant can never desync the two halves of the mapping.
    const ORDER: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard];

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
        }
    }

    /// Multiplier on damage the player takes.
    pub fn damage_taken(self) -> f32 {
        match self {
            Difficulty::Easy => 0.55,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.6,
        }
    }

    /// Multiplier on enemy health.
    pub fn enemy_health(self) -> f32 {
        match self {
            Difficulty::Easy => 0.8,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.3,
        }
    }

    pub fn next(self) -> Difficulty {
        match self {
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }

    pub fn code(self) -> u8 {
        Self::ORDER
            .iter()
            .position(|&difficulty| difficulty == self)
            .map(|index| index as u8)
            .unwrap_or(0)
    }

    pub fn from_code(code: u8) -> Difficulty {
        Self::ORDER
            .get(code as usize)
            .copied()
            .unwrap_or(Difficulty::Normal)
    }
}

#[derive(Default)]
pub struct Settings {
    pub difficulty: Difficulty,
    pub loaded: bool,
}

#[derive(Default)]
pub struct PlayerState {
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
    pub dash_dir: Vec3,
    pub iframes: f32,
    /// Frames to swallow movement input after a (re)spawn, so the key that
    /// dismissed the briefing cutscene isn't also read as a jump.
    pub spawn_grace: u32,
    /// Wallrun state: which side the wall is on (-1 left, 1 right, 0 none), how
    /// long the run can last, the cooldown after it ends, the wall normal, and
    /// the lerped camera roll currently baked into the view.
    pub wall_run_side: i8,
    pub wall_run_timer: f32,
    pub wall_run_cooldown: f32,
    pub wall_run_normal: Vec3,
    pub wall_run_tilt: f32,
    /// True on frames the sim actually steps (playing and not frozen by
    /// hitstop). The wallrun camera-roll strip (pre_look) and bake
    /// (apply_camera_feel) both key off this single flag so they never desync.
    pub sim_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeaponKind {
    #[default]
    Shotgun,
    Nailgun,
    Rocket,
    Railgun,
    /// Weak infinite-ammo fallback. Always available so a player who burns
    /// through every other pool can never soft-lock — they switch to this and
    /// keep fighting, while its low damage keeps the real guns worth chasing.
    Pistol,
}

/// Per-weapon ammo economy: starting reserve, hard cap, and refill per pickup.
struct AmmoSpec {
    start: u32,
    max: u32,
    pickup: u32,
}

impl WeaponKind {
    /// Every weapon in slot order. Indexing it is the canonical weapon→slot map
    /// (see [`WeaponKind::index`]); iterate it to touch each weapon's ammo pool.
    pub const ALL: [WeaponKind; 5] = [
        WeaponKind::Shotgun,
        WeaponKind::Nailgun,
        WeaponKind::Rocket,
        WeaponKind::Railgun,
        WeaponKind::Pistol,
    ];

    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Shotgun => "SHOTGUN",
            WeaponKind::Nailgun => "NAILGUN",
            WeaponKind::Rocket => "ROCKET",
            WeaponKind::Railgun => "RAILGUN",
            WeaponKind::Pistol => "PISTOL",
        }
    }

    /// This weapon's slot in [`WeaponKind::ALL`] and in the ammo-pool array.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The infinite-ammo sidearm draws from no pool and never depletes.
    pub fn infinite(self) -> bool {
        matches!(self, WeaponKind::Pistol)
    }

    fn ammo_spec(self) -> AmmoSpec {
        match self {
            WeaponKind::Shotgun => AmmoSpec {
                start: tuning::SHOTGUN_START,
                max: tuning::SHOTGUN_MAX,
                pickup: tuning::SHOTGUN_PICKUP,
            },
            WeaponKind::Nailgun => AmmoSpec {
                start: tuning::NAIL_START,
                max: tuning::NAIL_MAX,
                pickup: tuning::NAIL_PICKUP,
            },
            WeaponKind::Rocket => AmmoSpec {
                start: tuning::ROCKET_START,
                max: tuning::ROCKET_MAX,
                pickup: tuning::ROCKET_PICKUP,
            },
            WeaponKind::Railgun => AmmoSpec {
                start: tuning::RAIL_START,
                max: tuning::RAIL_MAX,
                pickup: tuning::RAIL_PICKUP,
            },
            // Infinite sidearm: no reserve, no cap that matters, no pickup share.
            WeaponKind::Pistol => AmmoSpec {
                start: 0,
                max: 0,
                pickup: 0,
            },
        }
    }

    pub fn max_ammo(self) -> u32 {
        self.ammo_spec().max
    }

    pub fn start_ammo(self) -> u32 {
        self.ammo_spec().start
    }

    pub fn pickup_ammo(self) -> u32 {
        self.ammo_spec().pickup
    }
}

/// Each weapon draws from its own ammo pool, so weapon choice is a real
/// resource decision rather than a shared spread-pattern toggle. Pools are keyed
/// by [`WeaponKind::index`].
pub struct WeaponState {
    pub current: WeaponKind,
    pools: [u32; WeaponKind::ALL.len()],
    pub cooldown: f32,
    /// Brief crosshair kick when a shot lands.
    pub hit_marker: f32,
}

impl WeaponState {
    pub fn ammo(&self, kind: WeaponKind) -> u32 {
        self.pools[kind.index()]
    }

    pub fn ammo_mut(&mut self, kind: WeaponKind) -> &mut u32 {
        &mut self.pools[kind.index()]
    }

    /// Add `amount` rounds to `kind`'s pool, clamped to that weapon's cap.
    pub fn add_ammo(&mut self, kind: WeaponKind, amount: u32) {
        let slot = self.ammo_mut(kind);
        *slot = (*slot + amount).min(kind.max_ammo());
    }
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            current: WeaponKind::Shotgun,
            pools: std::array::from_fn(|slot| WeaponKind::ALL[slot].start_ammo()),
            cooldown: 0.0,
            hit_marker: 0.0,
        }
    }
}

pub struct PlayerStats {
    pub health: f32,
    pub max_health: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: tuning::MAX_HEALTH,
            max_health: tuning::MAX_HEALTH,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Playing,
    Dead,
}

#[derive(Default)]
pub struct GameState {
    pub phase: Phase,
    pub score: u32,
    pub best_score: u32,
    pub combo: u32,
    pub combo_timer: f32,
    pub score_flash: f32,
    pub damage_flash: f32,
    pub shake: f32,
    pub cam_kick: f32,
    pub fov_pop: f32,
    pub hitstop: f32,
    pub spawn_timer: f32,
    pub spawn_queue: Vec<SpawnEntry>,
    /// Remaining waves for this level, popped front to back as each clears.
    pub waves: Vec<Vec<SpawnEntry>>,
    /// Seconds since the last kill; feeds the anti-camping pressure build-up.
    pub since_kill: f32,
    pub pressure: f32,
    /// Whether the keycard has been recovered this mission (Keycard objective).
    pub has_key: bool,
    /// The living boss enemy (for the health bar), and its full health.
    pub boss_entity: Option<Entity>,
    pub boss_max_health: f32,
    pub random_state: u64,
    pub seeded: bool,
}

#[derive(Default)]
pub struct LevelState {
    pub index: usize,
    pub cycle: u32,
    pub wave: u32,
    pub wave_count: u32,
    /// Footprint half-extents of the current level (for clamping enemies inside).
    pub half_x: f32,
    pub half_z: f32,
    pub pads: Vec<Vec3>,
    pub geometry: Vec<Entity>,
    pub exit_entity: Option<Entity>,
    pub exit_position: Vec3,
    pub exit_active: bool,
    pub banner: f32,
    /// True while playing a level authored in the editor (single level, its own
    /// spawn points, and the exit returns to the editor rather than cycling).
    pub custom: bool,
    pub custom_spawns: Vec<(f32, f32)>,
    /// True while playing a Story-mode mission; the exit advances the campaign.
    pub story: bool,
    pub objective: crate::campaign::Objective,
}

/// In-editor authoring state: the level being built, its live preview entities,
/// the current brush, and the placement cursor.
#[derive(Default)]
pub struct EditorState {
    pub active: bool,
    pub data: crate::content::LevelData,
    pub block_entities: Vec<Entity>,
    pub markers: Vec<Entity>,
    pub ghost: Option<Entity>,
    pub brush: crate::content::BlockKind,
    pub place_height: f32,
    pub cursor: Vec3,
    pub status: f32,
    pub status_text: String,
}

/// A travelling projectile linked to a billboard render entity. Enemy
/// fireballs hurt the player; player rockets explode and hurt enemies.
pub struct Projectile {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub damage: f32,
    /// True for enemy fireballs (damage the player), false for player rockets.
    pub hostile: bool,
    /// Blast radius for area damage on impact; 0 means a direct hit only.
    pub splash_radius: f32,
}

#[derive(Default)]
pub struct ProjectileState {
    pub items: Vec<Projectile>,
}

/// Short-lived render entities (particle bursts, tracer lines) tracked by
/// time-to-live so they get despawned after their effect plays out.
#[derive(Default)]
pub struct TransientState {
    pub items: Vec<(Entity, f32)>,
}

#[derive(Default)]
pub struct AudioPool {
    pub sources: Vec<(Entity, f32)>,
    pub round_robin: std::collections::HashMap<&'static str, usize>,
}

#[derive(Default)]
pub struct TitleHandles {
    pub root: Entity,
    pub story_button: Entity,
    pub play_button: Entity,
    pub level_select_button: Entity,
    pub editor_button: Entity,
    pub quit_button: Entity,
}

#[derive(Default)]
pub struct LevelSelectHandles {
    pub root: Entity,
    pub level_buttons: Vec<Entity>,
    pub back_button: Entity,
}

#[derive(Default)]
pub struct MissionSelectHandles {
    pub root: Entity,
    pub mission_buttons: Vec<Entity>,
    pub back_button: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct PauseHandles {
    pub root: Entity,
    pub resume_button: Entity,
    pub restart_button: Entity,
    pub difficulty_button: Entity,
    pub difficulty_label: Entity,
    pub main_menu_button: Entity,
    pub quit_button: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Entity,
    pub health_label: Entity,
    pub ammo_label: Entity,
    pub weapon_label: Entity,
    pub ammo_rack: Entity,
    pub wave_label: Entity,
    pub objective_label: Entity,
    pub score_label: Entity,
    pub combo_label: Entity,
    pub boss_panel: Entity,
    pub boss_bar: Entity,
    pub status_panel: Entity,
    pub status_label: Entity,
    pub hint_label: Entity,
    pub crosshair: Entity,
    pub damage_overlay: Entity,
    pub low_health_overlay: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct EditorHandles {
    pub root: Entity,
    pub status: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct CutsceneHandles {
    pub root: Entity,
    pub title: Entity,
    pub body: Entity,
    pub hint: Entity,
}

#[derive(Default)]
pub struct UiHandles {
    pub title: TitleHandles,
    pub level_select: LevelSelectHandles,
    pub mission_select: MissionSelectHandles,
    pub pause: PauseHandles,
    pub hud: HudHandles,
    pub editor: EditorHandles,
    pub cutscene: CutsceneHandles,
}
