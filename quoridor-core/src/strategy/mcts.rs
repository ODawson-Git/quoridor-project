// --- File: quoridor-project/quoridor-core/src/strategy/mcts.rs ---

use crate::game::Quoridor;
use crate::player::Player;
use crate::strategy::base::QuoridorStrategy;
use crate::strategy::Strategy;
use rand::prelude::*;
use std::cmp::Ordering; // Needed for max_by
use std::{f64, ptr}; // ptr might not be needed if we avoid raw pointers

// --- Platform-specific Timer Handling ---
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

// Define wasm_utils only when compiling for wasm32
#[cfg(target_arch = "wasm32")]
mod wasm_utils {
    // Simple iteration counter as a proxy for time in WASM
    #[derive(Debug, Clone, Copy)]
    pub struct WasmSafeInstant {
        pub iteration_count: usize,
    }
    impl WasmSafeInstant {
        pub fn now() -> Self {
            WasmSafeInstant { iteration_count: 0 }
        }
        // Method to increment and return the count, simulating elapsed "time"
        pub fn elapsed(&mut self) -> usize {
            self.iteration_count += 1;
            self.iteration_count
        }
    }
}
#[cfg(target_arch = "wasm32")]
use wasm_utils::WasmSafeInstant;
// --- End Platform-specific Timer Handling ---

// --- MCTS Node ---
#[derive(Clone)] // Clone needed for game state cloning during simulation
struct MCTSNode {
    move_str: String,       // The move that led to this node's state
    player_to_move: Player, // The player whose turn it is *at* this node's state
    visits: usize,
    wins: f64, // Score accumulated based on simulation wins from this node's player perspective
    children: Vec<MCTSNode>,
    unexpanded_moves: Vec<String>, // Legal moves from this state not yet added as children
}

impl MCTSNode {
    /// Creates a new node representing a game state.
    fn new(move_str: String, player_to_move: Player, legal_moves: Vec<String>) -> Self {
        MCTSNode {
            move_str,
            player_to_move,
            visits: 0,
            wins: 0.0,
            children: Vec::new(),
            unexpanded_moves: legal_moves,
        }
    }

    /// Calculates the UCT value for selecting this node during the Selection phase.
    /// The win rate is calculated from the perspective of the *parent* node's player.
    fn uct_value(&self, parent_visits: usize, exploration_param: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY; // Ensure unvisited nodes are selected first
        }

        // Exploitation term: Average score obtained from simulations starting here.
        // The score (self.wins) is stored relative to the player whose turn it is *at this node* (self.player_to_move).
        // We need the win rate from the perspective of the player whose turn it was at the PARENT.
        // The parent's player is the *opponent* of self.player_to_move.
        let win_rate_for_parent = (self.visits as f64 - self.wins) / self.visits as f64; // Win rate for the opponent of node's player

        // Exploration term: Encourages visiting less explored nodes.
        let exploration = exploration_param
            * ((parent_visits as f64).ln() / (self.visits as f64)).sqrt();

        win_rate_for_parent + exploration
    }

    /// Selects the index of the child with the highest UCT value.
    fn select_best_child_index(&self, exploration_param: f64) -> Option<usize> {
        if self.children.is_empty() {
            return None;
        }
        let parent_visits = self.visits; // Total simulations through the parent (this node)

        self.children
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let uct_a = a.uct_value(parent_visits, exploration_param);
                let uct_b = b.uct_value(parent_visits, exploration_param);
                // Use partial_cmp for f64 comparison, handle NaN/Infinities if necessary
                uct_a.partial_cmp(&uct_b).unwrap_or(Ordering::Equal)
            })
            .map(|(index, _)| index)
    }

    /// Selects the index of the child with the highest visit count (for final move selection).
    fn select_most_visited_child_index(&self) -> Option<usize> {
        if self.children.is_empty() {
            return None;
        }
        self.children
            .iter()
            .enumerate()
            .max_by_key(|(_, child)| child.visits)
            .map(|(index, _)| index)
    }

    /// Adds a new child node after expansion.
    fn add_child(&mut self, move_str: String, player_to_move: Player, legal_moves: Vec<String>) {
        let new_node = MCTSNode::new(move_str, player_to_move, legal_moves);
        self.children.push(new_node);
    }

    /// Updates the node's statistics during backpropagation.
    /// `score`: The score from the simulation (e.g., 10.0 for win, 5.0 for draw, 0.0 for loss)
    ///        relative to the player whose turn it is *at this node*.
    fn update(&mut self, score: f64) {
        self.visits += 1;
        self.wins += score;
    }
}

