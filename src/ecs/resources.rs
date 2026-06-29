use super::components::EnemyKind;
use crate::tuning;
use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Title,
    InGame,
    Paused,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeaponKind {
    #[default]
    Shotgun,
    Nailgun,
}

impl WeaponKind {
    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Shotgun => "SHOTGUN",
            WeaponKind::Nailgun => "NAILGUN",
        }
    }
}

pub struct WeaponState {
    pub current: WeaponKind,
    pub ammo: u32,
    pub max_ammo: u32,
    pub cooldown: f32,
    /// Brief crosshair kick when a shot lands.
    pub hit_marker: f32,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            current: WeaponKind::Shotgun,
            ammo: tuning::START_AMMO,
            max_ammo: tuning::MAX_AMMO,
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
            health: 100.0,
            max_health: 100.0,
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
    pub spawn_queue: Vec<EnemyKind>,
    pub random_state: u64,
    pub seeded: bool,
}

#[derive(Default)]
pub struct LevelState {
    pub index: usize,
    pub cycle: u32,
    pub geometry: Vec<Entity>,
    pub exit_entity: Option<Entity>,
    pub exit_position: Vec3,
    pub exit_active: bool,
    pub banner: f32,
}

/// A travelling enemy fireball. Linked to a billboard render entity.
pub struct Projectile {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub damage: f32,
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
}

#[derive(Default)]
pub struct TitleHandles {
    pub root: Entity,
    pub play_button: Entity,
    pub quit_button: Entity,
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
    pub wave_label: Entity,
    pub score_label: Entity,
    pub combo_label: Entity,
    pub status_label: Entity,
    pub hint_label: Entity,
    pub crosshair: Entity,
    pub damage_overlay: Entity,
    pub low_health_overlay: Entity,
}

#[derive(Default)]
pub struct UiHandles {
    pub title: TitleHandles,
    pub pause: PauseHandles,
    pub hud: HudHandles,
}
