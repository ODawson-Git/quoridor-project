// --- File: quoridor-project/quoridor-core/src/player.rs ---

//! Defines the Player enum and associated methods.

use std::fmt;

/// Enum identifying the two players in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    /// Returns the opponent of the current player.
    pub fn opponent(&self) -> Self {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }

    /// Returns a simple string name for the player.
    pub fn name(&self) -> &'static str {
        match self {
            Player::Player1 => "player1",
            Player::Player2 => "player2",
        }
    }

    /// Returns a numerical representation (1 or 2).
    pub fn number(&self) -> usize {
        match self {
            Player::Player1 => 1,
            Player::Player2 => 2,
        }
    }
}

// Implement Display for easier printing
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}