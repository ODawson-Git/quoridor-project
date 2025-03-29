// --- File: quoridor-project/quoridor-core/src/strategy/random.rs ---

use crate::game::Quoridor;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::Strategy;
use rand::prelude::*;

pub struct RandomStrategy {
    base: QuoridorStrategy,
}

impl RandomStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>) -> Self {
        RandomStrategy {
            base: QuoridorStrategy::new("Random", opening_name, opening_moves),
        }
    }
}

impl Strategy for RandomStrategy {
    fn name(&self) -> String {
        self.base.name.clone()
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        // Otherwise choose randomly from all legal moves
        let legal_pawn_moves = game.get_legal_moves(game.active_player);
        let legal_wall_moves = game.get_legal_walls(game.active_player); // Checks availability internally

        let all_legal_moves: Vec<String> = legal_pawn_moves
            .into_iter()
            .chain(legal_wall_moves.into_iter())
            .collect();

        if all_legal_moves.is_empty() {
            None // No legal moves available
        } else {
            let mut rng = thread_rng();
            // Select a random move from the combined list
            all_legal_moves.choose(&mut rng).cloned()
        }
    }
}