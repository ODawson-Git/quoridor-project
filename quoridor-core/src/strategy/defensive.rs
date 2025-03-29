// --- File: quoridor-project/quoridor-core/src/strategy/defensive.rs ---

use crate::game::Quoridor;
use crate::player::Player;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::{Strategy, ShortestPathStrategy}; // Import base and ShortestPath
use rand::prelude::*;

pub struct DefensiveStrategy {
    base: QuoridorStrategy,
    wall_preference: f64, // Probability to prefer placing a wall
    // Internal strategy for pawn movement when not placing a wall
    offensive_strategy: ShortestPathStrategy,
}

impl DefensiveStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>, wall_preference: f64) -> Self {
        // Ensure the offensive strategy doesn't use openings itself
        let offensive_strategy = ShortestPathStrategy::new("", Vec::new());
        DefensiveStrategy {
            base: QuoridorStrategy::new("Defensive", opening_name, opening_moves),
            wall_preference,
            offensive_strategy,
        }
    }
}

impl Strategy for DefensiveStrategy {
    fn name(&self) -> String {
        // You might want to include the wall_preference in the name if it varies
        // format!("{}-P{}", self.base.name.clone(), self.wall_preference)
        self.base.name.clone()
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        let player = game.active_player;
        let opponent = player.opponent();
        let mut rng = thread_rng();

        let legal_wall_moves = game.get_legal_walls(player); // Checks availability

        // Decide whether to consider placing a wall
        if !legal_wall_moves.is_empty() && rng.gen::<f64>() < self.wall_preference {
            let current_opponent_distance = game.distance_to_goal(opponent);
            let mut best_blocking_wall: Option<String> = None;
            let mut max_distance_increase = 0; // Find wall that hinders opponent most

            for wall_move in &legal_wall_moves {
                let mut temp_game = game.clone();
                // Use internal add_wall without checks, assuming get_legal_walls was correct
                if temp_game.add_wall(wall_move, false, false) {
                     let new_opponent_distance = temp_game.distance_to_goal(opponent);
                     // Ensure opponent is not completely blocked (handled by get_legal_walls check)
                     if new_opponent_distance > current_opponent_distance {
                         let increase = new_opponent_distance.saturating_sub(current_opponent_distance);
                         if increase > max_distance_increase {
                              max_distance_increase = increase;
                              best_blocking_wall = Some(wall_move.clone());
                         }
                     }
                }
            }

            // If a good blocking wall was found, place it
            if best_blocking_wall.is_some() {
                return best_blocking_wall;
            }
            // If no wall increased distance, fall through to pawn move
        }

        // If not placing a wall (or no good wall found), use the offensive strategy for pawn movement
        self.offensive_strategy.choose_move(game)
    }
}