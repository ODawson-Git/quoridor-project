// --- File: quoridor-project/quoridor-core/src/game.rs ---

//! Contains the main Quoridor game state struct and core rule implementations.

use petgraph::algo::dijkstra;
use crate::types::Coord;
use crate::player::Player;
use crate::utils::{algebraic_to_coord, coord_to_algebraic};
use crate::graph::{self, initialize_board_graph, get_blocked_edges_by_wall, check_wall_path_blocking, get_shortest_path_len}; // Use graph module

use std::collections::{HashMap, HashSet};
use petgraph::graph::{NodeIndex, UnGraph};

/// Represents the state of a Quoridor game.
#[derive(Clone)]
pub struct Quoridor {
    pub size: usize,
    pub walls: usize, // Initial walls per player
    pub graph: UnGraph<Coord, ()>,
    pub node_indices: HashMap<Coord, NodeIndex>,
    // Store wall positions by the bottom-left coord they touch
    pub hwall_positions: HashSet<Coord>,
    pub vwall_positions: HashSet<Coord>,
    pub pawn_positions: HashMap<Player, Coord>,
    pub walls_available: HashMap<Player, usize>,
    pub active_player: Player,
    pub goal_positions: HashMap<Player, Vec<Coord>>,
    // Optional: Keep track of game state history for analysis or UI
    pub state_string: String,
    pub previous_state: String, // State before the last move
    pub last_move: String,      // Last move made (algebraic notation)
}

impl Quoridor {
    /// Creates a new Quoridor game instance.
    /// `state_string`: Optional FEN-like string to load a specific state.
    pub fn new(size: usize, walls: usize, state_string: Option<&str>) -> Self {
        if size < 3 || size % 2 == 0 {
            panic!("Board size must be an odd number >= 3");
        }
        let (graph, node_indices) = initialize_board_graph(size);

        let mut game = Quoridor {
            size,
            walls,
            graph,
            node_indices,
            hwall_positions: HashSet::new(),
            vwall_positions: HashSet::new(),
            pawn_positions: HashMap::new(),
            walls_available: HashMap::new(),
            active_player: Player::Player1, // Player 1 typically starts
            goal_positions: HashMap::new(),
            state_string: String::new(),
            previous_state: String::new(),
            last_move: "None".to_string(),
        };

        // Define goal lines
        game.goal_positions.insert(Player::Player1, (0..size).map(|c| (0, c)).collect()); // Top row for P1
        game.goal_positions.insert(Player::Player2, (0..size).map(|c| (size - 1, c)).collect()); // Bottom row for P2

        // Initialize state
        if let Some(state_str) = state_string {
            game.parse_state_string(state_str); // Load from string
        } else {
            // Default starting positions
            let center = size / 2;
            game.pawn_positions.insert(Player::Player1, (size - 1, center)); // P1 starts at bottom center
            game.pawn_positions.insert(Player::Player2, (0, center));       // P2 starts at top center
            game.walls_available.insert(Player::Player1, walls);
            game.walls_available.insert(Player::Player2, walls);
            game.active_player = Player::Player1;
            game.update_state_string(true); // Generate initial state string
        }

        game
    }

