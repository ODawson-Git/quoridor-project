// --- File: quoridor-project/quoridor-core/src/strategy/minimax.rs ---

use crate::game::Quoridor;
use crate::player::Player;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::Strategy;
use std::cmp::Ordering;

pub struct MinimaxStrategy {
    base: QuoridorStrategy,
    depth: usize,
}

impl MinimaxStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>, depth: usize) -> Self {
        if depth == 0 {
            panic!("Minimax depth must be at least 1");
        }
        let name = format!("Minimax{}", depth);
        MinimaxStrategy {
            base: QuoridorStrategy::new(&name, opening_name, opening_moves),
            depth,
        }
    }

    /// Evaluates the current board state from the perspective of the *player whose turn it is*.
    /// Higher scores are better for the current player.
    /// Uses the heuristic (f2+f3+f4 with weights) from the Mertens paper (strategy C3).
    fn evaluate_state(&self, game: &Quoridor) -> f64 {
        let current_player = game.active_player; // Player to potentially move *next*
        let opponent = current_player.opponent();

         // Important: Evaluate based on the state *before* the current player moves.
         // Typically, evaluation functions assess the position itself, not whose turn it is.
         // Let's evaluate from Player 1's perspective consistently, negating if it's P2's turn conceptually in minimax.
         // Or, evaluate relative to the *player who made the move leading to this state*.
         // The Mertens paper heuristic seems relative to the MAX player (higher is better).

        let p1_dist = game.distance_to_goal(Player::Player1) as f64;
        let p2_dist = game.distance_to_goal(Player::Player2) as f64;

        // Heuristic relative to Player 1 (consistent reference)
        // f2: Position difference (opponent distance - player distance)
        let f2_pos_diff = p2_dist - p1_dist;

         // f3: Max-player's (P1) moves to next column (inverted for higher score = better)
         let p1_moves_next = game.moves_to_next_row(Player::Player1) as f64;
         let f3_p1_attack = if p1_moves_next == 0.0 { 100.0 } else { 1.0 / (p1_moves_next + 0.1) }; // Avoid div by zero

         // f4: Min-player's (P2) moves to next column (lower score = better for P1)
          let p2_moves_next = game.moves_to_next_row(Player::Player2) as f64;
          let f4_p2_defense = p2_moves_next; // Higher value means P2 is slower

         // Weights from paper for C3 (f2 + f3 - f4 effectively, as lower f4 is better for Max)
         const W2: f64 = 0.6001;
         const W3: f64 = 14.45;
         const W4: f64 = 6.52; // Weight for opponent's slowness

          let score = W2 * f2_pos_diff + W3 * f3_p1_attack - W4 * f4_p2_defense;

        // Adjust score based on whose turn it *actually* is in the simulation tree
         // If the player who needs to move *from* this state is P1, the score is as is.
         // If the player who needs to move *from* this state is P2, we negate the score because
         // P2 wants to minimize this P1-centric evaluation.
         // Note: The alpha-beta function handles the maximizing/minimizing turns.
         // The evaluation function itself should just return the static score of the position.
         score
    }


    /// Recursive minimax function with alpha-beta pruning.
    fn minimax_alphabeta(
        &self,
        game: &Quoridor,
        depth: usize,
        mut alpha: f64, // Best score MAX player can guarantee
        mut beta: f64,  // Best score MIN player can guarantee
        is_maximizing_player: bool, // Is the current node for the player maximizing the score?
    ) -> f64 {

         // Check terminal conditions: depth limit or game over
          // Check if the *previous* move resulted in a win
          let last_player = game.active_player.opponent(); // Player who just moved
          if let Some(goal_line) = game.goal_positions.get(&last_player) {
               if let Some(last_pos) = game.pawn_positions.get(&last_player) {
                    if goal_line.contains(last_pos) {
                        // Game ended. Return evaluation favoring winner.
                        return if last_player == Player::Player1 { f64::INFINITY } else { f64::NEG_INFINITY }; // P1 maximizes
                    }
               }
          }
          // Check depth limit
          if depth == 0 {
               return self.evaluate_state(game);
          }


        let current_player = game.active_player;
        let legal_pawn_moves = game.get_legal_moves(current_player);
        let legal_wall_moves = game.get_legal_walls(current_player);
        let all_moves: Vec<String> = legal_pawn_moves
            .into_iter()
            .chain(legal_wall_moves.into_iter())
            .collect();

        if all_moves.is_empty() {
            // No moves possible, usually means the other player wins (or draw if reciprocal)
             return if is_maximizing_player { f64::NEG_INFINITY } else { f64::INFINITY };
        }

        if is_maximizing_player { // Player 1 (or the one maximizing the heuristic)
            let mut max_eval = f64::NEG_INFINITY;
            for move_str in all_moves {
                let mut next_game = game.clone();
                let moved = if move_str.len() >= 3 {
                    next_game.add_wall(&move_str, false, false)
                } else {
                    next_game.move_pawn(&move_str, false)
                };
                if !moved { continue; } // Should not happen if get_legal_* works

                let eval = self.minimax_alphabeta(&next_game, depth - 1, alpha, beta, false);
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval); // Update alpha
                if beta <= alpha {
                    break; // Beta cutoff
                }
            }
            max_eval
        } else { // Minimizing player (Player 2)
            let mut min_eval = f64::INFINITY;
            for move_str in all_moves {
                let mut next_game = game.clone();
                 let moved = if move_str.len() >= 3 {
                    next_game.add_wall(&move_str, false, false)
                } else {
                    next_game.move_pawn(&move_str, false)
                };
                 if !moved { continue; }

                let eval = self.minimax_alphabeta(&next_game, depth - 1, alpha, beta, true);
                min_eval = min_eval.min(eval);
                beta = beta.min(eval); // Update beta
                if beta <= alpha {
                    break; // Alpha cutoff
                }
            }
            min_eval
        }
    }
}

