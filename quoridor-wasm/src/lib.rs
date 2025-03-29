// --- File: quoridor-project/quoridor-wasm/src/lib.rs ---

// Import necessary items from the core library
use quoridor_core::{Quoridor, Player, Strategy, Coord}; // Add more imports as needed
use quoridor_core::strategy::{ self, RandomStrategy, ShortestPathStrategy, MCTSStrategy, MinimaxStrategy, DefensiveStrategy, AdaptiveStrategy, BalancedStrategy, MirrorStrategy, SimulatedAnnealingStrategy}; // Example strategy imports
use quoridor_core::openings; // Import the openings module
use web_sys::js_sys;

// Import wasm-bindgen essentials
use wasm_bindgen::prelude::*;

// Import the utils module we will create next
mod utils;

// This function is called when the WASM module is loaded.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    utils::set_panic_hook(); // Enable better panic messages in console
    console_log!("Quoridor WASM module loaded and panic hook set.");
    Ok(())
}

// Macro for easier console logging from Rust
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (utils::log(&format_args!($($t)*).to_string()))
}

// Define the main struct that will be exposed to JavaScript.
// It wraps the core Quoridor game logic.
#[wasm_bindgen]
pub struct QuoridorGame {
    game_instance: Quoridor,
    // Store strategies as boxed traits. Option allows for 'Human' players.
    player1_strategy: Option<Box<dyn Strategy>>,
    player2_strategy: Option<Box<dyn Strategy>>,
    // Cache legal moves to avoid recalculating constantly if state hasn't changed
    // Note: Caching needs careful implementation to ensure it's invalidated correctly.
    // For simplicity, we might initially omit caching and calculate on demand.
    // cached_legal_moves: Option<Vec<String>>,
    // cached_legal_walls: Option<Vec<String>>,
}