     /// Parses a state string (custom format) and configures the game.
     /// Format: "h_walls/v_walls/p1_pos p2_pos/p1_walls p2_walls/active_player"
     /// Example: "e3f4/b3d5/e1 e9/8 9/1"
     fn parse_state_string(&mut self, state_string: &str) {
         println!("Parsing state string: {}", state_string);
         let parts: Vec<&str> = state_string.split('/').collect();
         if parts.len() != 5 {
             panic!("Invalid state string format: {}", state_string);
         }

         let hwall_str = parts[0].trim();
         let vwall_str = parts[1].trim();
         let pawn_str = parts[2].trim();
         let walls_avail_str = parts[3].trim();
         let active_player_str = parts[4].trim();

         // Reset walls and graph edges before applying loaded state
         self.hwall_positions.clear();
         self.vwall_positions.clear();
         let (new_graph, new_node_indices) = initialize_board_graph(self.size);
         self.graph = new_graph;
         self.node_indices = new_node_indices;


         // --- Parse and apply walls ---
         // Apply horizontal walls
         if !hwall_str.is_empty() {
             for i in (0..hwall_str.len()).step_by(2) {
                 if i + 2 <= hwall_str.len() {
                     let wall_pos_alg = &hwall_str[i..i + 2];
                     let wall_move = format!("{}h", wall_pos_alg);
                     // Use add_wall internally, skipping checks but applying graph changes
                     self.add_wall_internal(&wall_move, true);
                 } else {
                      eprintln!("Warning: Malformed horizontal wall segment '{}' in state string", hwall_str);
                 }
             }
         }
         // Apply vertical walls
         if !vwall_str.is_empty() {
             for i in (0..vwall_str.len()).step_by(2) {
                 if i + 2 <= vwall_str.len() {
                     let wall_pos_alg = &vwall_str[i..i + 2];
                     let wall_move = format!("{}v", wall_pos_alg);
                     self.add_wall_internal(&wall_move, true);
                 } else {
                     eprintln!("Warning: Malformed vertical wall segment '{}' in state string", vwall_str);
                 }
             }
         }

         // --- Parse pawn positions ---
         let pawn_parts: Vec<&str> = pawn_str.split_whitespace().collect();
         if pawn_parts.len() == 2 {
             self.pawn_positions.insert(Player::Player1, self.algebraic_to_coord(pawn_parts[0]));
             self.pawn_positions.insert(Player::Player2, self.algebraic_to_coord(pawn_parts[1]));
         } else {
             panic!("Invalid pawn position format in state string: '{}'", pawn_str);
         }

         // --- Parse walls available ---
         let wall_avail_parts: Vec<&str> = walls_avail_str.split_whitespace().collect();
         if wall_avail_parts.len() == 2 {
             self.walls_available.insert(Player::Player1, wall_avail_parts[0].parse().unwrap_or(self.walls));
             self.walls_available.insert(Player::Player2, wall_avail_parts[1].parse().unwrap_or(self.walls));
         } else {
             panic!("Invalid walls available format in state string: '{}'", walls_avail_str);
         }

         // --- Parse active player ---
         self.active_player = match active_player_str {
             "1" => Player::Player1,
             "2" => Player::Player2,
             _ => panic!("Invalid active player in state string: '{}'", active_player_str),
         };

         // Update the internal state string representation
         self.update_state_string(true); // keep_player = true as we just set it
          println!("Parsed state. Active: {}, P1: {:?}, P2: {:?}, P1W: {}, P2W: {}",
                 self.active_player, self.pawn_positions[&Player::Player1], self.pawn_positions[&Player::Player2],
                 self.walls_available[&Player::Player1], self.walls_available[&Player::Player2]);

     }

