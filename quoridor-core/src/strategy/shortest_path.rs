// --- File: quoridor-project/quoridor-core/src/strategy/shortest_path.rs ---

use crate::game::Quoridor;
use crate::player::Player;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::Strategy;

pub struct ShortestPathStrategy {
    base: QuoridorStrategy,
}

impl ShortestPathStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>) -> Self {
        ShortestPathStrategy {
            base: QuoridorStrategy::new("ShortestPath", opening_name, opening_moves),
        }
    }
}

impl Strategy for ShortestPathStrategy {
    fn name(&self) -> String {
        self.base.name.clone()
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        // Always choose the legal pawn move that results in the shortest path to the goal.
        let legal_pawn_moves = game.get_legal_moves(game.active_player);
        if legal_pawn_moves.is_empty() {
            // If no pawn moves, check for wall moves as fallback? Or just return None?
            // Standard shortest path usually doesn't place walls. Let's return None if no pawn moves.
             return None;
        }

        let player = game.active_player;
        let mut best_move: Option<String> = None;
        let mut min_distance = usize::MAX;

        for move_str in &legal_pawn_moves {
             // Check for immediate win first
             if game.win_check(move_str) {
                  return Some(move_str.clone());
             }

            // Simulate the move by creating a temporary game state
            let mut temp_game = game.clone();
            if temp_game.move_pawn(move_str, false) { // Use internal move, skipping checks
                let distance = temp_game.distance_to_goal(player);
                if distance < min_distance {
                    min_distance = distance;
                    best_move = Some(move_str.clone());
                }
            }
        }

         // Fallback if no move improved distance (should usually not happen unless blocked)
         if best_move.is_none() && !legal_pawn_moves.is_empty() {
              best_move = Some(legal_pawn_moves[0].clone()); // Just take the first legal move
         }


        best_move
    }
}