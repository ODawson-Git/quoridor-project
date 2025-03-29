// --- File: quoridor-project/quoridor-core/src/utils.rs ---

//! Utility functions for coordinate conversions and calculations.

use crate::types::Coord; // Use the type alias from this crate

/// Converts algebraic notation (e.g., "e1", "a9") to board coordinates (row, col).
/// Panics on invalid input.
pub fn algebraic_to_coord(square: &str, board_size: usize) -> Coord {
    // Handle potential wall notation passed erroneously
     let pos_str = if square.len() > 2 && (square.ends_with('h') || square.ends_with('v')) {
         &square[0..2]
     } else {
         square
     };

    if pos_str.len() < 2 {
        panic!("Invalid algebraic notation length: '{}'", square);
    }

    let bytes = pos_str.as_bytes();
    let col_char = bytes[0] as char;

    if !col_char.is_ascii_alphabetic() {
        panic!("Invalid column character in algebraic notation: '{}'", square);
    }

    let col = (col_char.to_ascii_lowercase() as u8) - b'a';

    let row_str = &pos_str[1..];
    let row_num: usize = match row_str.parse() {
        Ok(num) if num >= 1 && num <= board_size => num,
        _ => panic!("Invalid row number in algebraic notation: '{}'", square),
    };

    // Convert algebraic row (1-based from bottom) to 0-based index from top
    let row = board_size - row_num;

    if row >= board_size || (col as usize) >= board_size {
        panic!("Algebraic notation out of bounds: '{}'", square);
    }

    (row, col as usize)
}

/// Converts board coordinates (row, col) to algebraic notation (e.g., "e1", "a9").
pub fn coord_to_algebraic(coord: Coord, board_size: usize) -> String {
    let (row, col) = coord;
    if row >= board_size || col >= board_size {
         panic!("Coordinate out of bounds: {:?}", coord);
    }
    let col_char = (b'a' + col as u8) as char;
    // Convert 0-based row index to 1-based algebraic row number
    let row_num = board_size - row;
    format!("{}{}", col_char, row_num)
}

/// Calculates the absolute difference between two usize values.
pub fn abs_diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    const TEST_SIZE: usize = 9;

    #[test]
    fn test_coord_conversion() {
        assert_eq!(algebraic_to_coord("a1", TEST_SIZE), (8, 0));
        assert_eq!(algebraic_to_coord("i9", TEST_SIZE), (0, 8));
        assert_eq!(algebraic_to_coord("e5", TEST_SIZE), (4, 4));
        assert_eq!(algebraic_to_coord("e1", TEST_SIZE), (8, 4)); // Player 1 start (usually)
        assert_eq!(algebraic_to_coord("e9", TEST_SIZE), (0, 4)); // Player 2 start (usually)

        assert_eq!(coord_to_algebraic((8, 0), TEST_SIZE), "a1");
        assert_eq!(coord_to_algebraic((0, 8), TEST_SIZE), "i9");
        assert_eq!(coord_to_algebraic((4, 4), TEST_SIZE), "e5");
         assert_eq!(coord_to_algebraic((8, 4), TEST_SIZE), "e1");
         assert_eq!(coord_to_algebraic((0, 4), TEST_SIZE), "e9");
    }

     #[test]
     fn test_alg_to_coord_with_wall_notation() {
         assert_eq!(algebraic_to_coord("a1h", TEST_SIZE), (8, 0));
         assert_eq!(algebraic_to_coord("e5v", TEST_SIZE), (4, 4));
     }

    #[test]
    #[should_panic]
    fn test_invalid_alg_col() {
        algebraic_to_coord("z5", TEST_SIZE);
    }

     #[test]
     #[should_panic]
     fn test_invalid_alg_row_num() {
          algebraic_to_coord("a10", TEST_SIZE);
     }

     #[test]
     #[should_panic]
     fn test_invalid_alg_row_char() {
          algebraic_to_coord("aX", TEST_SIZE);
     }

     #[test]
     #[should_panic]
     fn test_invalid_alg_short() {
          algebraic_to_coord("a", TEST_SIZE);
     }

    #[test]
    fn test_abs_diff() {
        assert_eq!(abs_diff(5, 2), 3);
        assert_eq!(abs_diff(2, 5), 3);
        assert_eq!(abs_diff(5, 5), 0);
    }
}