// Methods exposed to JavaScript via wasm-bindgen
#[wasm_bindgen]
impl QuoridorGame {
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize, walls: usize) -> Self {
        console_log!("Creating new QuoridorGame instance ({}x{} board, {} walls)", size, size, walls);
        let game = Quoridor::new(size, walls, None);
        Self {
            game_instance: game,
            player1_strategy: None, // Default to Human
            player2_strategy: None, // Default to Human
        }
    }

    /// Resets the game to its initial state.
    pub fn reset_game(&mut self) {
         console_log!("Resetting game...");
         // Create a new instance to ensure clean state
         self.game_instance = Quoridor::new(
             self.game_instance.size,
             self.game_instance.walls,
             None // Start from default position
         );
         // Keep strategies as they were (or reset them if desired)
         // self.player1_strategy = None;
         // self.player2_strategy = None;
         console_log!("Game reset complete.");
    }

    /// Sets the AI strategy for a given player.
    /// player_number: 1 or 2
    /// strategy_name: Name of the strategy (e.g., "Random", "Minimax2", "MCTS1sec")
    /// opening_name: Name of the opening book to use
    /// Returns true if the strategy was successfully set, false otherwise.
    pub fn set_strategy(&mut self, player_number: usize, strategy_name: &str, opening_name: &str) -> bool {
        console_log!("Setting strategy for Player {} to '{}' with opening '{}'", player_number, strategy_name, opening_name);

        let player = match player_number {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => {
                console_log!("Error: Invalid player number '{}'", player_number);
                return false;
            }
        };

        // Get opening moves from the core library
        let opening_moves = openings::get_opening_moves(opening_name, player);
        if !opening_moves.is_empty() {
             console_log!("Loaded {} opening moves for Player {}", opening_moves.len(), player_number);
        }

        // Create the strategy based on the name
        // This needs to match the strategy implementations in quoridor-core
        let strategy_instance: Option<Box<dyn Strategy>> = match strategy_name {
            "Human" => None, // Represent Human player with None
            "Random" => Some(Box::new(RandomStrategy::new(opening_name, opening_moves))),
            "ShortestPath" => Some(Box::new(ShortestPathStrategy::new(opening_name, opening_moves))),
            "Defensive" => Some(Box::new(DefensiveStrategy::new(opening_name, opening_moves, 0.7))), // Example param
            "Balanced" => Some(Box::new(BalancedStrategy::new(opening_name, opening_moves, 0.5))), // Example param
            "Adaptive" => Some(Box::new(AdaptiveStrategy::new(opening_name, opening_moves))),
            "Mirror" => Some(Box::new(MirrorStrategy::new(opening_name, opening_moves))),
            s if s.starts_with("SimulatedAnnealing") => {
                // Example: "SimulatedAnnealing1.5" -> 1.5
                let factor_str = s.trim_start_matches("SimulatedAnnealing");
                let factor = factor_str.parse::<f64>().unwrap_or(1.0); // Default factor if parsing fails
                console_log!("Creating SimulatedAnnealing strategy with factor {}", factor);
                Some(Box::new(SimulatedAnnealingStrategy::new(opening_name, opening_moves, factor)))
            },
            s if s.starts_with("Minimax") => {
                // Example: "Minimax2" -> depth 2
                let depth_str = s.trim_start_matches("Minimax");
                let depth = depth_str.parse::<usize>().unwrap_or(1); // Default depth 1
                 console_log!("Creating Minimax strategy with depth {}", depth);
                Some(Box::new(MinimaxStrategy::new(opening_name, opening_moves, depth)))
            },
             s if s.starts_with("MCTS") => {
                // Handle time-based ("MCTS1sec") or simulation-based ("MCTS60k")
                if s.ends_with("sec") {
                    let time_str = s.trim_start_matches("MCTS").trim_end_matches("sec");
                    let seconds = time_str.parse::<f64>().unwrap_or(1.0);
                    // Convert time to an approximate simulation count for WASM environment
                    // This factor (e.g., 50000) is highly dependent on execution speed
                    // and needs tuning or a different approach for true time limits in WASM.
                    let simulations = (seconds * 50000.0).max(1000.0) as usize; // Ensure minimum simulations
                     console_log!("Creating MCTS strategy with time limit ~{} simulations ({}s)", simulations, seconds);
                    Some(Box::new(MCTSStrategy::new(opening_name, opening_moves, simulations)))
                    // If using time directly:
                    // let mut mcts = MCTSStrategy::new(opening_name, opening_moves, usize::MAX); // MAX sims, rely on time
                    // mcts = mcts.with_time_limit(seconds); // Note: requires cfg adjustments
                    // Some(Box::new(mcts))

                } else {
                    let sim_str = s.trim_start_matches("MCTS").replace("k", "000");
                    let simulations = sim_str.parse::<usize>().unwrap_or(10000); // Default 10k
                     console_log!("Creating MCTS strategy with simulation limit {}", simulations);
                    Some(Box::new(MCTSStrategy::new(opening_name, opening_moves, simulations)))
                }
            },
            _ => {
                console_log!("Error: Unknown strategy name '{}'", strategy_name);
                return false; // Unknown strategy
            }
        };

        // Store the strategy instance
        match player {
            Player::Player1 => self.player1_strategy = strategy_instance,
            Player::Player2 => self.player2_strategy = strategy_instance,
        }
        true
    }

    /// Gets the AI's chosen move for the current active player.
    /// Returns the move as a string (e.g., "e2", "a3h") or an empty string if no AI move is applicable.
     pub fn get_ai_move(&mut self) -> String {
        let active_player = self.game_instance.active_player;
        console_log!("Requesting AI move for {}", active_player.name());

        let strategy_option = match active_player {
            Player::Player1 => &mut self.player1_strategy,
            Player::Player2 => &mut self.player2_strategy,
        };

        if let Some(strategy) = strategy_option {
            console_log!("Using strategy: {}", strategy.name());
            // Clone the game state to pass to the strategy
            // This might be inefficient for complex strategies; consider passing a reference if possible,
            // but mutable access for strategy state (like opening move counters) complicates this.
            let current_game_state = self.game_instance.clone();
            match strategy.choose_move(&current_game_state) {
                Some(move_str) => {
                    console_log!("AI chose move: {}", move_str);
                    move_str
                }
                None => {
                    console_log!("AI could not find a move.");
                    "".to_string() // Return empty string if no move found
                }
            }
        } else {
            console_log!("No AI strategy set for active player (Human player).");
            "".to_string() // No AI strategy set (Human player)
        }
    }


    /// Attempts to make a move (pawn or wall) based on algebraic notation.
    /// move_str: The move in algebraic notation (e.g., "e2", "a3h", "b4v").
    /// Returns true if the move was successful, false otherwise.
    pub fn make_move(&mut self, move_str: &str) -> bool {
        console_log!("Attempting to make move: {}", move_str);
        let result = if move_str.len() >= 3 && (move_str.ends_with('h') || move_str.ends_with('v')) {
            // It's a wall move
            self.game_instance.add_wall(move_str, false, true) // check=true
        } else {
            // It's a pawn move
            self.game_instance.move_pawn(move_str, true) // check=true
        };

        if result {
            console_log!("Move successful: {}", move_str);
             // Invalidate caches if implemented
             // self.cached_legal_moves = None;
             // self.cached_legal_walls = None;
        } else {
            console_log!("Move failed: {}", move_str);
        }
        result
    }

    /// Gets the list of legal pawn moves for the active player.
    /// Returns a JS array of strings.
    #[wasm_bindgen(js_name = getLegalMoves)]
    pub fn get_legal_moves(&self) -> JsValue {
        let moves = self.game_instance.get_legal_moves(self.game_instance.active_player);
        // Convert Vec<String> to JsValue (JS Array)
        JsValue::from(moves.into_iter().map(JsValue::from).collect::<js_sys::Array>())
    }

    /// Gets the list of legal wall placements for the active player.
    /// Returns a JS array of strings (e.g., ["a3h", "b4v", ...]).
    #[wasm_bindgen(js_name = getLegalWalls)]
     pub fn get_legal_walls(&self) -> JsValue {
         let player = self.game_instance.active_player;
         // Only return walls if the player has any left
         let walls = if self.game_instance.walls_available[&player] > 0 {
             self.game_instance.get_legal_walls(player)
         } else {
             Vec::new()
         };
         JsValue::from(walls.into_iter().map(JsValue::from).collect::<js_sys::Array>())
     }


    /// Gets the current game state as a JSON string.
    /// Suitable for sending to the frontend to render the board.
    #[wasm_bindgen(js_name = getGameState)]
    pub fn get_game_state(&self) -> String {
        // Use serde_json if more complex state is needed. For now, manual string building.
        let p1 = self.game_instance.pawn_positions[&Player::Player1];
        let p2 = self.game_instance.pawn_positions[&Player::Player2];

        // Convert wall coordinates to algebraic notation strings
        let h_walls_alg: Vec<String> = self.game_instance.hwall_positions.iter()
            .map(|&pos| self.game_instance.coord_to_algebraic(pos))
            .collect();
        let v_walls_alg: Vec<String> = self.game_instance.vwall_positions.iter()
            .map(|&pos| self.game_instance.coord_to_algebraic(pos))
            .collect();

        // Use format! macro with proper JSON syntax, escaping strings
        format!(
            r#"{{"size": {}, "player1": {{"row": {}, "col": {}}}, "player2": {{"row": {}, "col": {}}}, "player1Walls": {}, "player2Walls": {}, "hWalls": {:?}, "vWalls": {:?}, "activePlayer": {}, "lastMove": {:?}, "currentStateString": {:?}}}"#,
            self.game_instance.size,
            p1.0, p1.1,
            p2.0, p2.1,
            self.game_instance.walls_available[&Player::Player1],
            self.game_instance.walls_available[&Player::Player2],
            h_walls_alg, // Already Vec<String>, no extra quotes needed by {:?}
            v_walls_alg, // Already Vec<String>
            if self.game_instance.active_player == Player::Player1 { 1 } else { 2 },
            self.game_instance.last_move,
            self.game_instance.state_string
        )
    }


    /// Checks if the given pawn move would result in a win for the currently active player.
    /// move_str: The pawn move in algebraic notation (e.g., "e1").
    /// Returns true if the move is a winning move.
    #[wasm_bindgen(js_name = checkWin)]
    pub fn check_win(&self, move_str: &str) -> bool {
        // Note: This uses the *current* active player stored in the game state.
        // The core win_check likely needs the *next* position,
        // so we might need to adjust how win_check works or parse the move here.
        // For now, assuming win_check correctly interprets the move string
        // relative to the active player's goal.
        self.game_instance.win_check(move_str)
    }

    /// Returns the currently active player (1 or 2).
     #[wasm_bindgen(js_name = getActivePlayer)]
     pub fn get_active_player(&self) -> usize {
         match self.game_instance.active_player {
             Player::Player1 => 1,
             Player::Player2 => 2,
         }
     }
}