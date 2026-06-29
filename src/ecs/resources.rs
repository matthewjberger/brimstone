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
}

pub struct WeaponState {
    pub ammo: u32,
    pub max_ammo: u32,
    pub cooldown: f32,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            ammo: 24,
            max_ammo: 48,
            cooldown: 0.0,
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
    Won,
}

#[derive(Default)]
pub struct GameState {
    pub phase: Phase,
    pub wave: u32,
    pub damage_flash: f32,
}

pub struct Flash {
    pub entity: Entity,
    pub position: Vec3,
    pub timer: f32,
    pub lifetime: f32,
    pub base_scale: f32,
}

#[derive(Default)]
pub struct FlashState {
    pub items: Vec<Flash>,
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
    pub enemies_label: Entity,
    pub wave_label: Entity,
    pub status_label: Entity,
    pub hint_label: Entity,
    pub crosshair: Entity,
    pub damage_overlay: Entity,
}

#[derive(Default)]
pub struct UiHandles {
    pub title: TitleHandles,
    pub pause: PauseHandles,
    pub hud: HudHandles,
}
