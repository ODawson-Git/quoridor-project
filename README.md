# Quoridor AI Arena

A modular implementation of the Quoridor board game with a focus on AI strategy development, analysis, and comparison.

## Project Overview

This repository contains a complete implementation of the Quoridor board game, including:

- Core game logic implemented in Rust
- Multiple AI strategies based on various algorithms
- Standard openings from Quoridor literature
- WebAssembly integration for browser-based gameplay
- Tournament runner for strategy comparison
- Analysis tools for evaluating tournament results

The project focuses on modularity, allowing easy addition of new strategies and openings, as well as extensive testing capabilities.

## Repository Structure

```
quoridor-project/
├── quoridor-core/        # Core game logic and strategies in Rust
├── quoridor-wasm/        # WebAssembly bindings for browser integration
├── quoridor-cli/         # CLI tournament runner
├── web-ui/               # React frontend for browser gameplay
├── analysis/             # Python scripts for tournament analysis
└── tournament_outputs/   # CSV files from tournament runs
```

## Game Rules

Quoridor is played on a 9×9 board. Each player is represented by a pawn which begins at the center space of one edge of the board (in a two-player game, the pawns begin opposite each other). The objective is to be the first player to move their pawn to any space on the opposite side of the game board from which it begins.

The distinguishing characteristic of Quoridor is its twenty walls. Walls are flat two-space-wide pieces which can be placed in the groove that runs between the spaces. Walls block the path of all pawns, which must go around them. The walls are divided equally among the players at the start of the game, and once placed, cannot be moved or removed. On a turn, a player may either move their pawn, or, if possible, place a wall.

### Pawn Movement

Pawns can be moved to any adjacent space (orthogonally, not diagonally). If adjacent to another pawn, the pawn may jump over that pawn if there is no wall blocking. If there's a wall or the edge of the board behind the opponent's pawn, the player may move diagonally adjacent to the opponent.

### Wall Placement

Walls can be placed directly between two spaces, in any groove not already occupied by a wall. However, a wall may not be placed which cuts off the only remaining path of any pawn to the side of the board it must reach.

### Notation

In this implementation, standard algebraic notation is used:

- Board coordinates range from a1 to i9, with a1 being the bottom-left corner
- Player 1 starts at e1 and aims to reach row 9
- Player 2 starts at e9 and aims to reach row 1
- Pawn moves are noted by the target square (e.g., "e2")
- Wall placements are noted by the bottom-left square they touch, followed by orientation (e.g., "e3h" for horizontal, "e3v" for vertical)

## AI Strategies

The following AI strategies are implemented in the core library:

| Strategy | Description |
|----------|-------------|
| Random | Randomly selects from all legal moves (pawn movements and wall placements) with uniform probability. |
| ShortestPath | Always moves the pawn along the shortest path to the goal (calculated via BFS), ignoring wall placement entirely. |
| Defensive | With (default) 70% probability, places a wall maximally increasing the opponent's shortest path distance (biased to place behind self). Otherwise, plays ShortestPath. |
| Balanced | With (default) 50% probability, plays Defensive; otherwise, plays ShortestPath. |
| Adaptive | Plays ShortestPath if closer to the goal than the opponent. Otherwise, plays Defensive. |
| Minimax*depth* | Uses Minimax search with alpha-beta pruning to a specified "depth". Employs Mertens' C3 heuristic (position difference, Max moves to next col, Min moves to next col) for evaluation. |
| Mirror | Attempts to move towards the mirrored position of the opponent's pawn and mirror wall placements. Falls back to Adaptive if mirroring is illegal or impossible. |
| SimulatedAnnealing*temp* | Implements global and local simulated annealing loops based on McDermid et al. (2003), using Mertens' C3 heuristic as the evaluation function. "*temp*" controls initial temperature/randomness. |
| MCTS*sims/time* | Implements Monte Carlo Tree Search with UCT, performing a specified number of simulations ("sims") or running for a time limit ("time") per move, following the structure in Respall (2018). |

## Opening Strategies

The following standard openings are implemented:

