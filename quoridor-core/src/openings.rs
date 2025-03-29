// --- File: quoridor-project/quoridor-core/src/openings.rs ---

//! Defines opening move sequences for different strategies.

use crate::player::Player;

/// Returns a vector of opening moves (in algebraic notation) for a given opening name and player.
pub fn get_opening_moves(opening_name: &str, player: Player) -> Vec<String> {
     // Use macro_rules! for cleaner definition? Or keep match statement.
     match (opening_name, player) {
        ("No Opening", _) => Vec::new(), // Explicitly handle "No Opening"

        // --- Offensive Openings ---
        ("Standard Opening", Player::Player1) => vec!["e2", "e3", "e4", "e3v"], // Move pawn, place central wall
        ("Standard Opening", Player::Player2) => vec!["e8", "e7", "e6", "e6v"], // Mirror P1

        ("Standard Opening (Symmetrical)", Player::Player1) => vec!["e2", "e3", "e4", "e3v"],
        ("Standard Opening (Symmetrical)", Player::Player2) => vec!["e8", "e7", "e6", "d6v"], // P2 places wall symmetrically

        ("Shiller Opening", Player::Player1) => vec!["e2", "e3", "e4", "c3v"], // Rush pawn, place side wall
        ("Shiller Opening", Player::Player2) => vec!["e8", "e7", "e6"], // Standard response

        ("Rush Variation", Player::Player1) => vec!["e2", "e3", "e4", "d5v", "e4h", "g4h", "h5v"], // Aggressive pawn + walls
        ("Rush Variation", Player::Player2) => vec!["e8", "e7", "e6", "e6h", "f6", "f5", "g5"], // Counter-rush

         ("Gap Opening", Player::Player1) => vec!["e2", "e3", "e4"], // Simple pawn push
         ("Gap Opening", Player::Player2) => vec!["e8", "e7", "e6"], // Mirror

         ("Gap Opening (Mainline)", Player::Player1) => vec!["e2", "e3", "e4"],
         ("Gap Opening (Mainline)", Player::Player2) => vec!["e8", "e7", "e6", "g6h"], // P2 places side wall

         ("Ala Opening", Player::Player1) => vec!["e2", "e3", "e4", "d5h", "f5h", "c4v", "g4v"], // Create central box
         ("Ala Opening", Player::Player2) => vec!["e8", "e7", "e6"], // Standard response


        // --- Defensive Openings ---
        ("Sidewall Opening", Player::Player1) => vec!["c3h", "f3h"], // Place walls near own start
        ("Sidewall Opening", Player::Player2) => vec!["c6h", "f6h"], // Mirror P1 defensively

        ("Stonewall", Player::Player1) => vec!["e2", "e3", "d2h"], // Pawn move + defensive wall
        ("Stonewall", Player::Player2) => vec!["e8", "e7", "e7h"], // Mirror P1

         ("Anti-Gap", Player::Player1) => vec!["e2", "e3", "e4"], // Standard pawn push
         ("Anti-Gap", Player::Player2) => vec!["e8", "e7", "e6", "b3h"], // Wall to potentially disrupt P1 side path


        // --- Other/Unusual Openings ---
        ("Sidewall", Player::Player1) => vec!["e2", "d7v"], // Move then place far wall
        ("Sidewall", Player::Player2) => vec!["e8"],

        ("Sidewall (Proper Counter)", Player::Player1) => vec!["e2", "d7v"],
        ("Sidewall (Proper Counter)", Player::Player2) => vec!["e8", "c7h"], // Counter the far wall

         ("Quick Box Variation", Player::Player1) => vec!["e2"],
         ("Quick Box Variation", Player::Player2) => vec!["e8", "d1h"], // Aggressive early wall near P1

         ("Shatranj Opening", Player::Player1) => vec!["d1v"], // Unusual first move wall
         ("Shatranj Opening", Player::Player2) => Vec::new(),

         ("Lee Inversion", Player::Player1) => vec!["e1v"], // Another unusual first move wall
         ("Lee Inversion", Player::Player2) => Vec::new(),

        // Default: No opening moves for unrecognized names
        _ => Vec::new(),
    }.into_iter().map(String::from).collect() // Convert &str to String
}