      /// Updates the canonical string representation of the game state.
     /// `keep_player`: If true, doesn't switch the active player (used during initialization).
     fn update_state_string(&mut self, keep_player: bool) {
         if !keep_player {
             self.active_player = self.active_player.opponent();
         }

         let player_char = self.active_player.number().to_string();

          // Sort wall positions for consistent string representation
         let mut h_coords: Vec<Coord> = self.hwall_positions.iter().cloned().collect();
         let mut v_coords: Vec<Coord> = self.vwall_positions.iter().cloned().collect();
         // Sort primarily by row (descending for alg row number), then by column (ascending)
         h_coords.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
         v_coords.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));


         let hwall_str: String = h_coords.iter()
             .map(|&pos| self.coord_to_algebraic(pos))
             .collect();
         let vwall_str: String = v_coords.iter()
             .map(|&pos| self.coord_to_algebraic(pos))
             .collect();

         let p1_pos_str = self.coord_to_algebraic(self.pawn_positions[&Player::Player1]);
         let p2_pos_str = self.coord_to_algebraic(self.pawn_positions[&Player::Player2]);

         let p1_walls_str = self.walls_available[&Player::Player1].to_string();
         let p2_walls_str = self.walls_available[&Player::Player2].to_string();

         self.state_string = format!(
             "{} / {} / {} {} / {} {} / {}",
             hwall_str, vwall_str, p1_pos_str, p2_pos_str, p1_walls_str, p2_walls_str, player_char
         );
     }


    /// Helper for coordinate conversion using the utils module.
    pub fn algebraic_to_coord(&self, square: &str) -> Coord {
        algebraic_to_coord(square, self.size)
    }

    /// Helper for coordinate conversion using the utils module.
    pub fn coord_to_algebraic(&self, coord: Coord) -> String {
        coord_to_algebraic(coord, self.size)
    }

    /// Returns a list of valid pawn moves for the given player in algebraic notation.
     pub fn get_legal_moves(&self, player: Player) -> Vec<String> {
         let opponent = player.opponent();
         let Some(own_pos) = self.pawn_positions.get(&player) else { return Vec::new(); }; // Player not found
         let Some(opponent_pos) = self.pawn_positions.get(&opponent) else { return Vec::new(); }; // Opponent not found
         let Some(own_node) = self.node_indices.get(own_pos) else { return Vec::new(); }; // Node not found

         let mut legal_coords = HashSet::new(); // Use HashSet to avoid duplicates

         // Check direct neighbors
         for neighbor_idx in self.graph.neighbors(*own_node) {
             let neighbor_pos = self.graph[neighbor_idx];

             if neighbor_pos == *opponent_pos {
                 // Adjacent to opponent - check for jumps
                 let Some(opponent_node) = self.node_indices.get(opponent_pos) else { continue; };

                 // --- Check straight jump ---
                 // Calculate potential jump destination
                 let jump_r = (own_pos.0 as i32) + 2 * (opponent_pos.0 as i32 - own_pos.0 as i32);
                 let jump_c = (own_pos.1 as i32) + 2 * (opponent_pos.1 as i32 - own_pos.1 as i32);

                 // Check if jump is on board
                 if jump_r >= 0 && jump_r < self.size as i32 && jump_c >= 0 && jump_c < self.size as i32 {
                     let jump_pos = (jump_r as usize, jump_c as usize);
                     if let Some(jump_node) = self.node_indices.get(&jump_pos) {
                         // Check if path from opponent to jump spot is clear (no wall)
                         if self.graph.contains_edge(*opponent_node, *jump_node) {
                              legal_coords.insert(jump_pos);
                              // If straight jump is possible, diagonal jumps are not considered (standard rules)
                              continue; // Go to next neighbor
                         }
                     }
                 }

                 // --- No straight jump possible or blocked - check diagonal jumps ---
                 // Check if opponent is blocked *behind* them (relative to player's jump direction)
                  let jump_blocked = if let Some(jump_node) = self.node_indices.get(&(jump_r as usize, jump_c as usize)) {
                      !self.graph.contains_edge(*opponent_node, *jump_node)
                  } else {
                      true // Off-board is considered blocked
                  };


                 if jump_blocked {
                     // Check opponent's neighbors (potential diagonal jump spots)
                     for op_neighbor_idx in self.graph.neighbors(*opponent_node) {
                         let op_neighbor_pos = self.graph[op_neighbor_idx];
                         // Must be adjacent to opponent and not where the jumping player came from
                         if op_neighbor_pos != *own_pos {
                             legal_coords.insert(op_neighbor_pos);
                         }
                     }
                 }
             } else {
                 // Not adjacent to opponent, direct move is possible
                 legal_coords.insert(neighbor_pos);
             }
         }

         // Convert coordinates to algebraic notation
         legal_coords.iter().map(|&coord| self.coord_to_algebraic(coord)).collect()
     }


    /// Returns a list of valid wall placements for the given player in algebraic notation.
    /// Includes checks for availability, overlap, intersection, and path blocking.
     pub fn get_legal_walls(&self, player: Player) -> Vec<String> {
        if self.walls_available[&player] == 0 {
             return Vec::new(); // No walls left
        }

         let mut legal_walls = Vec::new();
         // Iterate through potential *top-left* coords of wall placement areas
         // Horizontal walls: rows 1 to size-1, cols 0 to size-2
         // Vertical walls: rows 1 to size-1, cols 0 to size-2
         // Note: The coord represents the bottom-left square the wall segment is adjacent to *above* or *to the left of*.
         // So, the algebraic notation is based on this bottom-left square.
         for r in 1..self.size { // Wall placement starts 'between' row 0 and 1 (alg rows 9 and 8)
             for c in 0..self.size - 1 { // Walls are 2 units wide/tall

                  // Check Horizontal Wall Possibility at (r, c) - blocking between row r-1 and r
                  let h_wall_alg = self.coord_to_algebraic((r, c)); // Alg notation for bottom-left square
                  let h_wall_move = format!("{}h", h_wall_alg);
                 if self.is_wall_placement_valid(player, (r, c), 'h') {
                      legal_walls.push(h_wall_move);
                  }

                  // Check Vertical Wall Possibility at (r, c) - blocking between col c and c+1
                   // The algebraic notation refers to the square left of the wall
                   let v_wall_alg = self.coord_to_algebraic((r,c)); // Alg notation uses bottom-left ref
                   let v_wall_move = format!("{}v", v_wall_alg);
                 if self.is_wall_placement_valid(player, (r, c), 'v') {
                      legal_walls.push(v_wall_move);
                  }
             }
         }
         legal_walls
     }

    /// Internal helper to check if placing a specific wall is geometrically valid and doesn't block paths.
     /// `wall_coord`: The bottom-left coordinate the wall is adjacent to (above or left).
     fn is_wall_placement_valid(&self, player: Player, wall_coord: Coord, orientation: char) -> bool {
        // 1. Check walls available (already done in get_legal_walls, but good practice)
        if self.walls_available[&player] == 0 { return false; }

        // 2. Check for overlaps and intersections
        match orientation {
            'h' => {
                // Check direct overlap
                if self.hwall_positions.contains(&wall_coord) { return false; }
                // Check adjacent horizontal overlap (wall is 2 units long)
                 if wall_coord.1 > 0 && self.hwall_positions.contains(&(wall_coord.0, wall_coord.1 - 1)) { return false;}
                 if wall_coord.1 + 1 < self.size -1 && self.hwall_positions.contains(&(wall_coord.0, wall_coord.1 + 1)) { return false; }
                // Check intersection with vertical wall at the same junction
                 if self.vwall_positions.contains(&wall_coord) { return false; }
                 // Need to also check intersection with vertical wall to the right
                  if wall_coord.1 + 1 < self.size {
                     if self.vwall_positions.contains(&(wall_coord.0, wall_coord.1 + 1)) { return false; }
                  }

            }
            'v' => {
                // Check direct overlap
                if self.vwall_positions.contains(&wall_coord) { return false; }
                // Check adjacent vertical overlap
                 if wall_coord.0 > 1 && self.vwall_positions.contains(&(wall_coord.0 - 1, wall_coord.1)) { return false; }
                 if wall_coord.0 + 1 < self.size && self.vwall_positions.contains(&(wall_coord.0 + 1, wall_coord.1)) { return false; }
                // Check intersection with horizontal wall at the same junction
                 if self.hwall_positions.contains(&wall_coord) { return false; }
                // Need to also check intersection with horizontal wall below
                if wall_coord.0 + 1 < self.size {
                     if self.hwall_positions.contains(&(wall_coord.0+1, wall_coord.1)) { return false;}
                }
            }
            _ => return false, // Invalid orientation
        }


        // 3. Check path blocking using a temporary graph modification
        if let Some(edges_to_remove) = get_blocked_edges_by_wall(wall_coord, orientation, self.size) {
            let mut temp_graph = self.graph.clone();
            let mut edges_removed_count = 0;

             for (u_coord, v_coord) in edges_to_remove.iter().filter(|(u,_)| u.0 != usize::MAX) { // Filter out dummy edge for 'v' top row
                 if let (Some(u_idx), Some(v_idx)) = (self.node_indices.get(u_coord), self.node_indices.get(v_coord)) {
                     if let Some(edge_ref) = temp_graph.find_edge(*u_idx, *v_idx) {
                         temp_graph.remove_edge(edge_ref);
                         edges_removed_count += 1;
                     } else {
                          // If an expected edge doesn't exist, the placement is likely invalid due to another wall
                          return false;
                     }
                 } else {
                    // Coordinates not found in graph index - should not happen if coords are valid
                     return false;
                 }
            }

            // If no edges could be found/removed (e.g., blocked by existing walls), it might be invalid placement
            if edges_removed_count == 0 {
                 // Be careful here: sometimes only one edge exists near border
                 if orientation == 'v' && wall_coord.0 == 0 {
                     // Only one edge expected to be removed for top-row vertical wall
                 } else if edges_removed_count < 2 { // Expect 2 edges usually
                     // This might indicate placement conflict
                    // return false; // Re-evaluate if this check is too strict
                 }
            }


            // Check if all players still have a path to their goal line
             check_wall_path_blocking(&temp_graph, &self.node_indices, &self.pawn_positions, &self.goal_positions)

        } else {
             false // Invalid wall coord/orientation for edge calculation
        }
     }

     /// Internal method to add a wall and update graph without checks or changing player state.
     /// Used during state parsing.
     fn add_wall_internal(&mut self, wall_move: &str, is_initialising: bool) -> bool {
         let Some(orientation) = wall_move.chars().last() else { return false; };
         let Some(pos_alg) = wall_move.get(0..wall_move.len()-1) else { return false; };
         let wall_coord = self.algebraic_to_coord(pos_alg);

         // Add to position sets
         match orientation {
             'h' => { self.hwall_positions.insert(wall_coord); },
             'v' => { self.vwall_positions.insert(wall_coord); },
             _ => return false,
         }

         // Remove edges from graph
         if let Some(edges_to_remove) = get_blocked_edges_by_wall(wall_coord, orientation, self.size) {
             for (u_coord, v_coord) in edges_to_remove.iter().filter(|(u,_)| u.0 != usize::MAX) {
                 if let (Some(u_idx), Some(v_idx)) = (self.node_indices.get(u_coord), self.node_indices.get(v_coord)) {
                     if let Some(edge_ref) = self.graph.find_edge(*u_idx, *v_idx) {
                         self.graph.remove_edge(edge_ref);
                     }
                 }
             }
         }
         // Don't update player state if initializing
         if !is_initialising {
              self.previous_state = self.state_string.clone();
              *self.walls_available.get_mut(&self.active_player).unwrap() -= 1;
              self.last_move = wall_move.to_string();
              self.update_state_string(false); // Switch player
         }

         true
     }

    /// Attempts to place a wall. Returns true if successful and legal.
     /// `check`: If true, performs all legality checks.
    pub fn add_wall(&mut self, wall_move: &str, is_initialising: bool, check: bool) -> bool {
         if wall_move.len() < 3 { return false; } // Basic format check
         let Some(orientation) = wall_move.chars().last() else { return false; };
         if orientation != 'h' && orientation != 'v' { return false; }
         let Some(pos_alg) = wall_move.get(0..wall_move.len()-1) else { return false; };
         let wall_coord = self.algebraic_to_coord(pos_alg);

         if check {
             if !self.is_wall_placement_valid(self.active_player, wall_coord, orientation) {
                 return false; // Failed check
             }
         }

         // --- Passed checks or checks skipped ---
         // Add wall to position sets and update graph (internal logic handles this)
         self.add_wall_internal(wall_move, is_initialising)
     }


    /// Attempts to move the active player's pawn. Returns true if successful and legal.
     /// `check`: If true, performs legality checks.
    pub fn move_pawn(&mut self, move_alg: &str, check: bool) -> bool {
        let destination = self.algebraic_to_coord(move_alg);

        if check {
            // Check if destination is in the list of legal moves
            let legal_moves = self.get_legal_moves(self.active_player);
             if !legal_moves.contains(&move_alg.to_string()) {
                // println!("Illegal move attempt: {} not in {:?}", move_alg, legal_moves);
                 return false;
            }
        }

        // Update pawn position
        self.pawn_positions.insert(self.active_player, destination);

        // Update game state history and switch player
        self.previous_state = self.state_string.clone();
        self.last_move = move_alg.to_string();
        self.update_state_string(false); // Switches active player

        true
    }

    /// Checks if the move (represented by the destination coord) is a winning move for the *current* active player.
    pub fn win_check(&self, move_alg: &str) -> bool {
        // --- CORRECTED LOGIC ---
        // A winning move must be a pawn move. Standard pawn moves in algebraic
        // notation (e.g., "e1", "a4") have a length of 2.
        // Wall moves (e.g., "e8h", "a1v") have a length of 3.
        // Therefore, if the move string isn't length 2, it cannot be a winning pawn move.
        if move_alg.len() != 2 {
            return false; // It's a wall move or an invalid format, not a winning pawn move.
        }
        // --- END CORRECTION ---

        // If it might be a pawn move, proceed with the original check:
        let destination = self.algebraic_to_coord(move_alg);
         if let Some(goal_line) = self.goal_positions.get(&self.active_player) {
             // Check if the destination coordinate is within the player's goal line
             goal_line.contains(&destination)
         } else {
             // This case should ideally not happen if goal_positions is always set up correctly.
             eprintln!("Warning: Could not find goal line for player {:?} during win check.", self.active_player);
             false
         }
    }

    /// Calculates the shortest path distance for a player to their goal line.
     /// Returns 100 if no path exists (consistent with paper's heuristic needs).
    pub fn distance_to_goal(&self, player: Player) -> usize {
        if let Some(start_coord) = self.pawn_positions.get(&player) {
             if let Some(goal_coords) = self.goal_positions.get(&player) {
                 let dist = get_shortest_path_len(&self.graph, &self.node_indices, *start_coord, goal_coords);
                 if dist == usize::MAX { 100 } else { dist } // Return 100 if no path
             } else {
                 100 // Goal not defined
             }
         } else {
             100 // Player not found
         }
    }

     /// Calculates the minimum number of pawn moves required for the player to reach *any* square
     /// in the next row towards their goal. Returns 100 if stuck or already at goal line.
     /// (Based on f3/f4 feature from Mertens paper)
    pub fn moves_to_next_row(&self, player: Player) -> usize {
        let Some(start_coord) = self.pawn_positions.get(&player) else { return 100; };
        let Some(start_node) = self.node_indices.get(start_coord) else { return 100; };

        // Determine target row
        let target_row = match player {
            Player::Player1 => { // Moving towards row 0
                 if start_coord.0 == 0 { return 0; } // Already at goal
                 start_coord.0.saturating_sub(1)
            },
            Player::Player2 => { // Moving towards row size-1
                 if start_coord.0 == self.size - 1 { return 0; } // Already at goal
                 (start_coord.0 + 1).min(self.size - 1)
            },
        };

        let mut target_nodes = Vec::new();
        for c in 0..self.size {
             if let Some(node_idx) = self.node_indices.get(&(target_row, c)) {
                 target_nodes.push(*node_idx);
             }
        }

        if target_nodes.is_empty() { return 100; } // Should not happen on valid board

        // Calculate distances to all reachable nodes
         let distances = dijkstra(&self.graph, *start_node, None, |_| 1);

        // Find minimum distance to any node in the target row
         let mut min_dist = usize::MAX;
         for target_node in target_nodes {
             if let Some(dist) = distances.get(&target_node) {
                 min_dist = min_dist.min(*dist);
             }
         }

        if min_dist == usize::MAX { 100 } else { min_dist }
    }
}

