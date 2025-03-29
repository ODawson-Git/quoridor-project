// --- File: quoridor-project/quoridor-core/src/strategy/mod.rs ---

//! Defines the core Strategy trait and publicly exports all implemented strategies.

// Use super/crate paths if needed to access core types
use crate::game::Quoridor;
// Potentially use crate::Player;

// --- Strategy Trait ---

/// Defines the interface for all AI playing strategies.
pub trait Strategy: Send + Sync { // Add Send + Sync for potential parallel execution
    /// Returns the name of the strategy (e.g., "Random", "Minimax3").
    fn name(&self) -> String;

    /// Chooses the next move for the active player in the given game state.
    /// Returns the chosen move in algebraic notation (e.g., "e2", "a3h") or None if no move is possible.
    /// Takes `&mut self` to allow strategies to maintain internal state (e.g., opening move counters, MCTS tree).
    fn choose_move(&mut self, game: &Quoridor) -> Option<String>;

    // Optional: Add a method to reset strategy state if needed between games
    // fn reset(&mut self) {}
}


// --- Module Declarations ---
// Declare each strategy implementation file as a submodule.
pub mod adaptive;
pub mod balanced;
pub mod base; // Contains QuoridorStrategy base struct
pub mod defensive;
pub mod mcts;
pub mod minimax;
pub mod mirror;
pub mod random;
pub mod shortest_path;
pub mod simulated_annealing;

// --- Public Exports ---
// Re-export the structs from the submodules so they can be easily used.
pub use adaptive::AdaptiveStrategy;
pub use balanced::BalancedStrategy;
pub use base::QuoridorStrategy; // Base struct might be useful externally too
pub use defensive::DefensiveStrategy;
pub use mcts::MCTSStrategy;
pub use minimax::MinimaxStrategy;
pub use mirror::MirrorStrategy;
pub use random::RandomStrategy;
pub use shortest_path::ShortestPathStrategy;
pub use simulated_annealing::SimulatedAnnealingStrategy;