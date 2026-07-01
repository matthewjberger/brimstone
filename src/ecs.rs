mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::freecs;

freecs::ecs! {
    BrimstoneWorld {
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
        adventure: AdventureState,
        projectiles: ProjectileState,
        transient: TransientState,
        audio: AudioPool,
        viewmodel: ViewmodelState,
        ui_handles: UiHandles,
    }
}
