//! Boomer — an arcade boomer-shooter built on the nightshade engine.
//!
//! Survive endless escalating waves in an arena: imps body-block, swarmers
//! rush, casters lob fireballs you have to strafe. Two weapons, a dash, and a
//! combo score chase. Architecture follows the nightshade sandbox style: a
//! state shell forwarding to system functions over a user-side ECS world.

mod art;
mod content;
mod ecs;
mod state;
mod systems;
mod theme;
mod tuning;

pub use state::Boomer;
