// --- File: quoridor-project/quoridor-core/src/graph.rs ---

//! Handles the graph representation of the board and pathfinding logic.

use crate::types::Coord;
use crate::player::Player;
use crate::Quoridor; // Access Quoridor struct methods
use std::collections::HashMap;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::algo::{dijkstra, has_path_connecting};

/// Initializes the graph with nodes and default edges.
pub(crate) fn initialize_board_graph(
    size: usize,
) -> (UnGraph<Coord, ()>, HashMap<Coord, NodeIndex>) {
    let mut graph = UnGraph::new_undirected();
    let mut node_indices = HashMap::new();

    // Add nodes
    for r in 0..size {
        for c in 0..size {
            let coord = (r, c);
            let index = graph.add_node(coord);
            node_indices.insert(coord, index);
        }
    }

    // Add edges
    for r in 0..size {
        for c in 0..size {
            let current_coord = (r, c);
            let current_index = node_indices[&current_coord];

            // Connect to right neighbor
            if c + 1 < size {
                let right_coord = (r, c + 1);
                let right_index = node_indices[&right_coord];
                graph.add_edge(current_index, right_index, ());
            }
            // Connect to bottom neighbor
            if r + 1 < size {
                let bottom_coord = (r + 1, c);
                let bottom_index = node_indices[&bottom_coord];
                graph.add_edge(current_index, bottom_index, ());
            }
        }
    }
    (graph, node_indices)
}

/// Returns the coordinates of the two edges a potential wall would block.
/// `wall_coord` is the bottom-left-most coord the wall touches.
pub(crate) fn get_blocked_edges_by_wall(
    wall_coord: Coord,
    orientation: char, // 'h' or 'v'
    size: usize,
) -> Option<[(Coord, Coord); 2]> {
    let (r, c) = wall_coord;

    match orientation {
        'h' => {
            // Horizontal wall blocks vertical movement between (r-1, c) <=> (r, c)
            // and (r-1, c+1) <=> (r, c+1)
            if r > 0 && c + 1 < size {
                Some([((r - 1, c), (r, c)), ((r - 1, c + 1), (r, c + 1))])
            } else {
                None // Wall placement invalid near edge
            }
        }
        'v' => {
            // Vertical wall blocks horizontal movement between (r, c) <=> (r, c+1)
            // and (r-1, c) <=> (r-1, c+1)
            if r > 0 && c + 1 < size {
                 Some([((r, c), (r, c + 1)), ((r - 1, c), (r - 1, c + 1))])
            } else if r == 0 && c + 1 < size { // Special case for top edge
                 Some([((r, c), (r, c + 1)), ((usize::MAX, usize::MAX), (usize::MAX, usize::MAX))]) // Indicate only one edge exists
            }
            else {
                None // Wall placement invalid near edge
            }
        }
        _ => None, // Invalid orientation
    }
}


/// Checks if placing a wall is valid according to game rules (path blocking).
/// This checks the state *after* the wall is hypothetically placed.
pub(crate) fn check_wall_path_blocking(
    graph: &UnGraph<Coord, ()>,
    node_indices: &HashMap<Coord, NodeIndex>,
    pawn_positions: &HashMap<Player, Coord>,
    goal_positions: &HashMap<Player, Vec<Coord>>,
) -> bool {
    for (player, goals) in goal_positions {
        if let Some(start_coord) = pawn_positions.get(player) {
            if let Some(start_node) = node_indices.get(start_coord) {
                let mut has_path_to_a_goal = false;
                for goal_coord in goals {
                    if let Some(goal_node) = node_indices.get(goal_coord) {
                        // Use petgraph's path check
                        if has_path_connecting(graph, *start_node, *goal_node, None) {
                            has_path_to_a_goal = true;
                            break; // Found a path for this player, check next player
                        }
                    }
                }
                // If no path found to any goal for this player, the wall placement is illegal
                if !has_path_to_a_goal {
                    return false; // Placement blocks this player
                }
            } else {
                 eprintln!("Warning: Pawn position {:?} not found in node indices during wall check.", start_coord);
                 return false; // Treat as invalid if pawn isn't on graph
            }
        } else {
            eprintln!("Warning: Player {:?} not found in pawn positions during wall check.", player);
            return false; // Treat as invalid if player doesn't exist
        }
    }
    true // All players still have a path
}

/// Calculates the shortest path distance for a player to their goal line.
/// Returns usize::MAX if no path exists.
pub(crate) fn get_shortest_path_len(
    graph: &UnGraph<Coord, ()>,
    node_indices: &HashMap<Coord, NodeIndex>,
    start_coord: Coord,
    goal_coords: &[Coord],
) -> usize {
    if let Some(start_node) = node_indices.get(&start_coord) {
        // Calculate distances from start_node to all reachable nodes
        let distances = dijkstra(graph, *start_node, None, |_| 1); // Edge cost is 1

        let mut min_dist = usize::MAX;
        for goal in goal_coords {
            if let Some(goal_node) = node_indices.get(goal) {
                if let Some(dist) = distances.get(goal_node) {
                    min_dist = min_dist.min(*dist);
                }
            }
        }
         min_dist // Return usize::MAX if no goal was reachable
    } else {
         eprintln!("Warning: Start coordinate {:?} not found in graph for path calculation.", start_coord);
        usize::MAX // Start node doesn't exist
    }
}