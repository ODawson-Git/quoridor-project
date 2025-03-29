// --- File: quoridor-project/quoridor-cli/src/main.rs ---

use quoridor_core::{Quoridor, Player, Strategy}; // Import from core crate
use quoridor_core::strategy::{ self, RandomStrategy, ShortestPathStrategy, MCTSStrategy, MinimaxStrategy, DefensiveStrategy, AdaptiveStrategy, BalancedStrategy, MirrorStrategy, SimulatedAnnealingStrategy }; // Import specific strategies
use quoridor_core::openings; // Import the openings module
use chrono; // Timestamped files

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::env;

use csv::Writer;
use rand::prelude::*;

// --- Tournament Structures ---

#[derive(Debug, Clone)]
pub struct TournamentResult {
    strategy1: String,
    strategy2: String,
    opening: String,
    strategy1_wins: usize,
    strategy2_wins: usize,
    draws: usize,
    games_played: usize, // Track total games for accurate win %
}

pub struct Tournament {
    board_size: usize,
    walls: usize,
    games_per_match: usize,
    results: Vec<TournamentResult>,
    // Add time limits or simulation counts if strategies need them
    mcts_simulations: usize,
    mcts_time_limit_secs: Option<f64>,
}

impl Tournament {
    pub fn new(board_size: usize, walls: usize, games_per_match: usize) -> Self {
        Tournament {
            board_size,
            walls,
            games_per_match,
            results: Vec::new(),
            mcts_simulations: 10000, // Default simulations
            mcts_time_limit_secs: None, // Default no time limit
        }
    }

    // Optional: Methods to configure MCTS parameters
    pub fn set_mcts_simulations(mut self, simulations: usize) -> Self {
        self.mcts_simulations = simulations;
        self
    }
    pub fn set_mcts_time_limit(mut self, seconds: f64) -> Self {
        self.mcts_time_limit_secs = Some(seconds);
        self
    }


    /// Creates a strategy instance based on name and player.
    /// This centralizes strategy creation.
    pub fn create_strategy(&self, strategy_name: &str, opening_name: &str, player: Player) -> Box<dyn Strategy> {
        let opening_moves = openings::get_opening_moves(opening_name, player);

        match strategy_name {
            "Random" => Box::new(RandomStrategy::new(opening_name, opening_moves)),
            "ShortestPath" => Box::new(ShortestPathStrategy::new(opening_name, opening_moves)),
            "Defensive" => Box::new(DefensiveStrategy::new(opening_name, opening_moves, 0.7)),
            "Balanced" => Box::new(BalancedStrategy::new(opening_name, opening_moves, 0.5)),
            "Adaptive" => Box::new(AdaptiveStrategy::new(opening_name, opening_moves)),
            "Mirror" => Box::new(MirrorStrategy::new(opening_name, opening_moves)),
            s if s.starts_with("SimulatedAnnealing") => {
                let factor_str = s.trim_start_matches("SimulatedAnnealing");
                let factor = factor_str.parse::<f64>().unwrap_or(1.0);
                Box::new(SimulatedAnnealingStrategy::new(opening_name, opening_moves, factor))
            },
            s if s.starts_with("Minimax") => {
                let depth_str = s.trim_start_matches("Minimax");
                let depth = depth_str.parse::<usize>().unwrap_or(1);
                Box::new(MinimaxStrategy::new(opening_name, opening_moves, depth))
            },
            s if s.starts_with("MCTS") => {
                // Handle time-based ("MCTS1sec") or simulation-based ("MCTS60k")
                let simulations: usize;
                let mut time_limit_secs: Option<f64> = None;

                 if s.ends_with("sec") {
                    let time_str = s.trim_start_matches("MCTS").trim_end_matches("sec");
                    let seconds = time_str.parse::<f64>().unwrap_or(1.0);
                    time_limit_secs = Some(seconds);
                    // Use the tournament's default simulation count when time limit is primary
                    simulations = self.mcts_simulations;
                    println!("Creating MCTS strategy with time limit {}s (sim limit: {})", seconds, simulations);
                 } else {
                    let sim_str = s.trim_start_matches("MCTS").replace("k", "000");
                    simulations = sim_str.parse::<usize>().unwrap_or(self.mcts_simulations);
                    // Use tournament's time limit if set globally
                    time_limit_secs = self.mcts_time_limit_secs;
                    println!("Creating MCTS strategy with simulation limit {} (time limit: {:?})", simulations, time_limit_secs);
                 }

                 // Create the MCTS strategy instance
                 let mut mcts_strategy = MCTSStrategy::new(opening_name, opening_moves, simulations);

                 // Apply time limit if specified
                 if let Some(seconds) = time_limit_secs {
                    // This requires MCTSStrategy to have a method like `with_time_limit`
                    // For now, we'll assume the simulation count is the primary driver in CLI
                    // or we modify MCTSStrategy later.
                     println!("Note: Time limit {}s requested, but CLI primarily uses simulation count for MCTS.", seconds);
                    // If MCTSStrategy has `with_time_limit`:
                    // mcts_strategy = mcts_strategy.with_time_limit(seconds);
                 }
                 Box::new(mcts_strategy)
            },
            _ => {
                 eprintln!("Warning: Unknown strategy name '{}', defaulting to Random.", strategy_name);
                 Box::new(RandomStrategy::new(opening_name, opening_moves)) // Default
            }
        }
    }


