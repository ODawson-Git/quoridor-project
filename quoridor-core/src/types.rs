// --- File: quoridor-project/quoridor-core/src/types.rs ---

//! Defines core type aliases used throughout the library.

/// Represents a coordinate on the Quoridor board.
/// (row, column), where (0, 0) is the top-left corner.
pub type Coord = (usize, usize);

// Add other core types here if needed later (e.g., Move enum)
// pub enum Move {
//     Pawn(Coord),
//     WallH(Coord),
//     WallV(Coord),
// }