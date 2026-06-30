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
}

#[derive(Default)]
pub struct ScreenState {
    pub current: Screen,
}

#[derive(Default)]
pub struct PlayerState {
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
    pub dash_dir: Vec3,
    pub iframes: f32,
    /// Wallrun state: which side the wall is on (-1 left, 1 right, 0 none), how
    /// long the run can last, the cooldown after it ends, the wall normal, and
    /// the lerped camera roll currently baked into the view.
    pub wall_run_side: i8,
    pub wall_run_timer: f32,
    pub wall_run_cooldown: f32,
    pub wall_run_normal: Vec3,
    pub wall_run_tilt: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeaponKind {
    #[default]
    Shotgun,
    Nailgun,
    Rocket,
}

impl WeaponKind {
    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Shotgun => "SHOTGUN",
            WeaponKind::Nailgun => "NAILGUN",
            WeaponKind::Rocket => "ROCKET",
        }
    }
}

/// Each weapon draws from its own ammo pool, so weapon choice is a real
/// resource decision rather than a shared spread-pattern toggle.
pub struct WeaponState {
    pub current: WeaponKind,
    pub shells: u32,
    pub nails: u32,
    pub rockets: u32,
    pub cooldown: f32,
    /// Brief crosshair kick when a shot lands.
    pub hit_marker: f32,
}

impl WeaponState {
    pub fn ammo(&self, kind: WeaponKind) -> u32 {
        match kind {
            WeaponKind::Shotgun => self.shells,
            WeaponKind::Nailgun => self.nails,
            WeaponKind::Rocket => self.rockets,
        }
    }

    pub fn ammo_mut(&mut self, kind: WeaponKind) -> &mut u32 {
        match kind {
            WeaponKind::Shotgun => &mut self.shells,
            WeaponKind::Nailgun => &mut self.nails,
            WeaponKind::Rocket => &mut self.rockets,
        }
    }

    pub fn max_ammo(kind: WeaponKind) -> u32 {
        match kind {
            WeaponKind::Shotgun => tuning::SHOTGUN_MAX,
            WeaponKind::Nailgun => tuning::NAIL_MAX,
            WeaponKind::Rocket => tuning::ROCKET_MAX,
        }
    }
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            current: WeaponKind::Shotgun,
            shells: tuning::SHOTGUN_START,
            nails: tuning::NAIL_START,
            rockets: tuning::ROCKET_START,
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

#[derive(Default)]
pub struct PauseHandles {
    pub root: Entity,
    pub resume_button: Entity,
    pub restart_button: Entity,
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
    pub boss_bar: Entity,
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
