// --- File: quoridor-project/quoridor-core/tests/strategy_tests.rs ---
//! Integration tests for Quoridor strategies

// Use items re-exported from lib.rs where possible
use quoridor_core::{
    Quoridor, Player, Strategy, Coord, // Use Coord from lib.rs (re-exported from types)
    MCTSStrategy, MinimaxStrategy, RandomStrategy, ShortestPathStrategy,
    MirrorStrategy, DefensiveStrategy, AdaptiveStrategy, BalancedStrategy,
    SimulatedAnnealingStrategy, // Added import
    // GameStatus is not defined/used this way, win condition checked differently
    // BoardPosition, WallPosition, WallDirection are likely internal or just Coord/tuples
};


/// Helper function to check if a move string is in a valid format.
/// Basic check: length 2 (pawn move like "e2") or 3 (wall like "e1h").
/// More robust checks could involve regex or parsing.
fn is_valid_move_format(move_str: &str) -> bool {
    let len = move_str.len();
    if len == 2 {
        // Basic pawn move check (e.g., 'a'-'i', '1'-'9')
        let mut chars = move_str.chars();
        let col = chars.next().unwrap_or(' ');
        let row = chars.next().unwrap_or(' ');
        col >= 'a' && col <= 'i' && row >= '1' && row <= '9'
    } else if len == 3 {
        // Basic wall move check (e.g., 'a'-'h', '1'-'8', 'h'/'v')
        let mut chars = move_str.chars();
        let col = chars.next().unwrap_or(' ');
        let row = chars.next().unwrap_or(' ');
        let orientation = chars.next().unwrap_or(' ');
        col >= 'a' && col <= 'h' && row >= '1' && row <= '8' && (orientation == 'h' || orientation == 'v')
    } else {
        false // Invalid length
    }
}


#[test]
fn test_mcts_returns_valid_move() {
    let game = Quoridor::new(9, 10, None); // Standard 9x9 game, 10 walls, default start
    // Use low simulation count for speed in testing
    let mut mcts_strategy = MCTSStrategy::new("None", vec![], 50);

    let chosen_move = mcts_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "MCTS strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "MCTS move '{}' has invalid format", move_str);
    // We could also check if the move is actually legal in the game state,
    // but that requires more setup or exposing legality checks.
    // For now, just check format and non-None.
}

#[test]
fn test_minimax_returns_valid_move() {
    let game = Quoridor::new(9, 10, None); // Standard 9x9 game, 10 walls, default start
    // Use low depth for speed in testing
    let mut minimax_strategy = MinimaxStrategy::new("None", vec![], 1); // Depth 1

    let chosen_move = minimax_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "Minimax strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "Minimax move '{}' has invalid format", move_str);
}
#[test]
fn test_random_returns_valid_move() {
    let game = Quoridor::new(9, 10, None); // Standard 9x9 game, 10 walls, default start
    let mut random_strategy = RandomStrategy::new("None", vec![]);

    let chosen_move = random_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "Random strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "Random move '{}' has invalid format", move_str);
}

#[test]
fn test_shortest_path_returns_valid_move() {
    let game = Quoridor::new(9, 10, None); // Standard 9x9 game, 10 walls, default start
    let mut sp_strategy = ShortestPathStrategy::new("None", vec![]);

    let chosen_move = sp_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "ShortestPath strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "ShortestPath move '{}' has invalid format", move_str);
}

#[test]
fn test_shortest_path_finds_winning_move() {
    // Setup: Player 1 at e8 (1 step from goal), Player 2 at e1 (far). P1 to move.
    // State string format: "h_walls/v_walls/p1_pos p2_pos/p1_walls p2_walls/active_player"
    let state_string = " / / e8 e1 / 10 10 / 1";
    let mut game = Quoridor::new(9, 10, Some(state_string));

    // Verify setup (optional but good practice)
    assert_eq!(game.active_player, Player::Player1);
    assert_eq!(game.pawn_positions[&Player::Player1], game.algebraic_to_coord("e8")); // (1, 4)
    assert_eq!(game.pawn_positions[&Player::Player2], game.algebraic_to_coord("e1")); // (8, 4)


    let mut sp_strategy = ShortestPathStrategy::new("None", vec![]);
    let chosen_move = sp_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "ShortestPath should find a move");
    let move_str = chosen_move.unwrap();
    assert_eq!(move_str, "e9", "ShortestPath should choose the winning move 'e9'");

    // Check win condition *before* applying the move
    assert!(game.win_check(&move_str), "Move 'e9' should be a winning move for P1");

    // Apply the move using the correct method (move_pawn)
    let result = game.move_pawn(&move_str, true); // Use check=true
    assert!(result, "Applying winning move 'e9' should succeed");

    // Verify final state (optional)
    assert_eq!(game.pawn_positions[&Player::Player1], game.algebraic_to_coord("e9")); // P1 reached goal
    assert_eq!(game.active_player, Player::Player2); // Turn should switch even on win
}