    /// Runs a single match (multiple games) between two strategies with a specific opening.
    pub fn run_match(
        &self, // Changed to immutable borrow as it only reads config
        strategy1_name: &str,
        strategy2_name: &str,
        opening_name: &str,
        display: bool,
    ) -> TournamentResult {
        let mut s1_wins = 0;
        let mut s2_wins = 0;
        let mut draws = 0;

        if display {
            println!("-> Running Match: {} vs {} (Opening: {})", strategy1_name, strategy2_name, opening_name);
        }

        for game_num in 0..self.games_per_match {
             // Alternate who goes first to reduce bias
             let (first_strategy_type, second_strategy_type, first_player_enum, second_player_enum) =
                 if game_num % 2 == 0 {
                     (strategy1_name, strategy2_name, Player::Player1, Player::Player2)
                 } else {
                     (strategy2_name, strategy1_name, Player::Player1, Player::Player2)
                 };

             if display && self.games_per_match > 1 {
                 println!("  - Game {}: {} (P1) vs {} (P2)", game_num + 1, first_strategy_type, second_strategy_type);
             }

             // Create fresh strategies for each game to reset internal state (like opening counters)
             let mut first_strategy = self.create_strategy(first_strategy_type, opening_name, first_player_enum);
             let mut second_strategy = self.create_strategy(second_strategy_type, opening_name, second_player_enum);

             let mut game = Quoridor::new(self.board_size, self.walls, None);
             let mut move_count = 0;
             let max_moves = 200; // Safeguard against infinite loops

             loop {
                 let current_player = game.active_player;
                 let current_strategy = if current_player == first_player_enum {
                     &mut first_strategy
                 } else {
                     &mut second_strategy
                 };

                 let move_result = current_strategy.choose_move(&game);

                 if move_result.is_none() {
                     if display { println!("    Game {}: {} ({}) cannot move, forfeits.", game_num + 1, current_strategy.name(), current_player.name()); }
                     // The *other* player wins
                     let winner_type = if current_player == first_player_enum { second_strategy_type } else { first_strategy_type };
                     if winner_type == strategy1_name { s1_wins += 1; } else { s2_wins += 1; }
                     break;
                 }

                 let move_str = move_result.unwrap();
                 if display && move_count < 10 { // Display only first few moves
                    println!("    Game {}: Turn {} ({}) plays {}", game_num + 1, move_count + 1, current_player.name(), move_str);
                 }

                 // Check for win *before* making the move on the board state
                 let is_win = game.win_check(&move_str);

                 // Apply the move
                 let move_success = if move_str.len() >= 3 && (move_str.ends_with('h') || move_str.ends_with('v')) {
                     game.add_wall(&move_str, false, true) // Perform checks
                 } else {
                     game.move_pawn(&move_str, true) // Perform checks
                 };

                 if !move_success {
                     eprintln!("!!!! CRITICAL ERROR: Strategy {} chose illegal move {} !!!!", current_strategy.name(), move_str);
                     // Award win to the other player
                     let winner_type = if current_player == first_player_enum { second_strategy_type } else { first_strategy_type };
                     if winner_type == strategy1_name { s1_wins += 1; } else { s2_wins += 1; }
                     break; // Stop the game on illegal move
                 }

                 if is_win {
                    if display { println!("    Game {}: {} ({}) wins with move {}.", game_num + 1, current_strategy.name(), current_player.name(), move_str); }
                    let winning_strategy_name = if current_player == first_player_enum {
                        first_strategy_type // The name assigned to the first player role in this game
                    } else {
                        second_strategy_type // The name assigned to the second player role in this game
                    };

                    // Compare the winning strategy's NAME to the original strategy1_name parameter
                    if winning_strategy_name == strategy1_name { // <-- CORRECT COMPARISON
                        s1_wins += 1;
                    } else {
                        s2_wins += 1;
                    }
                    break; // Exit game loop
                 }

                 move_count += 1;
                 if move_count >= max_moves {
                     if display { println!("    Game {}: Draw due to move limit ({} moves).", game_num + 1, max_moves); }
                     draws += 1;
                     break;
                 }
             } // End game loop
        } // End loop over games_per_match

        TournamentResult {
            strategy1: strategy1_name.to_string(),
            strategy2: strategy2_name.to_string(),
            opening: opening_name.to_string(),
            strategy1_wins: s1_wins,
            strategy2_wins: s2_wins,
            draws,
            games_played: self.games_per_match,
        }
    }


