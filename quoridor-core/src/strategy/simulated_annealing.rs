// --- File: quoridor-project/quoridor-core/src/strategy/simulated_annealing.rs ---

use crate::game::Quoridor;
use crate::player::Player;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::Strategy;
use rand::prelude::*;
use std::f64;

pub struct SimulatedAnnealingStrategy {
    base: QuoridorStrategy,
    // Parameters can be added here, e.g., iteration count, cooling schedule
    // For simplicity, using fixed iterations based on paper's context
    max_global_iterations: usize,
    max_local_iterations: usize,
}

impl SimulatedAnnealingStrategy {
     pub fn new(opening_name: &str, opening_moves: Vec<String>, _time_factor: f64) -> Self {
          // The time_factor isn't directly used in the paper's SA logic description,
          // but we could use it to scale iterations if desired.
          // Paper implies a large number of iterations are run until a condition is met.
          SimulatedAnnealingStrategy {
               base: QuoridorStrategy::new("SimulatedAnnealing", opening_name, opening_moves), // Name doesn't include factor for now
               max_global_iterations: 500, // Example: Limit iterations for performance
               max_local_iterations: 500,  // Example: Limit iterations
          }
     }

     /// Evaluation function based on the Mertens paper's Minimax heuristic (C3: f2+f3-f4).
     /// Higher score is better for Player 1.
     fn evaluate_position(&self, game: &Quoridor) -> f64 {
         let p1_dist = game.distance_to_goal(Player::Player1) as f64;
         let p2_dist = game.distance_to_goal(Player::Player2) as f64;
         let f2_pos_diff = p2_dist - p1_dist; // P2 further = good for P1

         let p1_moves_next = game.moves_to_next_row(Player::Player1) as f64;
         let f3_p1_attack = if p1_moves_next == 0.0 { 100.0 } else { 1.0 / (p1_moves_next + 0.1) };

          let p2_moves_next = game.moves_to_next_row(Player::Player2) as f64;
          let f4_p2_defense = p2_moves_next; // Higher means P2 is slower = good for P1

         const W2: f64 = 0.6001;
         const W3: f64 = 14.45;
         const W4: f64 = 6.52;

          // Score from P1's perspective
          W2 * f2_pos_diff + W3 * f3_p1_attack - W4 * f4_p2_defense
     }

      /// Selects the opponent's best response (minimizing P1's score).
      fn select_opponent_best_move(&self, game: &Quoridor) -> Option<String> {
          let opponent = game.active_player; // Player whose turn it is in this state
          let pawn_moves = game.get_legal_moves(opponent);
          let wall_moves = game.get_legal_walls(opponent);
          let all_moves: Vec<String> = pawn_moves.into_iter().chain(wall_moves.into_iter()).collect();

          if all_moves.is_empty() { return None; }

          let mut best_move: Option<String> = None;
          let mut min_score = f64::INFINITY; // Opponent minimizes P1's score

          for move_str in all_moves {
               let mut next_game = game.clone();
               let moved = if move_str.len() >= 3 {
                    next_game.add_wall(&move_str, false, false)
               } else {
                    next_game.move_pawn(&move_str, false)
               };
                if !moved { continue; }

               let score = self.evaluate_position(&next_game); // Evaluate state after opponent moves
               if score < min_score {
                    min_score = score;
                    best_move = Some(move_str);
               }
          }
          best_move
      }

}


