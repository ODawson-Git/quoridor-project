// --- File: quoridor-project/quoridor-core/src/strategy/adaptive.rs ---

use crate::game::Quoridor;
use crate::player::Player;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::{Strategy, DefensiveStrategy, ShortestPathStrategy};

pub struct AdaptiveStrategy {
    base: QuoridorStrategy,
    // Strategies to switch between
    defensive_strategy: DefensiveStrategy,
    offensive_strategy: ShortestPathStrategy,
}

impl AdaptiveStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>) -> Self {
         // Inner strategies don't need opening info directly
        let defensive_strategy = DefensiveStrategy::new("", Vec::new(), 0.7); // Example preference
        let offensive_strategy = ShortestPathStrategy::new("", Vec::new());

        AdaptiveStrategy {
            base: QuoridorStrategy::new("Adaptive", opening_name, opening_moves),
            defensive_strategy,
            offensive_strategy,
        }
    }
}

impl Strategy for AdaptiveStrategy {
    fn name(&self) -> String {
        self.base.name.clone()
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        let player = game.active_player;
        let opponent = player.opponent();

        let player_distance = game.distance_to_goal(player);
        let opponent_distance = game.distance_to_goal(opponent);

        // Basic adaptation: play offensively if closer or equal, defensively if further away
        if player_distance <= opponent_distance {
             // Play offensively
             self.offensive_strategy.choose_move(game)
        } else {
             // Play defensively
             // Defensive strategy will internally decide between wall placement or pawn move
             self.defensive_strategy.choose_move(game)
        }
    }
}