#[test]
fn test_mirror_returns_valid_move() {
    let mut game = Quoridor::new(9, 10, None);
    // Need to make a first move for mirror to react to
    assert!(game.move_pawn("e2", true)); // Player 1 moves using move_pawn
    let mut mirror_strategy = MirrorStrategy::new("None", vec![]);

    let chosen_move = mirror_strategy.choose_move(&game); // Player 2's turn (game state updated by move_pawn)

    assert!(chosen_move.is_some(), "Mirror strategy should return a move");
    let move_str = chosen_move.unwrap();
    // Mirror might place a wall or move pawn, just check format
    assert!(is_valid_move_format(&move_str), "Mirror move '{}' has invalid format", move_str);
    // We could also try applying the mirrored move to see if it's legal
}

#[test]
fn test_mirror_mirrors_pawn_move() {
    let mut game = Quoridor::new(9, 10, None);
    // P1 moves pawn from e1 to e2
    assert!(game.move_pawn("e2", true));
    assert_eq!(game.active_player, Player::Player2);

    let mut mirror_strategy = MirrorStrategy::new("None", vec![]);
    let chosen_move = mirror_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "Mirror strategy should choose a move");
    let move_str = chosen_move.unwrap();

    // P2 should mirror P1's move from e9 to e8
    assert_eq!(move_str, "e8", "Mirror should mirror pawn move e2 -> e8");
}

#[test]
fn test_mirror_mirrors_wall_move() {
    let mut game = Quoridor::new(9, 10, None);
    // P1 places wall at e2h (between row 2 and 3, col e/f)
    assert!(game.add_wall("e2h", false, true));
    assert_eq!(game.active_player, Player::Player2);

    let mut mirror_strategy = MirrorStrategy::new("None", vec![]);
    let chosen_move = mirror_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "Mirror strategy should choose a move");
    let move_str = chosen_move.unwrap();

    // P2 should mirror P1's wall at d7h (between row 7 and 8, col d/e)
    assert_eq!(move_str, "d7h", "Mirror should mirror wall move e2h -> e7h");

    // Optional: Verify the mirrored wall is legal
    let legal_walls = game.get_legal_walls(Player::Player2);
    assert!(legal_walls.contains(&move_str), "Mirrored wall '{}' must be legal", move_str);
}


#[test]
fn test_defensive_returns_valid_move() {
    let game = Quoridor::new(9, 10, None);
    // Provide the required wall_preference argument (e.g., 0.5)
    let mut defensive_strategy = DefensiveStrategy::new("None", vec![], 0.5);

    let chosen_move = defensive_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "Defensive strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "Defensive move '{}' has invalid format", move_str);
}

// Re-enable test for AdaptiveStrategy with correct constructor
#[test]
fn test_adaptive_returns_valid_move() {
    let game = Quoridor::new(9, 10, None);
    // AdaptiveStrategy::new only takes opening_name and opening_moves
    let mut adaptive_strategy = AdaptiveStrategy::new("None", vec![]);

    let chosen_move = adaptive_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "Adaptive strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "Adaptive move '{}' has invalid format", move_str);
}

// Re-enable test for BalancedStrategy with correct constructor
#[test]
fn test_balanced_returns_valid_move() {
    let game = Quoridor::new(9, 10, None);
    // Provide the required defense_weight argument (e.g., 0.5)
    let mut balanced_strategy = BalancedStrategy::new("None", vec![], 0.5);

    let chosen_move = balanced_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "Balanced strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "Balanced move '{}' has invalid format", move_str);
}


#[test]
fn test_simulated_annealing_returns_valid_move() {
    let game = Quoridor::new(9, 10, None);
    // Provide the required time_factor argument (e.g., 1.0)
    let mut sa_strategy = SimulatedAnnealingStrategy::new("None", vec![], 1.0);

    let chosen_move = sa_strategy.choose_move(&game);

    assert!(chosen_move.is_some(), "SimulatedAnnealing strategy should return a move");
    let move_str = chosen_move.unwrap();
    assert!(is_valid_move_format(&move_str), "SimulatedAnnealing move '{}' has invalid format", move_str);
}


// TODO: Add more specific scenario tests (e.g., blocking, forcing moves)