    /// Runs the full tournament, distributing matches across threads.
    pub fn run_tournament_parallel(&mut self, display: bool) {
        let start_time = Instant::now();
        println!(
            "Starting parallel tournament ({}x{} board, {} walls, {} games/match)...",
            self.board_size, self.board_size, self.walls, self.games_per_match
        );

        // --- Configuration ---
        let strategy_names = vec![
            // Basic
            "Random",
            "ShortestPath",
            // Intermediate
            "Defensive",
            "Balanced",
            "Adaptive",
            // Advanced / From Papers
            "Mirror",
            "SimulatedAnnealing0.5", // From paper's experiments
            "SimulatedAnnealing1.0",
            "SimulatedAnnealing2.0",
            "Minimax1", // Low depth for speed
            "Minimax2", // Reference depth from paper
            "Minimax3", // Higher depth
             // MCTS (adjust simulation counts/time as needed)
             "MCTS10k",
             "MCTS60k", // From paper's experiments
             "MCTS1sec", // Time-based example
        ];

        let opening_names = vec![
            "No Opening",
            "Sidewall Opening",
            "Standard Opening",
            "Shiller Opening", // Example other openings
            "Ala Opening",
        ];
        // --- End Configuration ---


        let mut match_configs = Vec::new();
        for opening_name in &opening_names {
            for i in 0..strategy_names.len() {
                for j in (i + 1)..strategy_names.len() { // Avoid self-play and duplicate pairs
                    match_configs.push((
                        strategy_names[i].to_string(),
                        strategy_names[j].to_string(),
                        opening_name.to_string(),
                        display,
                    ));
                }
            }
        }

        let total_matches = match_configs.len();
        println!("Total matches to run: {}", total_matches);

        // Determine number of threads, use available parallelism or fallback
        let num_threads = thread::available_parallelism().map_or(4, |n| n.get());
        println!("Using {} threads.", num_threads);

        let results = Arc::new(Mutex::new(Vec::with_capacity(total_matches)));
        let mut handles = Vec::new();
        let configs_per_thread = (total_matches + num_threads - 1) / num_threads;

        // Create a read-only Arc of the Tournament config to share with threads
        let tournament_config = Arc::new(Tournament::new(self.board_size, self.walls, self.games_per_match)
            .set_mcts_simulations(self.mcts_simulations) // Pass config through
            .set_mcts_time_limit(self.mcts_time_limit_secs.unwrap_or(0.0))); // Pass config through


        for (thread_id, chunk) in match_configs.chunks(configs_per_thread).enumerate() {
            let thread_chunk = chunk.to_vec(); // Clone chunk for the thread
            let results_clone = Arc::clone(&results);
            let config_clone = Arc::clone(&tournament_config); // Clone Arc, not Tournament

            let handle = thread::spawn(move || {
                let thread_start = Instant::now();
                let mut thread_results = Vec::with_capacity(thread_chunk.len());
                if display { println!("[Thread {}] Starting {} matches...", thread_id, thread_chunk.len()); }

                for (s1, s2, opening, disp) in thread_chunk {
                    // Use the cloned config to run the match
                    let result = config_clone.run_match(&s1, &s2, &opening, disp);
                    thread_results.push(result);
                }

                 // Lock mutex once to add all results for this thread
                 let mut shared_results_guard = results_clone.lock().unwrap();
                 shared_results_guard.extend(thread_results);

                if display { println!("[Thread {}] Finished in {:?}", thread_id, thread_start.elapsed()); }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for (i, handle) in handles.into_iter().enumerate() {
             if display { println!("Waiting for thread {}...", i); }
             handle.join().expect("Thread panicked");
        }

        // Collect results
        // Use Arc::try_unwrap only if you are sure this is the last Arc reference
        // Otherwise, just lock and clone the data.
        let final_results = results.lock().unwrap().clone();
        self.results = final_results; // Store results back into the main tournament instance

        println!(
            "Tournament finished {} matches in {:.2?}.",
            self.results.len(),
            start_time.elapsed()
        );
    }


    /// Writes the collected tournament results to a CSV file.
    pub fn write_results_to_csv(&self, filename: &str) -> std::io::Result<()> {
        println!("Writing results to {}...", filename);
        let path = Path::new(filename);
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut writer = Writer::from_path(path)?;

        // Write CSV Header
        writer.write_record(&[
            "Opening",
            "Strategy",
            "Opponent",
            "Wins",    // Wins for 'Strategy' against 'Opponent'
            "Losses",  // Losses for 'Strategy' against 'Opponent' (Opponent Wins)
            "Draws",
            "Win %",   // Win percentage for 'Strategy'
            "Games Played",
        ])?;

        // Write data rows for each match result
        for result in &self.results {
             let total_games_non_draw = result.games_played - result.draws;

            // Calculate win percentages, handle division by zero
             let win_percentage1 = if total_games_non_draw > 0 {
                 (result.strategy1_wins as f64 / total_games_non_draw as f64) * 100.0
             } else { 0.0 };
             let win_percentage2 = if total_games_non_draw > 0 {
                 (result.strategy2_wins as f64 / total_games_non_draw as f64) * 100.0
             } else { 0.0 };


            // Row for Strategy1 vs Strategy2
            writer.write_record(&[
                &result.opening,
                &result.strategy1,
                &result.strategy2,
                &result.strategy1_wins.to_string(),
                &result.strategy2_wins.to_string(), // Strategy 1's losses = Strategy 2's wins
                &result.draws.to_string(),
                &format!("{:.2}", win_percentage1),
                &result.games_played.to_string(),
            ])?;

            // Row for Strategy2 vs Strategy1
             writer.write_record(&[
                &result.opening,
                &result.strategy2,
                &result.strategy1,
                &result.strategy2_wins.to_string(),
                &result.strategy1_wins.to_string(), // Strategy 2's losses = Strategy 1's wins
                &result.draws.to_string(),
                &format!("{:.2}", win_percentage2),
                &result.games_played.to_string(),
            ])?;
        }

        writer.flush()?; // Ensure all data is written to the file
        println!("Results successfully written to {}.", filename);
        Ok(())
    }
}


// --- Main Application Logic ---

fn main() {
    // Check for debug environment variable
    let debug_enabled = env::var("QUORIDOR_DEBUG").map_or(false, |val| val == "1" || val.to_lowercase() == "true");

    println!("--- Quoridor CLI Tournament Runner ---");
    if debug_enabled {
        println!("Debug mode: Enabled (more verbose output)");
    }

    // Configure tournament parameters
    let mut tournament = Tournament::new(
        9,   // board size (standard)
        10,  // walls per player (standard)
        50, // Number of games per matchup (e.g., 50 games, 25 starting each side)
    );

    // Optional: Configure MCTS parameters if needed globally
    // tournament = tournament.set_mcts_simulations(50000);
    // tournament = tournament.set_mcts_time_limit(1.0); // 1 second per move

    // Run the tournament using multiple threads
    tournament.run_tournament_parallel(debug_enabled);

    // Define the output directory and filename
    let output_dir = "tournament_outputs";
    let output_filename = format!("{}/rust_tournament_results_{}.csv", output_dir, chrono::Local::now().format("%Y%m%d_%H%M%S"));

    // Write results to CSV
    match tournament.write_results_to_csv(&output_filename) {
        Ok(_) => println!("Tournament results saved to '{}'", output_filename),
        Err(e) => eprintln!("Error writing results to CSV: {}", e),
    }

     println!("--- Tournament Finished ---");
}