// --- MCTS Strategy ---

pub struct MCTSStrategy {
    base: QuoridorStrategy,
    simulation_limit: usize,
    exploration_param: f64, // C value in UCT
    #[cfg(not(target_arch = "wasm32"))]
    time_limit: Option<Duration>,
    #[cfg(target_arch = "wasm32")]
    time_limit_iterations: Option<usize>, // Iteration limit proxy for WASM
}

impl MCTSStrategy {
    pub fn new(opening_name: &str, opening_moves: Vec<String>, simulation_limit: usize) -> Self {
        let sim_limit = if simulation_limit == 0 { 1000 } else { simulation_limit };
        let name = format!("MCTS{}", sim_limit); // Base name on sim count
        MCTSStrategy {
            base: QuoridorStrategy::new(&name, opening_name, opening_moves),
            simulation_limit: sim_limit,
            exploration_param: 1.414_f64, // sqrt(2)
            #[cfg(not(target_arch = "wasm32"))]
            time_limit: None,
            #[cfg(target_arch = "wasm32")]
            time_limit_iterations: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        if seconds > 0.0 {
            self.time_limit = Some(Duration::from_secs_f64(seconds));
            // Optionally update the name stored in base if needed
            // self.base.name = format!("MCTS{:.1}s", seconds);
        }
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        if seconds > 0.0 {
            // Crude approximation: iterations = time * simulations_per_second_estimate
            let iterations = (seconds * 50000.0).max(1000.0) as usize; // Example factor
            self.time_limit_iterations = Some(iterations);
            // Optionally update the name stored in base
            // self.base.name = format!("MCTS~{:.1}s", seconds);
        }
        self
    }

    /// Runs the MCTS search for the best move from the given game state.
    fn run_search(&self, game: &Quoridor) -> String {
        let mut rng = thread_rng();
        let root_player = game.active_player; // Player whose turn it is at the root

        // Get initial legal moves
        let legal_pawn = game.get_legal_moves(root_player);
        let legal_walls = game.get_legal_walls(root_player);
        let root_moves: Vec<String> = legal_pawn.into_iter().chain(legal_walls.into_iter()).collect();

        if root_moves.is_empty() { return "resign".to_string(); }
        if root_moves.len() == 1 { return root_moves[0].clone(); }

        // Create the root node representing the current state
        let mut root_node = MCTSNode::new(
            "root".to_string(),
            root_player, // It's this player's turn to move from the root state
            root_moves.clone(),
        );

        let mut simulations_run = 0;
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = Instant::now();
        #[cfg(target_arch = "wasm32")]
        let mut wasm_timer = WasmSafeInstant::now(); // Initialize timer proxy

        // --- Main MCTS Loop ---
        loop {
            // --- Termination Check ---
            simulations_run += 1;
            // Use simulation limit primarily
            if simulations_run > self.simulation_limit { break; }

            // Check time limit secondarily (if applicable)
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(limit) = self.time_limit {
                if start_time.elapsed() >= limit { break; }
            }
            #[cfg(target_arch = "wasm32")]
            if let Some(iter_limit) = self.time_limit_iterations {
                 // Use the elapsed() method which increments the counter
                if wasm_timer.elapsed() >= iter_limit { break; }
            }
            // --- End Termination Check ---


            let mut current_game_sim = game.clone(); // Clone state for this simulation run
            let mut path: Vec<*mut MCTSNode> = vec![&mut root_node]; // Path of *mutable* pointers


            // --- 1. Selection ---
            // Traverse the tree using UCT until a leaf or unexpanded node is found
             loop {
                let current_node_ptr = *path.last().unwrap();
                let current_node = unsafe { &*current_node_ptr }; // Immutable borrow for checks

                if !current_node.unexpanded_moves.is_empty() || current_node.children.is_empty() {
                    // Node is expandable or a leaf node - stop selection
                    break;
                }
                 if self.is_terminal(&current_game_sim) {
                     // Reached terminal state during selection
                      break;
                 }

                // Select the best child using UCT
                 let Some(best_child_idx) = current_node.select_best_child_index(self.exploration_param) else {
                     // Should not happen if children is not empty, but handle defensively
                      break;
                 };

                 // Get mutable reference to the chosen child and add to path
                let next_node_ptr = unsafe { &mut (*current_node_ptr).children[best_child_idx] as *mut MCTSNode };
                path.push(next_node_ptr);

                // Apply the child's move to the simulation game state
                let move_str = &unsafe { &*next_node_ptr }.move_str; // Borrow immutably
                let move_applied = if move_str.len() >= 3 {
                    current_game_sim.add_wall(move_str, false, true)
                } else {
                    current_game_sim.move_pawn(move_str, true)
                };

                if !move_applied {
                    eprintln!("MCTS Error: Failed to apply selected move {} during selection.", move_str);
                    // Backtrack or stop simulation? For now, stop this iteration.
                    break; // Exit inner loop, simulation will proceed from previous state
                }
            } // End Selection loop

            // --- 2. Expansion ---
             let expandable_node_ptr = *path.last().unwrap();
             let expandable_node = unsafe { &mut *expandable_node_ptr };

              // Expand if the node is not terminal and has untried moves
              if !self.is_terminal(&current_game_sim) && !expandable_node.unexpanded_moves.is_empty() {
                  let move_to_expand = expandable_node.unexpanded_moves.remove(rng.gen_range(0..expandable_node.unexpanded_moves.len()));
                   let player_after_expansion = current_game_sim.active_player; // Player *before* applying expansion move

                   // Apply the expansion move
                    let move_applied = if move_to_expand.len() >= 3 {
                        current_game_sim.add_wall(&move_to_expand, false, true)
                    } else {
                        current_game_sim.move_pawn(&move_to_expand, true)
                    };

                    if move_applied {
                          // Get legal moves for the *new* state
                          let new_node_player = current_game_sim.active_player; // Player whose turn it is now
                         let child_moves = if self.is_terminal(&current_game_sim) {
                              Vec::new()
                          } else {
                              let p = current_game_sim.get_legal_moves(new_node_player);
                              let w = current_game_sim.get_legal_walls(new_node_player);
                              p.into_iter().chain(w.into_iter()).collect()
                          };

                          // Add the new child node
                           expandable_node.add_child(move_to_expand, new_node_player, child_moves);
                          let new_child_ptr = expandable_node.children.last_mut().unwrap() as *mut MCTSNode;
                          path.push(new_child_ptr); // Add expanded node to path for backpropagation
                    } else {
                         // If expansion move failed, just simulate from the current state
                         // This might happen if get_legal_moves had an issue earlier
                          eprintln!("MCTS Warning: Failed to apply expansion move {}. Simulating from parent.", move_to_expand);
                    }
              }


            // --- 3. Simulation ---
            // Simulate from the state reached at the end of selection/expansion
             let winner: Option<Player> = self.simulate_random_playout(&mut current_game_sim);

            // --- 4. Backpropagation ---
            // Update nodes along the path with the simulation result
            for node_ptr in path.iter().rev() { // Iterate backwards from leaf to root
                 let node = unsafe { &mut **node_ptr };
                  // The score should be relative to the player whose turn it was *at this node*
                  let score = match winner {
                      Some(winning_player) if winning_player == node.player_to_move => 10.0, // Win
                      Some(_) => 0.0, // Loss
                      None => 5.0, // Draw
                  };
                  node.update(score);
            }

        } // End MCTS loop

        // --- Select Final Move ---
         if let Some(best_child_idx) = root_node.select_most_visited_child_index() {
             // Defensive check: ensure index is valid
              if best_child_idx < root_node.children.len() {
                  root_node.children[best_child_idx].move_str.clone()
              } else {
                  // Fallback if index is somehow out of bounds
                  eprintln!("MCTS Warning: Best child index out of bounds.");
                   root_moves.choose(&mut rng).cloned().unwrap_or_else(|| "resign".to_string())
              }
         } else {
             // Fallback if root has no children explored (should only happen if error or 1 move)
              root_moves.choose(&mut rng).cloned().unwrap_or_else(|| "resign".to_string())
         }
    }

    /// Checks if the game state is terminal (win).
    fn is_terminal(&self, game: &Quoridor) -> bool {
        // Check Player 1 win
        if let Some(p1_pos) = game.pawn_positions.get(&Player::Player1) {
            if p1_pos.0 == 0 { return true; }
        }
        // Check Player 2 win
        if let Some(p2_pos) = game.pawn_positions.get(&Player::Player2) {
            if p2_pos.0 == game.size - 1 { return true; }
        }
        false
    }

    /// Simulates a game using the heuristic from the Mertens paper (page 23).
     fn simulate_random_playout(&self, game_state: &mut Quoridor) -> Option<Player> {
         // No need to clone again if we modify the state passed from run_search directly
         // let mut current_game = game_state.clone();
         let mut current_game = game_state; // Modify the passed mutable state
         let mut rng = thread_rng();
         let max_sim_moves = 150; // Limit simulation length

         for _ in 0..max_sim_moves {
             // Check for terminal state *before* making a move
             if let Some(p1_pos) = current_game.pawn_positions.get(&Player::Player1) { if p1_pos.0 == 0 { return Some(Player::Player1); } }
             if let Some(p2_pos) = current_game.pawn_positions.get(&Player::Player2) { if p2_pos.0 == current_game.size - 1 { return Some(Player::Player2); } }

             let player = current_game.active_player;
             let p_dist = current_game.distance_to_goal(player);
             let o_dist = current_game.distance_to_goal(player.opponent());

             let next_move: Option<String>;

             // Apply Mertens' simulation heuristic
             if p_dist <= o_dist || current_game.walls_available[&player] == 0 {
                 // --- Heuristic Branch 1: Move pawn towards shortest path ---
                 let pawn_moves = current_game.get_legal_moves(player);
                 if !pawn_moves.is_empty() {
                     let mut best_pawn_move = pawn_moves[0].clone(); // Default to first
                     let mut min_dist = p_dist;
                     for mv in &pawn_moves {
                         let mut temp_game = current_game.clone(); // Clone *only* for distance check
                         if temp_game.move_pawn(mv, false) { // Use non-checking move
                             let new_dist = temp_game.distance_to_goal(player);
                             if new_dist < min_dist {
                                 min_dist = new_dist;
                                 best_pawn_move = mv.clone();
                             }
                         }
                     }
                     next_move = Some(best_pawn_move);
                 } else {
                     next_move = None; // No pawn moves possible
                 }
             } else {
                 // --- Heuristic Branch 2: Consider all moves randomly ---
                 let pawn_moves = current_game.get_legal_moves(player);
                 let wall_moves = current_game.get_legal_walls(player);
                 let all_moves: Vec<String> = pawn_moves.into_iter().chain(wall_moves.into_iter()).collect();
                 next_move = all_moves.choose(&mut rng).cloned();
             }

             // Apply the chosen move to the main simulation state
             if let Some(mv_str) = next_move {
                 let moved = if mv_str.len() >= 3 {
                     current_game.add_wall(&mv_str, false, true) // Use checking move in simulation? Paper implies random valid moves. Let's use check=true.
                 } else {
                     current_game.move_pawn(&mv_str, true)
                 };
                 if !moved {
                      // If a chosen "legal" move fails, it indicates a problem. End sim as draw.
                     // eprintln!("Simulation Error: Failed to apply move {}", mv_str);
                     return None;
                 }
             } else {
                 // No legal move available for the current player - opponent wins
                 return Some(player.opponent());
             }
         } // End simulation loop

         None // Draw if max moves reached
     }
} // end impl MCTSStrategy

impl Strategy for MCTSStrategy {
    fn name(&self) -> String {
        // Provide a name reflecting configuration
        let mut name = format!("MCTS{}", self.simulation_limit);
         #[cfg(not(target_arch = "wasm32"))]
         if let Some(limit) = self.time_limit {
              name = format!("MCTS{:.1}s", limit.as_secs_f64());
         }
         #[cfg(target_arch = "wasm32")]
         if let Some(iter_limit) = self.time_limit_iterations {
              let approx_secs = iter_limit as f64 / 50000.0; // Example factor
              name = format!("MCTS~{:.1}s({}i)", approx_secs, iter_limit);
         }
         // Append opening name if one was used
         // format!("{} ({})", name, self.base.opening_name) // Access base struct field? Need pub
         name // Return combined name
    }

    fn choose_move(&mut self, game: &Quoridor) -> Option<String> {
        // Try opening move first
        if let Some(opening_move) = self.base.try_opening_move(game) {
            return Some(opening_move);
        }

        // Run the MCTS search
        let best_move = self.run_search(game);

        if best_move == "resign" {
             None
        } else {
             Some(best_move)
        }
    }
}