impl Strategy for SimulatedAnnealingStrategy {
    fn name(&self) -> String {
        self.base.name.clone()
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        let player = game.active_player; // The player making the decision *now*
        let opponent = player.opponent();
        let mut rng = thread_rng();
        let e = f64::consts::E;

        let initial_score = self.evaluate_position(game); // Evaluate current state
        let mut best_overall_move: Option<String> = None; // Best first move found

         // Pre-calculate legal moves for the current player
         let player_pawn_moves = game.get_legal_moves(player);
         let player_wall_moves = game.get_legal_walls(player);
         let all_player_moves: Vec<String> = player_pawn_moves.iter().cloned()
             .chain(player_wall_moves.iter().cloned())
             .collect();

         if all_player_moves.is_empty() { return None; } // No moves possible

          // Check for immediate win
          for move_str in &player_pawn_moves {
              if game.win_check(move_str) {
                  return Some(move_str.clone());
              }
          }


        // --- Global Annealing Loop (Choosing the first move) ---
        for time1 in 1..=self.max_global_iterations {
             // 1. Select a candidate first move randomly
             let Some(candidate_first_move) = all_player_moves.choose(&mut rng).cloned() else { continue; };

              // 2. Simulate this move
              let mut game_after_first = game.clone();
              let moved1 = if candidate_first_move.len() >= 3 {
                   game_after_first.add_wall(&candidate_first_move, false, false)
              } else {
                   game_after_first.move_pawn(&candidate_first_move, false)
              };
               if !moved1 { continue; } // Should not happen with legal moves


              // 3. Simulate opponent's *best* response
               let Some(opponent_best_response) = self.select_opponent_best_move(&game_after_first) else {
                    // If opponent has no moves after our first move, we win with first move
                     best_overall_move = Some(candidate_first_move);
                     break;
               };
               let mut game_after_opponent = game_after_first.clone();
               let moved2 = if opponent_best_response.len() >= 3 {
                    game_after_opponent.add_wall(&opponent_best_response, false, false)
               } else {
                    game_after_opponent.move_pawn(&opponent_best_response, false)
               };
                if !moved2 { continue; }


               let score_after_opponent = self.evaluate_position(&game_after_opponent);


              // 4. Local Annealing Loop (Choosing the second move for *us*)
               let mut best_second_move_found: Option<String> = None;
               let player2_pawn_moves = game_after_opponent.get_legal_moves(player);
               let player2_wall_moves = game_after_opponent.get_legal_walls(player);
                let all_second_moves: Vec<String> = player2_pawn_moves.iter().cloned()
                    .chain(player2_wall_moves.iter().cloned())
                    .collect();

               if all_second_moves.is_empty() { continue; } // Cannot respond

              for time2 in 1..=self.max_local_iterations {
                   let Some(candidate_second_move) = all_second_moves.choose(&mut rng).cloned() else { continue; };

                    // Simulate second move
                    let mut game_after_second = game_after_opponent.clone();
                    let moved3 = if candidate_second_move.len() >= 3 {
                         game_after_second.add_wall(&candidate_second_move, false, false)
                    } else {
                         game_after_second.move_pawn(&candidate_second_move, false)
                    };
                     if !moved3 { continue; }

                    let score_after_second = self.evaluate_position(&game_after_second);

                     // Compare score after *our* second move vs score after *opponent's* response
                     // We want to maximize our score (relative to P1's perspective)
                     let delta_e_local = score_after_second - score_after_opponent;

                    if delta_e_local > 0.0 { // Better move found
                         best_second_move_found = Some(candidate_second_move);
                         break; // Found a locally better move
                    } else {
                         // Accept worse move with probability
                         let temp_local = (self.max_local_iterations - time2 + 1) as f64 / self.max_local_iterations as f64; // Example cooling
                          let acceptance_prob = (delta_e_local / temp_local).exp();
                         if rng.gen::<f64>() < acceptance_prob {
                              best_second_move_found = Some(candidate_second_move);
                              break; // Accepted worse move
                         }
                    }
              } // End Local Annealing

                // If local annealing found a response, evaluate the initial move globally
               if best_second_move_found.is_some() {
                    // Evaluate the state *after* our first candidate move
                    let score_after_first = self.evaluate_position(&game_after_first);
                     // Compare score after first move vs initial score
                    let delta_e_global = score_after_first - initial_score;

                    if delta_e_global > 0.0 { // Globally better move
                         best_overall_move = Some(candidate_first_move);
                         break; // Found a globally better move
                    } else {
                         // Accept worse global move with probability
                          let temp_global = (self.max_global_iterations - time1 + 1) as f64 / self.max_global_iterations as f64; // Example cooling
                          let acceptance_prob = (delta_e_global / temp_global).exp();
                         if rng.gen::<f64>() < acceptance_prob {
                              best_overall_move = Some(candidate_first_move);
                              break; // Accepted worse global move
                         }
                    }
               }
               // If local annealing didn't find a response (or we didn't accept), continue global search

        } // End Global Annealing


        // Fallback if SA didn't converge on a move
        if best_overall_move.is_none() {
             all_player_moves.choose(&mut rng).cloned()
        } else {
             best_overall_move
        }
    }
}