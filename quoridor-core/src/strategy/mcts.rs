// --- File: quoridor-project/quoridor-core/src/strategy/mcts.rs ---

use crate::{game::Quoridor, player::Player, strategy::base::QuoridorStrategy, strategy::Strategy};
use rand::prelude::*;
use std::{
    cmp::Ordering,
    collections::HashMap,
    f64, // ptr, // ptr might not be needed if we avoid raw pointers (REMOVED)
    sync::{Arc, Mutex},
};

// --- Platform-specific Timer Handling & Parallelism ---
// #[cfg(not(target_arch = "wasm32"))] // Removed prelude import, using rayon::scope directly
// use rayon::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rayon; // Need the base crate for rayon::scope
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
// --- End Platform-specific Timer Handling & Parallelism ---

// --- MCTS Node ---
#[derive(Clone, Debug)] // Added Debug
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

    // Helper to get child statistics (move_str, visits) - useful for aggregation
    fn get_child_stats(&self) -> Vec<(String, usize)> {
        self.children
            .iter()
            .map(|child| (child.move_str.clone(), child.visits))
            .collect()
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

    // Removed unused method: select_most_visited_child_index

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
    #[cfg(not(target_arch = "wasm32"))]
    num_threads: usize, // Number of threads for parallel execution
    #[cfg(target_arch = "wasm32")]
    time_limit_iterations: Option<usize>, // Iteration limit proxy for WASM
}

impl MCTSStrategy {
    const DEFAULT_SIMULATIONS: usize = 1000;
    const DEFAULT_EXPLORATION: f64 = 1.414; // sqrt(2)
    #[cfg(target_arch = "wasm32")]
    const WASM_ITER_PER_SEC_ESTIMATE: f64 = 50000.0; // Adjust as needed

