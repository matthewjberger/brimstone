use nalgebra_glm::Vec3;
pub use nightshade::ecs::sync::EngineEntity;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyKind {
    /// Slow wall of bodies. Shotgun fodder, herds the player.
    #[default]
    Imp,
    /// Fast, weak, comes in packs. Punishes ignored flanks.
    Swarmer,
    /// Keeps distance and lobs fireballs. The reason you keep moving.
    Caster,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyState {
    #[default]
    Chase,
    Attack,
    Dying,
}

/// State carried by each enemy in the game world. The matching billboard
/// render entity in the engine world is referenced via [`EngineEntity`].
#[derive(Default, Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub position: Vec3,
    /// Knockback / impulse velocity, decays each frame.
    pub velocity: Vec3,
    pub health: f32,
    pub state: EnemyState,
    pub attack_cooldown: f32,
    pub fire_cooldown: f32,
    /// Caster wind-up before a shot, doubles as an attack telegraph.
    pub windup: f32,
    pub hit_flash: f32,
    pub death_timer: f32,
    pub showing_hurt: bool,
    pub strafe_dir: f32,
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
