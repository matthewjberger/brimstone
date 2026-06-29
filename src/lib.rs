//! Boomer — a boomer-shooter vertical slice built on the nightshade engine.
//!
//! ## Architecture
//!
//! - `src/state.rs` — `Boomer` struct + `State` trait impl. The state shell owns
//!   the user-side ECS world and forwards each frame to system functions.
//! - `src/ecs.rs` — declares `BoomerWorld` (a [`freecs`] world) with the game
//!   components and resources.
//! - `src/ecs/components.rs` — `Enemy` / `Pickup` components, linked to their
//!   billboard render entities via `EngineEntity`.
//! - `src/ecs/resources.rs` — screen, player, weapon, stats, game, ui handles.
//! - `src/systems/` — behavior. `input` and `lifecycle` drive screens; `screens`
//!   builds the title/pause/hud UI; `world` holds the gameplay systems.

mod art;
mod ecs;
mod state;
mod systems;
mod theme;

pub use state::Boomer;