impl Strategy for MinimaxStrategy {
    fn name(&self) -> String {
        self.base.name.clone()
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        let current_player = game.active_player;
        let legal_pawn_moves = game.get_legal_moves(current_player);
        let legal_wall_moves = game.get_legal_walls(current_player);

         // Check for immediate wins
         for move_str in &legal_pawn_moves {
             if game.win_check(move_str) {
                 return Some(move_str.clone());
             }
         }

        let all_moves: Vec<String> = legal_pawn_moves
            .into_iter()
            .chain(legal_wall_moves.into_iter())
            .collect();

        if all_moves.is_empty() {
            return None;
        }

        let mut best_move: Option<String> = None;
        let mut best_score = f64::NEG_INFINITY; // Since the current player is maximizing

        // Iterate through possible first moves and evaluate them using minimax
        for move_str in all_moves {
             let mut next_game = game.clone();
             let moved = if move_str.len() >= 3 {
                 next_game.add_wall(&move_str, false, false) // Use internal move for simulation
             } else {
                 next_game.move_pawn(&move_str, false)
             };
              if !moved { continue; } // Skip if somehow illegal

             // Call minimax for the opponent's turn (minimizing player)
             let score = self.minimax_alphabeta(
                 &next_game,
                 self.depth - 1, // Decrease depth
                 f64::NEG_INFINITY,
                 f64::INFINITY,
                 false, // The next turn is for the minimizing player
             );

            if score > best_score {
                best_score = score;
                best_move = Some(move_str);
            }
        }

        // Fallback if no move could be evaluated (shouldn't happen if all_moves is not empty)
        if best_move.is_none() && !game.get_legal_moves(current_player).is_empty() {
            best_move = Some(game.get_legal_moves(current_player)[0].clone())
        } else if best_move.is_none() && !game.get_legal_walls(current_player).is_empty() {
             best_move = Some(game.get_legal_walls(current_player)[0].clone())
        }


        best_move
    }
}