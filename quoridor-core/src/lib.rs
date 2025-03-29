// --- File: quoridor-project/quoridor-core/src/lib.rs ---

//! # Quoridor Core Library
//!
//! This crate contains the core game logic for the Quoridor board game,
//! including board representation, rules enforcement, pathfinding,
//! and AI strategy implementations. It is designed to be used by other
//! crates, such as a command-line interface, a graphical user interface,
//! or WebAssembly bindings.

// Declare the modules that will make up the core library.
// Rust will look for corresponding files (e.g., game.rs, player.rs)
// or directories (e.g., strategy/mod.rs) within this `src` directory.
pub mod game;
pub mod player;
pub mod types;
pub mod utils;
pub mod graph;
pub mod openings;
pub mod strategy; // This declares the strategy *directory* as a module

// Re-export the most commonly used types and traits for easier access
// by consumers of this library.
pub use game::Quoridor;
pub use player::Player;
pub use types::Coord;
pub use strategy::Strategy;

// Re-export specific strategy implementations
pub use strategy::{
    RandomStrategy,
    ShortestPathStrategy,
    DefensiveStrategy,
    BalancedStrategy,
    AdaptiveStrategy,
    MinimaxStrategy,
    MCTSStrategy,
    MirrorStrategy,
    SimulatedAnnealingStrategy,
};
pub use openings::get_opening_moves; // Make opening function easily available

// Basic test to ensure the library structure compiles
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // This test doesn't do much yet, just confirms compilation.
        assert_eq!(2 + 2, 4);
    }
}