// --- Tests for Game Logic ---
#[cfg(test)]
mod game_tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = Quoridor::new(9, 10, None);
        assert_eq!(game.size, 9);
        assert_eq!(game.walls, 10);
        assert_eq!(game.pawn_positions[&Player::Player1], (8, 4));
        assert_eq!(game.pawn_positions[&Player::Player2], (0, 4));
        assert_eq!(game.walls_available[&Player::Player1], 10);
        assert_eq!(game.active_player, Player::Player1);
        assert!(game.state_string.ends_with("/ 1"));
    }

     #[test]
     fn test_pawn_move() {
         let mut game = Quoridor::new(9, 10, None);
         assert_eq!(game.active_player, Player::Player1);
         assert!(game.move_pawn("e2", true)); // P1 moves from e1 to e2
         assert_eq!(game.pawn_positions[&Player::Player1], (7, 4)); // (row 7, col 4)
         assert_eq!(game.active_player, Player::Player2);
         assert!(game.state_string.contains("e2 e9"));
         assert!(game.state_string.ends_with("/ 2"));
         assert_eq!(game.last_move, "e2");

         assert!(game.move_pawn("e8", true)); // P2 moves from e9 to e8
         assert_eq!(game.pawn_positions[&Player::Player2], (1, 4)); // (row 1, col 4)
         assert_eq!(game.active_player, Player::Player1);
         assert!(game.state_string.contains("e2 e8"));
         assert!(game.state_string.ends_with("/ 1"));
         assert_eq!(game.last_move, "e8");
     }

     #[test]
     fn test_illegal_pawn_move() {
          let mut game = Quoridor::new(9, 10, None);
          assert!(!game.move_pawn("e3", true)); // Cannot move 2 squares
          assert_eq!(game.pawn_positions[&Player::Player1], (8, 4)); // Position unchanged
          assert_eq!(game.active_player, Player::Player1); // Player unchanged
     }

     #[test]
      fn test_add_wall() {
          let mut game = Quoridor::new(9, 10, None);
          assert_eq!(game.walls_available[&Player::Player1], 10);
          assert!(game.add_wall("e8h", false, true)); // P1 places wall near P2 start
          assert_eq!(game.walls_available[&Player::Player1], 9);
          assert!(game.hwall_positions.contains(&game.algebraic_to_coord("e8")));
          assert_eq!(game.active_player, Player::Player2);
          assert!(game.state_string.starts_with("e8 /"));
          assert!(game.state_string.ends_with("/ 2"));
          assert_eq!(game.last_move, "e8h");

          // Check graph edge removed (between e8 and e9) - requires coord conversion
          let e8_coord = game.algebraic_to_coord("e8"); // (1, 4)
          let e9_coord = game.algebraic_to_coord("e9"); // (0, 4)
          let f8_coord = game.algebraic_to_coord("f8"); // (1, 5)
          let f9_coord = game.algebraic_to_coord("f9"); // (0, 5)
          let e8_idx = game.node_indices[&e8_coord];
          let e9_idx = game.node_indices[&e9_coord];
          let f8_idx = game.node_indices[&f8_coord];
          let f9_idx = game.node_indices[&f9_coord];
          assert!(game.graph.find_edge(e8_idx, e9_idx).is_none()); // Edge removed
          assert!(game.graph.find_edge(f8_idx, f9_idx).is_none()); // Second part of wall

           assert!(game.add_wall("a1v", false, true)); // P2 places wall near P1 start
           assert_eq!(game.walls_available[&Player::Player2], 9);
           assert!(game.vwall_positions.contains(&game.algebraic_to_coord("a1"))); // Note: alg uses bottom-left ref
           assert_eq!(game.active_player, Player::Player1);

      }

      #[test]
      fn test_wall_intersection() {
          let mut game = Quoridor::new(9, 10, None);
          assert!(game.add_wall("e5h", false, true)); // P1 places horizontal
          assert_eq!(game.active_player, Player::Player2);
          // P2 tries to place vertical intersecting wall
          assert!(!game.add_wall("e5v", false, true));
          assert_eq!(game.walls_available[&Player::Player2], 10); // Wall not placed
          assert_eq!(game.active_player, Player::Player2); // Still P2's turn
      }

       #[test]
       fn test_wall_overlap() {
           let mut game = Quoridor::new(9, 10, None);
           assert!(game.add_wall("e5h", false, true)); // P1 places horizontal
           assert_eq!(game.active_player, Player::Player2);
           // P2 tries to place overlapping horizontal wall
           assert!(!game.add_wall("e5h", false, true)); // Direct overlap
           assert!(!game.add_wall("d5h", false, true)); // Adjacent overlap left
           assert!(!game.add_wall("f5h", false, true)); // Adjacent overlap right
           assert_eq!(game.walls_available[&Player::Player2], 10);
           assert_eq!(game.active_player, Player::Player2);
       }

      #[test]
      fn test_distance_goal() {
           let game = Quoridor::new(9, 10, None);
           assert_eq!(game.distance_to_goal(Player::Player1), 8); // e1 to row 0
           assert_eq!(game.distance_to_goal(Player::Player2), 8); // e9 to row 8

           // Add a wall and check again
           let mut game_walled = Quoridor::new(9, 10, None);
           game_walled.add_wall("e2h", false, true); // Block direct path for P1 and P2
           assert!(game_walled.distance_to_goal(Player::Player1) > 8);
           assert!(game_walled.distance_to_goal(Player::Player2) > 8); // P2 also effected
      }
       #[test]
       fn test_win_check() {
            let mut game = Quoridor::new(9, 10, None);
            game.pawn_positions.insert(Player::Player1, (1, 4)); // P1 at e8
            game.active_player = Player::Player1;
            assert!(game.win_check("e9")); // Moving to e9 (row 0) is a win for P1

            game.pawn_positions.insert(Player::Player2, (7, 4)); // P2 at e2
            game.active_player = Player::Player2;
            assert!(game.win_check("e1")); // Moving to e1 (row 8) is a win for P2
       }

        #[test]
        fn test_legal_moves_simple() {
             let game = Quoridor::new(9, 10, None); // P1 at e1, P2 at e9
             let p1_moves = game.get_legal_moves(Player::Player1);
             assert_eq!(p1_moves.len(), 3); // d1, e2, f1
             assert!(p1_moves.contains(&"d1".to_string()));
             assert!(p1_moves.contains(&"e2".to_string()));
             assert!(p1_moves.contains(&"f1".to_string()));

             let p2_moves = game.get_legal_moves(Player::Player2);
             assert_eq!(p2_moves.len(), 3); // d9, e8, f9
              assert!(p2_moves.contains(&"d9".to_string()));
             assert!(p2_moves.contains(&"e8".to_string()));
             assert!(p2_moves.contains(&"f9".to_string()));
        }

         #[test]
         fn test_legal_moves_jump() {
              // P1 at e5, P2 at e6
              let state = " / / e5 e6 / 10 10 / 1";
              let game = Quoridor::new(9, 10, Some(state));
              let p1_moves = game.get_legal_moves(Player::Player1);
               // Should be able to move d5, f5, e4, and JUMP to e7, this time not d6 or f6
              assert_eq!(p1_moves.len(), 4);
              assert!(p1_moves.contains(&"d5".to_string()));
              assert!(p1_moves.contains(&"f5".to_string()));
              assert!(p1_moves.contains(&"e4".to_string()));
              assert!(p1_moves.contains(&"e7".to_string())); // The jump
              assert!(!p1_moves.contains(&"d6".to_string())); // No diagonal
              assert!(!p1_moves.contains(&"f6".to_string())); // No diagonal
         }

          #[test]
          fn test_legal_moves_jump_blocked() {
               // P1 at e5, P2 at e6, Wall at e6h (blocks jump to e7 and adds side)
               let state = "e6 / / e5 e6 / 10 9 / 1";
               let game = Quoridor::new(9, 10, Some(state));
               let p1_moves = game.get_legal_moves(Player::Player1);
                // Should move d5, f5, e4 and JUMP to d6 or f6
               assert_eq!(p1_moves.len(), 5);
               assert!(p1_moves.contains(&"d6".to_string()));
               assert!(p1_moves.contains(&"f6".to_string()));
               assert!(p1_moves.contains(&"d5".to_string()));
               assert!(p1_moves.contains(&"f5".to_string()));
               assert!(p1_moves.contains(&"e4".to_string()));
               assert!(!p1_moves.contains(&"e7".to_string())); // Straight jump blocked
          }
}