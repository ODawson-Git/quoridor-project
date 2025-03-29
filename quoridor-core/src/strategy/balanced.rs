// --- File: quoridor-project/quoridor-core/src/strategy/balanced.rs ---

use crate::game::Quoridor;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::{Strategy, DefensiveStrategy, ShortestPathStrategy};
use rand::prelude::*;

pub struct BalancedStrategy {
    base: QuoridorStrategy,
    defense_weight: f64, // Probability to play defensively (place wall)
    // Internal strategies for decision making
    defensive_strategy: DefensiveStrategy,
    offensive_strategy: ShortestPathStrategy,
}

impl BalancedStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>, defense_weight: f64) -> Self {
         // Inner strategies don't need opening info directly
        let defensive_strategy = DefensiveStrategy::new("", Vec::new(), 1.0); // Use preference 1.0 inside
        let offensive_strategy = ShortestPathStrategy::new("", Vec::new());

        BalancedStrategy {
            base: QuoridorStrategy::new("Balanced", opening_name, opening_moves),
            defense_weight,
            defensive_strategy,
            offensive_strategy,
        }
    }
}

impl Strategy for BalancedStrategy {
    fn name(&self) -> String {
         // format!("{}-W{}", self.base.name.clone(), self.defense_weight)
        self.base.name.clone()
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        let player = game.active_player;
        let mut rng = thread_rng();

        // Decide whether to attempt a defensive wall placement or an offensive pawn move
        if game.walls_available[&player] > 0 && rng.gen::<f64>() < self.defense_weight {
             // Try defensive move. DefensiveStrategy internally handles if no good wall is found.
             // It will fall back to its offensive_strategy (ShortestPath) if needed.
            self.defensive_strategy.choose_move(game)
        } else {
            // Play offensively (move pawn using ShortestPath)
            self.offensive_strategy.choose_move(game)
        }
    }
}