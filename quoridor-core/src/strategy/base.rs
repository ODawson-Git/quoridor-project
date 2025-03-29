// --- File: quoridor-project/quoridor-core/src/strategy/base.rs ---

//! Contains a base struct and logic potentially shared by multiple strategies,
//! especially for handling opening moves.

use crate::game::Quoridor;

/// A base struct for strategies, handling opening moves and naming.
pub struct QuoridorStrategy {
    pub name: String, // Made public for access in strategy implementations
    pub opening_moves: Vec<String>, // Made public
    pub move_counter: usize, // Made public
}

impl QuoridorStrategy {
    /// Creates a new base strategy instance.
    pub fn new(base_name: &str, opening_name: &str, opening_moves: Vec<String>) -> Self {
        let full_name = if opening_name.is_empty() || opening_name == "No Opening" || opening_moves.is_empty() {
             base_name.to_string()
        } else {
            // Include opening name only if moves are actually provided for it
            format!("{}-{}", base_name, opening_name)
        };

        QuoridorStrategy {
            name: full_name,
            opening_moves,
            move_counter: 0,
        }
    }

    /// Attempts to return the next opening move if available and legal.
    /// Increments the internal move counter.
    pub fn try_opening_move(&mut self, game: &Quoridor) -> Option<String> {
        if self.move_counter < self.opening_moves.len() {
            let move_str = self.opening_moves[self.move_counter].clone();
            // Crucially, check if the opening move is actually legal in the *current* position
            let legal_pawn = game.get_legal_moves(game.active_player);
            let legal_walls = game.get_legal_walls(game.active_player); // Already checks walls_available

            if legal_pawn.contains(&move_str) || legal_walls.contains(&move_str) {
                self.move_counter += 1; // Only increment if legal and used
                // println!("Using opening move #{}: {} for {}", self.move_counter, move_str, game.active_player);
                return Some(move_str);
            } else {
                 // println!("Skipping illegal opening move #{}: {} for {}", self.move_counter + 1, move_str, game.active_player);
                 // Don't increment counter, let the main strategy logic take over
                 // Or potentially advance counter anyway if opening assumes perfect play? Depends on design.
                 // For robustness, let's not increment if illegal. The strategy will choose a valid move instead.
                 self.move_counter +=1; // Let's assume we should advance past illegal opening moves
                 return None; // Let the main strategy choose
            }
        }
        None // No more opening moves
    }

     /// Resets the opening move counter.
     pub fn reset(&mut self) {
         self.move_counter = 0;
     }
}