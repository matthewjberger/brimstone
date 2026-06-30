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
    /// Heavy bruiser. Soaks shots, telegraphs a slow slam, caps late waves.
    Brute,
    /// Winged flyer. Hovers above the fray, then dive-bombs from the air.
    Gargoyle,
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
    /// Stronger, larger, brighter variant introduced in later cycles.
    pub elite: bool,
    pub position: Vec3,
    /// Knockback / impulse velocity, decays each frame.
    pub velocity: Vec3,
    pub health: f32,
    pub state: EnemyState,
    pub attack_cooldown: f32,
    pub fire_cooldown: f32,
    /// Wind-up before a strike (melee lunge or caster shot): the dodge window.
    pub windup: f32,
    pub hit_flash: f32,
    pub death_timer: f32,
    /// Animation clock and the material code currently displayed (sentinel 255
    /// forces the first render to set the material).
    pub anim_time: f32,
    pub shown: u8,
    pub strafe_dir: f32,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickupKind {
    #[default]
    Health,
    Ammo,
    Keycard,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Pickup {
    pub position: Vec3,
    pub kind: PickupKind,
    pub bob_phase: f32,
}