    pub fn new(opening_name: &str, opening_moves: Vec<String>, simulation_limit: usize) -> Self {
        let sim_limit = if simulation_limit == 0 {
            Self::DEFAULT_SIMULATIONS
        } else {
            simulation_limit
        };
        let name = format!("MCTS{}", sim_limit); // Base name on sim count
        MCTSStrategy {
            base: QuoridorStrategy::new(&name, opening_name, opening_moves),
            simulation_limit: sim_limit,
            exploration_param: Self::DEFAULT_EXPLORATION,
            #[cfg(not(target_arch = "wasm32"))]
            time_limit: None,
            #[cfg(not(target_arch = "wasm32"))]
            num_threads: rayon::current_num_threads().max(1), // Default to available threads
            #[cfg(target_arch = "wasm32")]
            time_limit_iterations: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        if seconds > 0.0 {
            self.time_limit = Some(Duration::from_secs_f64(seconds));
            // Name update handled in name() method
        }
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_num_threads(mut self, threads: usize) -> Self {
        self.num_threads = threads.max(1); // Ensure at least one thread
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        if seconds > 0.0 {
            // Crude approximation: iterations = time * simulations_per_second_estimate
            let iterations = (seconds * Self::WASM_ITER_PER_SEC_ESTIMATE)
                .max(Self::DEFAULT_SIMULATIONS as f64) as usize;
            self.time_limit_iterations = Some(iterations);
            // Name update handled in name() method
        }
        self
    }

    /// Runs a batch of MCTS simulations starting from a given root node and game state.
    /// Returns the statistics (move, visits) of the children of the root node after the simulations.
    /// This function contains the core MCTS loop (Select, Expand, Simulate, Backprop).
    fn run_mcts_simulation_batch(
        &self,
        initial_game: &Quoridor,
        simulations_to_run: usize,
        #[cfg(not(target_arch = "wasm32"))] time_limit_for_batch: Option<Duration>,
        #[cfg(target_arch = "wasm32")] iteration_limit_for_batch: Option<usize>,
    ) -> Vec<(String, usize)> {
        let mut rng = thread_rng();
        let root_player = initial_game.active_player;

        let legal_pawn = initial_game.get_legal_moves(root_player);
        let legal_walls = initial_game.get_legal_walls(root_player);
        let root_moves: Vec<String> = legal_pawn
            .into_iter()
            .chain(legal_walls.into_iter())
            .collect();

        // If no moves or only one move, no search needed for this batch
        if root_moves.is_empty() {
            return vec![("resign".to_string(), 1)];
        } // Return dummy stats
        if root_moves.len() == 1 {
            return vec![(root_moves[0].clone(), 1)];
        } // Return dummy stats

        let mut root_node = MCTSNode::new("root".to_string(), root_player, root_moves.clone());

        let mut simulations_run_in_batch = 0;
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = Instant::now();
        #[cfg(target_arch = "wasm32")]
        let mut wasm_timer = WasmSafeInstant::now();

        // --- MCTS Loop for this Batch ---
        loop {
            // --- Termination Check for Batch ---
            simulations_run_in_batch += 1;
            if simulations_run_in_batch > simulations_to_run {
                break;
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(limit) = time_limit_for_batch {
                if start_time.elapsed() >= limit {
                    break;
                }
            }
            #[cfg(target_arch = "wasm32")]
            if let Some(iter_limit) = iteration_limit_for_batch {
                if wasm_timer.elapsed() >= iter_limit {
                    break;
                }
            }
            // --- End Termination Check ---

            let mut current_game_sim = initial_game.clone(); // Clone state for this simulation run
            let mut path: Vec<*mut MCTSNode> = vec![&mut root_node]; // Path of *mutable* pointers

            // --- 1. Selection ---
            // (Selection logic remains the same as original)
            loop {
                let current_node_ptr = *path.last().unwrap();
                let current_node = unsafe { &*current_node_ptr }; // Immutable borrow for checks

                if !current_node.unexpanded_moves.is_empty() || current_node.children.is_empty() {
                    break; // Node is expandable or a leaf node
                }
                if self.is_terminal(&current_game_sim) {
                    break; // Reached terminal state during selection
                }

                let Some(best_child_idx) = current_node.select_best_child_index(self.exploration_param) else {
                    break; // Should not happen if children is not empty
                };

                let next_node_ptr =
                    unsafe { &mut (*current_node_ptr).children[best_child_idx] as *mut MCTSNode };
                path.push(next_node_ptr);

                let move_str = &unsafe { &*next_node_ptr }.move_str;
                let move_applied = if move_str.len() >= 3 {
                    current_game_sim.add_wall(move_str, false, true)
                } else {
                    current_game_sim.move_pawn(move_str, true)
                };

                if !move_applied {
                    eprintln!(
                        "MCTS Error: Failed to apply selected move {} during selection.",
                        move_str
                    );
                    break;
                }
            } // End Selection loop

            // --- 2. Expansion ---
            // (Expansion logic remains the same as original)
            let expandable_node_ptr = *path.last().unwrap();
            let expandable_node = unsafe { &mut *expandable_node_ptr };

            if !self.is_terminal(&current_game_sim) && !expandable_node.unexpanded_moves.is_empty()
            {
                let move_to_expand = expandable_node
                    .unexpanded_moves
                    .remove(rng.gen_range(0..expandable_node.unexpanded_moves.len()));
                // let player_after_expansion = current_game_sim.active_player; // Not needed directly

                let move_applied = if move_to_expand.len() >= 3 {
                    current_game_sim.add_wall(&move_to_expand, false, true)
                } else {
                    current_game_sim.move_pawn(&move_to_expand, true)
                };

                if move_applied {
                    let new_node_player = current_game_sim.active_player;
                    let child_moves = if self.is_terminal(&current_game_sim) {
                        Vec::new()
                    } else {
                        let p = current_game_sim.get_legal_moves(new_node_player);
                        let w = current_game_sim.get_legal_walls(new_node_player);
                        p.into_iter().chain(w.into_iter()).collect()
                    };

                    expandable_node.add_child(move_to_expand.clone(), new_node_player, child_moves);
                    let new_child_ptr =
                        expandable_node.children.last_mut().unwrap() as *mut MCTSNode;
                    path.push(new_child_ptr);
                } else {
                    eprintln!(
                        "MCTS Warning: Failed to apply expansion move {}. Simulating from parent.",
                        move_to_expand
                    );
                }
            }

            // --- 3. Simulation ---
            // (Simulation logic remains the same as original)
            let winner: Option<Player> = self.simulate_random_playout(&mut current_game_sim);

            // --- 4. Backpropagation ---
            // (Backpropagation logic remains the same as original)
            for node_ptr in path.iter().rev() {
                let node = unsafe { &mut **node_ptr };
                let score = match winner {
                    Some(winning_player) if winning_player == node.player_to_move => 10.0, // Win
                    Some(_) => 0.0,                                                      // Loss
                    None => 5.0,                                                         // Draw
                };
                node.update(score);
            }
        } // End MCTS loop for this batch

        // Return the statistics of the root's children for aggregation
        root_node.get_child_stats()
    }

    /// Runs the MCTS search for the best move from the given game state.
    /// Uses parallel execution if not compiled for WASM.
    fn run_search(&self, game: &Quoridor) -> String {
        let mut rng = thread_rng();
        let root_player = game.active_player;

        // Get initial legal moves
        let legal_pawn = game.get_legal_moves(root_player);
        let legal_walls = game.get_legal_walls(root_player);
        let root_moves: Vec<String> = legal_pawn
            .into_iter()
            .chain(legal_walls.into_iter())
            .collect();

        if root_moves.is_empty() {
            return "resign".to_string();
        }
        if root_moves.len() == 1 {
            return root_moves[0].clone();
        }

        // --- Parallel Execution (Native) ---
        #[cfg(not(target_arch = "wasm32"))]
        let best_move = {
            let num_threads = self.num_threads;
            let total_simulations = self.simulation_limit;
            let sims_per_thread = (total_simulations as f64 / num_threads as f64).ceil() as usize;

            // Calculate time limit per thread if a total time limit is set
            let time_limit_per_thread = self.time_limit.map(|limit| {
                // Divide time, but ensure a minimum duration to avoid instant timeouts
                let duration_per_thread = limit.as_secs_f64() / num_threads as f64;
                Duration::from_secs_f64(duration_per_thread.max(0.01)) // Min 10ms
            });

            // Use Arc<Mutex<...>> for thread-safe aggregation of results
            let aggregated_visits: Arc<Mutex<HashMap<String, usize>>> =
                Arc::new(Mutex::new(HashMap::new()));

            // Use rayon::scope for structured concurrency
            rayon::scope(|s| {
                for _ in 0..num_threads {
                    let game_clone = game.clone(); // Clone game state for each thread
                    let aggregated_visits_clone = Arc::clone(&aggregated_visits);

                    s.spawn(move |_| {
                        // Run an independent MCTS batch in this thread
                        let batch_results = self.run_mcts_simulation_batch(
                            &game_clone,
                            sims_per_thread,
                            time_limit_per_thread, // Pass the calculated per-thread time limit
                        );

                        // Lock the mutex and aggregate results
                        let mut visits_map = aggregated_visits_clone.lock().unwrap();
                        for (mv, visits) in batch_results {
                            *visits_map.entry(mv).or_insert(0) += visits;
                        }
                    });
                }
            }); // Scope automatically waits for all spawned tasks

            // Find the best move from aggregated results
            let final_visits = aggregated_visits.lock().unwrap();
            final_visits
                .iter()
                .max_by_key(|&(_, visits)| visits)
                .map(|(mv, _)| mv.clone())
                .unwrap_or_else(|| {
                    eprintln!("MCTS Warning: No moves found after parallel aggregation.");
                    root_moves
                        .choose(&mut rng)
                        .cloned()
                        .unwrap_or_else(|| "resign".to_string())
                })
        };

        // --- Sequential Execution (WASM or fallback) ---
        #[cfg(target_arch = "wasm32")]
        let best_move = {
            // Run a single batch with the total simulation/iteration limit
            let results = self.run_mcts_simulation_batch(
                game,
                self.simulation_limit,
                self.time_limit_iterations, // Pass iteration limit for WASM
            );

            // Find the best move from the single batch result
            results
                .iter()
                .max_by_key(|&(_, visits)| visits)
                .map(|(mv, _)| mv.clone())
                .unwrap_or_else(|| {
                    eprintln!("MCTS Warning: No moves found after sequential run.");
                    root_moves
                        .choose(&mut rng)
                        .cloned()
                        .unwrap_or_else(|| "resign".to_string())
                })
        };

        best_move
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
         let current_game = game_state; // Modify the passed mutable state (made immutable as it's not reassigned)
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
        // The base name already includes opening info if applicable, constructed in base::new.
        let base_name = self.base.name.clone();

        // Append parallel execution info if relevant and not already part of the base name logic
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Check if time limit is used, as base::new might not include thread count then.
            // A more robust approach might involve parsing the base_name or storing config separately.
            // For now, let's assume if num_threads > 1, we append it.
            if self.num_threads > 1 {
                 // Avoid appending if base_name already seems to include 'xN' (simple check)
                 if !base_name.contains(&format!("x{}", self.num_threads)) {
                    format!("{} x{}", base_name, self.num_threads)
                 } else {
                     base_name
                 }
            } else {
                base_name // No change if single-threaded
            }
        }

        // For WASM, just return the base name constructed initially.
        #[cfg(target_arch = "wasm32")]
        {
            base_name
        }
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
