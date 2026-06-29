mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::freecs;

freecs::ecs! {
    BoomerWorld {
        engine_entity: EngineEntity => ENGINE_ENTITY,
        enemy: Enemy => ENEMY,
        pickup: Pickup => PICKUP,
    }
    Tags {}
    Events {}
    Resources {
        screen: ScreenState,
        player: PlayerState,
        weapon: WeaponState,
        stats: PlayerStats,
        game: GameState,
        flashes: FlashState,
        audio: AudioPool,
        ui_handles: UiHandles,
    }
}