| Opening | Description |
|---------|-------------|
| Standard Opening | Forces the sequence: [e2 e8, e3 e7, e4 e6, e3v e6v] using standard Quoridor notation. |
| Standard Opening (Symmetrical) | Similar to Standard Opening but with a symmetrical wall placement for Player 2. |
| Shiller Opening | Rush pawn and place side wall: [e2 e8, e3 e7, e4 e6, c3v]. |
| Stonewall | Pawn move + defensive wall: [e2 e8, e3 e7, d2h e7h]. |
| Ala Opening | Creates a central box: [e2 e8, e3 e7, e4 e6, d5h, f5h, c4v, g4v]. |
| Rush Variation | Aggressive pawn + walls: [e2 e8, e3 e7, e4 e6, d5v e6h, e4h f6, g4h f5, h5v g5]. |
| Gap Opening | Simple pawn push: [e2 e8, e3 e7, e4 e6]. |
| Sidewall Opening | Place walls near own start: [c3h e8, f3h e7]. |
| Anti-Gap | Standard pawn push with wall disruption: [e2 e8, e3 e7, e4 e6, b3h]. |
| Sidewall | Move then place far wall: [e2 e8, d7v]. |
| Lee Inversion | Unusual first move wall: [e1v]. |
| Shatranj Opening | Unusual first move wall: [d1v]. |

## Running the Project

### Web UI

To run the web interface:

1. Ensure you have Node.js and Rust installed
2. Install wasm-pack if not already installed: `cargo install wasm-pack`
3. Build the WebAssembly module:
   ```
   cd quoridor-wasm
   wasm-pack build --target web
   ```
4. Run the web UI:
   ```
   cd web-ui
   npm install
   npm run dev
   ```
5. Access the UI at http://localhost:5173/

### Tournament Runner

To run AI tournaments:

1. Ensure Rust is installed
2. Run the tournament CLI:
   ```
   cd quoridor-project
   cargo run -p quoridor-cli
   ```
3. Results will be saved in the `tournament_outputs/` directory

### Analysis

To analyze tournament results:

1. Ensure Python and required libraries are installed (pandas, matplotlib, seaborn, nashpy)
2. Run the analysis script:
   ```
   cd analysis
   python analyze_tournament.py ../tournament_outputs/[your_tournament_file].csv
   ```
3. Results will be saved in the `analysis_outputs/` directory

## Analysis Features

The analysis toolkit includes:

- Strategy performance metrics (win rates, draw percentages)
- Strategy vs. Opening heatmaps
- Head-to-head matchup analysis
- Replicator dynamics simulations
- Nash equilibrium calculations

## References

- Mertens, P. J. C. (2006). [A Quoridor-playing Agent](https://project.dke.maastrichtuniversity.nl/games/files/bsc/Mertens_BSc-paper.pdf). B.Sc. thesis, Maastricht University.
- Glendenning, L. (2005). Mastering Quoridor. Bachelor Thesis, Department of Computer Science, The University of New Mexico.
- Respall, V. M. (2018). Quoridor Agent using Monte Carlo Tree Search. Graduate thesis, Innopolis University.
- McDermid, Q., Patil, A., & Raguimov, T. (2003). [Applying Genetic Algorithms to Quoridor Game Search Trees for Next-Move Selection](https://web.archive.org/web/20140914030824/http://quoridorai.googlecode.com/svn-history/r2/trunk/papers/AI_Project.pdf). CS486 Final Group Project Report, University of Waterloo.
- [Quoridor Strats: Notation](https://quoridorstrats.wordpress.com/notation/) (2014).
- [Gigamic: Quoridor](https://en.gigamic.com/modern-classics/107-quoridor.html).
- Knight, V. (2017). [NashPy](https://nashpy.readthedocs.io/en/stable/).
- Hofbauer, J., & Sigmund, K. (1998). Evolutionary Games and Population Dynamics. Cambridge University Press.
- Axelrod, R. (1980). Effective Choice in the Prisoner's Dilemma. Journal of Conflict Resolution, 24(1), 3-25.

## License

This project is licensed under MIT OR Apache-2.0.