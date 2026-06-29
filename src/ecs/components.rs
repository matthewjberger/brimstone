use nalgebra_glm::Vec3;
pub use nightshade::ecs::sync::EngineEntity;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyState {
    #[default]
    Chase,
    Attack,
    Dying,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpriteFrame {
    #[default]
    Idle,
    Attack,
    Hurt,
}

/// State carried by each imp in the game world. The matching billboard
/// render entity in the engine world is referenced via [`EngineEntity`].
#[derive(Default, Clone, Copy, Debug)]
pub struct Enemy {
    pub position: Vec3,
    pub health: f32,
    pub state: EnemyState,
    pub attack_cooldown: f32,
    pub hit_flash: f32,
    pub death_timer: f32,
    pub sprite: SpriteFrame,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickupKind {
    #[default]
    Health,
    Ammo,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Pickup {
    pub position: Vec3,
    pub kind: PickupKind,
    pub bob_phase: f32,
}
