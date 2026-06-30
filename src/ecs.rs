mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::freecs;

freecs::ecs! {
    CobaltWorld {
        engine_entity: EngineEntity => ENGINE_ENTITY,
        enemy: Enemy => ENEMY,
        pickup: Pickup => PICKUP,
    }
    Tags {}
    Events {}
    Resources {
        screen: ScreenState,
        settings: Settings,
        player: PlayerState,
        weapon: WeaponState,
        stats: PlayerStats,
        game: GameState,
        level: LevelState,
        editor: EditorState,
        story: StoryState,
        projectiles: ProjectileState,
        transient: TransientState,
        audio: AudioPool,
        ui_handles: UiHandles,
    }
}
