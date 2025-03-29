// --- File: web-ui/src/components/Controls.jsx ---
import React from 'react';

// Define STRATEGIES and OPENINGS constants if not passed as props or imported
const STRATEGIES = [
    'Human', 'Random', 'ShortestPath', 'Defensive', 'Balanced', 'Adaptive',
    'Minimax1', 'Minimax2', 'Mirror', "MCTS10k", "MCTS60k", "MCTS1sec",
    "SimulatedAnnealing0.5", "SimulatedAnnealing1.0", "SimulatedAnnealing2.0" // Added SA
];
const OPENINGS = [
    'No Opening', 'Sidewall Opening', 'Standard Opening', 'Shiller Opening',
    'Stonewall', 'Ala Opening', 'Standard Opening (Symmetrical)', 'Rush Variation',
    'Gap Opening', 'Gap Opening (Mainline)', 'Anti-Gap', 'Sidewall',
    'Sidewall (Proper Counter)', 'Quick Box Variation', 'Shatranj Opening', 'Lee Inversion'
];


const Controls = ({
  player1Strategy, setPlayer1Strategy,
  player2Strategy, setPlayer2Strategy,
  selectedOpening, setSelectedOpening,
  onStartGame, onResetGame,
  isGameActive, isLoadingWasm, isThinking, // Use isLoadingWasm
  aiMoveSpeed, setAiMoveSpeed, isAiVsAiMode,
  player1Walls, player2Walls // Pass wall counts
}) => {
    // Determine if controls should be disabled
    const controlsDisabled = isGameActive || isLoadingWasm;

  return (
    <div className="w-full md:w-64 p-4 bg-gray-50 rounded-bl-lg border-r border-gray-200">
      <div className="space-y-4">
        {/* Player 1 */}
        <div className="bg-blue-50 p-3 rounded-lg border border-blue-200">
          <label className="text-sm font-medium text-blue-800 block mb-1">Player 1 (Blue)</label>
          <select
            className="w-full border border-blue-300 rounded px-2 py-1 text-blue-800 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            value={player1Strategy}
            onChange={(e) => setPlayer1Strategy(e.target.value)}
            disabled={controlsDisabled}
          >
            <option value="Human">Human</option>
            {STRATEGIES.filter(s => s !== 'Human').map(strategy => (
              <option key={`p1-${strategy}`} value={strategy}>{strategy}</option>
            ))}
          </select>
           <div className="flex items-center mt-2">
                <div className="h-3 w-3 rounded-full bg-blue-600 mr-2"></div>
                <span className="text-xs text-blue-800">Walls: {player1Walls}</span>
            </div>
        </div>

        {/* Player 2 */}
        <div className="bg-red-50 p-3 rounded-lg border border-red-200">
          <label className="text-sm font-medium text-red-800 block mb-1">Player 2 (Red)</label>
          <select
            className="w-full border border-red-300 rounded px-2 py-1 text-red-800 bg-white focus:outline-none focus:ring-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
            value={player2Strategy}
            onChange={(e) => setPlayer2Strategy(e.target.value)}
            disabled={controlsDisabled}
          >
            <option value="Human">Human</option>
            {STRATEGIES.filter(s => s !== 'Human').map(strategy => (
              <option key={`p2-${strategy}`} value={strategy}>{strategy}</option>
            ))}
          </select>
           <div className="flex items-center mt-2">
                <div className="h-3 w-3 rounded-full bg-red-600 mr-2"></div>
                <span className="text-xs text-red-800">Walls: {player2Walls}</span>
            </div>
        </div>

        {/* Opening */}
        <div className="bg-gray-100 p-3 rounded-lg border border-gray-300">
          <label className="text-sm font-medium text-gray-700 block mb-1">Opening</label>
          <select
            className="w-full border border-gray-300 rounded px-2 py-1 text-gray-800 bg-white focus:outline-none focus:ring-2 focus:ring-gray-500 disabled:opacity-50 disabled:cursor-not-allowed"
            value={selectedOpening}
            onChange={(e) => setSelectedOpening(e.target.value)}
            disabled={controlsDisabled}
          >
            {OPENINGS.map(opening => (
              <option key={opening} value={opening}>{opening}</option>
            ))}
          </select>
        </div>

         {/* AI Speed Control */}
         {isAiVsAiMode && (
            <div className="mt-4 bg-purple-50 p-3 rounded-lg border border-purple-200">
                <label className="text-sm font-medium text-purple-800 block mb-1">
                    AI Move Speed (ms)
                </label>
                <input
                    type="range"
                    min="10" // Faster minimum
                    max="2000"
                    step="10" // Finer steps
                    className="w-full h-2 bg-purple-200 rounded-lg appearance-none cursor-pointer disabled:opacity-50"
                    value={aiMoveSpeed}
                    onChange={(e) => setAiMoveSpeed(parseInt(e.target.value, 10))}
                    disabled={isLoadingWasm} // Disable only during loading
                />
                <div className="flex justify-between text-xs text-purple-800 mt-1">
                    <span>Fast</span>
                    <span>{aiMoveSpeed}ms</span>
                    <span>Slow</span>
                </div>
            </div>
        )}


        {/* Buttons */}
        <div className="flex flex-col space-y-2">
          <button
            className={`px-4 py-2 rounded-lg text-white font-medium shadow-md transition-colors ${
              isGameActive
                ? 'bg-red-600 hover:bg-red-700 active:bg-red-800 cursor-pointer'
                : 'bg-blue-600 hover:bg-blue-700 active:bg-blue-800 cursor-pointer'
            } ${isLoadingWasm || isThinking ? 'opacity-50 cursor-not-allowed' : ''}`}
            onClick={isGameActive ? onResetGame : onStartGame}
            disabled={isLoadingWasm || isThinking} // Disable if loading WASM or AI is thinking
          >
            {isGameActive ? 'Reset Game' : 'Start Game'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default Controls;