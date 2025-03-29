// --- File: quoridor-project/quoridor-core/src/strategy/mirror.rs ---

use crate::game::Quoridor;
use crate::player::Player;
use crate::types::Coord;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::{Strategy, AdaptiveStrategy}; // Using Adaptive as a fallback
use crate::utils::abs_diff;
use std::collections::HashSet;

pub struct MirrorStrategy {
    base: QuoridorStrategy,
    backup_strategy: Box<dyn Strategy>, // Fallback strategy
    board_center: Option<(f64, f64)>,   // Cache board center
}

impl MirrorStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>) -> Self {
        MirrorStrategy {
            base: QuoridorStrategy::new("Mirror", opening_name, opening_moves),
            // Use a reasonable fallback like Adaptive, ensuring it doesn't use openings
            backup_strategy: Box::new(AdaptiveStrategy::new("", Vec::new())),
            board_center: None,
        }
    }

    /// Calculates the center of the board once.
    fn get_board_center(&mut self, game: &Quoridor) -> (f64, f64) {
        if self.board_center.is_none() {
            self.board_center = Some(((game.size - 1) as f64 / 2.0, (game.size - 1) as f64 / 2.0));
        }
        self.board_center.unwrap()
    }

    /// Calculates the mirrored position relative to the board center.
    fn calculate_mirrored_coord(&mut self, game: &Quoridor, coord: Coord) -> Coord {
        let center = self.get_board_center(game);
        let mirrored_row = 2.0 * center.0 - coord.0 as f64;
        let mirrored_col = 2.0 * center.1 - coord.1 as f64;

        // Clamp to board boundaries and round
        let row = (mirrored_row.round() as i32).clamp(0, (game.size - 1) as i32) as usize;
        let col = (mirrored_col.round() as i32).clamp(0, (game.size - 1) as i32) as usize;
        (row, col)
    }

    /// Finds the best legal pawn move towards a target coordinate.
    fn find_best_move_towards(&self, game: &Quoridor, target_coord: Coord) -> Option<String> {
        let player = game.active_player;
        let Some(current_pos) = game.pawn_positions.get(&player) else { return None; };
        let legal_moves = game.get_legal_moves(player);

        if legal_moves.is_empty() { return None; }

        let mut best_move: Option<String> = None;
        let mut min_dist_sq = f64::MAX; // Use squared distance to avoid sqrt

        for move_str in &legal_moves {
            let move_coord = game.algebraic_to_coord(move_str);
            let dist_sq = ((move_coord.0 as f64 - target_coord.0 as f64).powi(2) +
                           (move_coord.1 as f64 - target_coord.1 as f64).powi(2));

            // Simple Manhattan distance might be sufficient too:
            // let dist_manhattan = abs_diff(move_coord.0, target_coord.0) + abs_diff(move_coord.1, target_coord.1);

            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_move = Some(move_str.clone());
            }
        }
        best_move
    }

    /// Tries to place a wall mirroring an opponent's wall placement.
    fn find_mirrored_wall_placement(&mut self, game: &Quoridor) -> Option<String> {
        let player = game.active_player;
        if game.walls_available[&player] == 0 { return None; }

        let legal_walls = game.get_legal_walls(player);
        let legal_walls_set: HashSet<String> = legal_walls.iter().cloned().collect();

        // Combine all placed walls for easy lookup
        let mut placed_walls = HashSet::new();
        for &h_wall_coord in &game.hwall_positions {
             placed_walls.insert(format!("{}h", game.coord_to_algebraic(h_wall_coord)));
        }
         for &v_wall_coord in &game.vwall_positions {
              placed_walls.insert(format!("{}v", game.coord_to_algebraic(v_wall_coord)));
         }


        // Check opponent's horizontal walls
        for &opponent_h_wall_coord in &game.hwall_positions {
            let mirrored_coord = self.calculate_mirrored_coord(game, opponent_h_wall_coord);
            let mirrored_wall_move = format!("{}h", game.coord_to_algebraic(mirrored_coord));
            if legal_walls_set.contains(&mirrored_wall_move) && !placed_walls.contains(&mirrored_wall_move) {
                return Some(mirrored_wall_move);
            }
        }

        // Check opponent's vertical walls
        for &opponent_v_wall_coord in &game.vwall_positions {
            let mirrored_coord = self.calculate_mirrored_coord(game, opponent_v_wall_coord);
            let mirrored_wall_move = format!("{}v", game.coord_to_algebraic(mirrored_coord));
             if legal_walls_set.contains(&mirrored_wall_move) && !placed_walls.contains(&mirrored_wall_move) {
                 return Some(mirrored_wall_move);
             }
        }

        None // No suitable mirror wall placement found
    }
}

impl Strategy for MirrorStrategy {
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
        let Some(opponent_pos) = game.pawn_positions.get(&opponent) else { return self.backup_strategy.choose_move(game); };

        // Priority 1: Move towards opponent's mirrored position
        let target_pos = self.calculate_mirrored_coord(game, *opponent_pos);
        if let Some(pawn_move) = self.find_best_move_towards(game, target_pos) {
            // Only move if we are not already at the target
             if game.pawn_positions[&player] != target_pos {
                  return Some(pawn_move);
             }
        }

        // Priority 2: Place a mirrored wall if possible
        if let Some(wall_move) = self.find_mirrored_wall_placement(game) {
            return Some(wall_move);
        }

        // Fallback: Use the backup strategy
        self.backup_strategy.choose_move(game)
    }
}