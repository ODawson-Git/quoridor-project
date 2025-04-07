// --- File: web-ui/src/components/Controls.jsx ---
import React, { useCallback } from 'react';

// Define OPENINGS constant if not passed as props or imported
// (This was removed accidentally during refactoring)
const OPENINGS = [
    'No Opening', 'Sidewall Opening', 'Standard Opening', 'Shiller Opening',
    'Stonewall', 'Ala Opening', 'Standard Opening (Symmetrical)', 'Rush Variation',
    'Gap Opening', 'Gap Opening (Mainline)', 'Anti-Gap', 'Sidewall',
    'Sidewall (Proper Counter)', 'Quick Box Variation', 'Shatranj Opening', 'Lee Inversion'
];

// Helper component for strategy parameters
const StrategyParams = ({ strategy, setStrategy, isDisabled, playerType }) => {
    const handleParamChange = useCallback((paramName, value) => {
        // Ensure numeric values are stored as numbers
        const numericValue = !isNaN(parseFloat(value)) ? parseFloat(value) : value;
        setStrategy(prev => ({
            ...prev,
            params: {
                ...prev.params,
                [paramName]: numericValue,
            }
        }));
    }, [setStrategy]);

    if (!strategy || strategy.baseName === 'Human') {
        return null; // No params for Human
    }

    // Set colors based on player type
    const bgColorClass = playerType === 'player1' ? 'bg-blue-50' : 'bg-red-50';
    const borderColorClass = playerType === 'player1' ? 'border-blue-300' : 'border-red-300';
    const ringColorClass = playerType === 'player1' ? 'focus:ring-blue-500' : 'focus:ring-red-500';
    const textColorClass = playerType === 'player1' ? 'text-blue-800' : 'text-red-800';

    const inputClass = `w-full border ${borderColorClass} rounded px-2 py-1 text-sm ${bgColorClass} ${textColorClass} focus:outline-none focus:ring-1 ${ringColorClass} disabled:opacity-50`;
    const labelClass = `text-xs font-medium ${textColorClass} block mb-0.5`;

    switch (strategy.baseName) {
        case 'Minimax':
            return (
                <div className="mt-2 space-y-1">
                    <label className={labelClass}>Depth (1-5):</label>
                    <input
                        type="text"
                        placeholder="e.g., 1, 2, or 3"
                        value={strategy.params?.depthText || strategy.params?.depth || "1"}
                        onChange={(e) => {
                            const textValue = e.target.value;
                            let numericValue = parseInt(textValue, 10);
                            if (isNaN(numericValue) || numericValue < 1) numericValue = 1;
                            if (numericValue > 5) numericValue = 5; // Cap at 5 for performance
                            
                            setStrategy(prev => ({
                                ...prev,
                                params: {
                                    ...prev.params,
                                    depthText: textValue, // Keep text for display
                                    depth: numericValue, // Store parsed number
                                }
                            }));
                        }}
                        className={inputClass}
                        disabled={isDisabled}
                    />
                </div>
            );
        case 'MCTS':
            return (
                <div className="mt-2 space-y-1">
                    <label className={labelClass}>Simulations:</label>
                     <input
                        type="text" // Use text to allow 'k' suffix
                        placeholder="e.g., 10k or 5000"
                        value={strategy.params?.simulationsText || '10k'} // Store text separately
                        onChange={(e) => {
                             const textValue = e.target.value;
                             let numericValue = parseInt(textValue.replace(/k$/i, '000'), 10);
                             if (isNaN(numericValue) || numericValue <= 0) numericValue = 10000; // Default/fallback

                             setStrategy(prev => ({
                                ...prev,
                                params: {
                                    ...prev.params,
                                    simulationsText: textValue, // Keep text for display
                                    simulations: numericValue, // Store parsed number
                                }
                            }));
                        }}
                        className={inputClass}
                        disabled={isDisabled}
                    />
                    {/* Optional: Time limit (less common for WASM)
                    <label className={labelClass}>Time Limit (sec):</label>
                    <input type="number" min="0.1" step="0.1" value={strategy.params?.time || 1.0} onChange={(e) => handleParamChange('time', e.target.value)} className={inputClass} disabled={isDisabled}/>
                    */}
                </div>
            );
        case 'SimulatedAnnealing':
            return (
                <div className="mt-2 space-y-1">
                    <label className={labelClass}>Factor:</label>
                    <input
                        type="text"
                        placeholder="e.g., 0.5, 1.0, 2.0"
                        value={strategy.params?.factorText || strategy.params?.factor || "1.0"}
                        onChange={(e) => {
                            const textValue = e.target.value;
                            let numericValue = parseFloat(textValue);
                            if (isNaN(numericValue) || numericValue < 0.1) numericValue = 1.0;
                            
                            setStrategy(prev => ({
                                ...prev,
                                params: {
                                    ...prev.params,
                                    factorText: textValue, // Keep text for display
                                    factor: numericValue, // Store parsed number
                                }
                            }));
                        }}
                        className={inputClass}
                        disabled={isDisabled}
                    />
                </div>
            );
        // Add cases for Defensive, Balanced if they need parameters exposed
        default:
            return null; // No parameters for other strategies yet
    }
};


const Controls = ({
  baseStrategies, // Renamed from strategies
  openings, // Pass openings array
  player1Strategy, setPlayer1Strategy, // Strategy objects
  player2Strategy, setPlayer2Strategy,
  selectedOpening, setSelectedOpening,
  onStartGame, onResetGame,
  isGameActive, isLoadingWasm, isThinking,
  aiMoveSpeed, setAiMoveSpeed, isAiVsAiMode,
  player1Walls, player2Walls
}) => {
    // Determine if controls should be disabled
    const controlsDisabled = isGameActive || isLoadingWasm;

    // Handler for changing the base strategy selection
    const handleBaseStrategyChange = useCallback((playerSetter, newBaseName) => {
        let defaultParams = {};
        // Set sensible defaults when switching strategy type
        switch (newBaseName) {
            case 'Minimax': defaultParams = { depth: 1 }; break;
            case 'MCTS': defaultParams = { simulations: 10000, simulationsText: '10k' }; break;
            case 'SimulatedAnnealing': defaultParams = { factor: 1.0 }; break;
            // Add other defaults if needed
        }
        playerSetter({ baseName: newBaseName, params: defaultParams });
    }, []);


  return (
    <div className="w-full lg:w-72 p-4 bg-gray-50 rounded-bl-lg border-r border-gray-200 flex-shrink-0"> {/* Adjusted width */}
      <div className="space-y-4">
        {/* Player 1 */}
        <div className="bg-blue-50 p-3 rounded-lg border border-blue-200">
          <label className="text-sm font-medium text-blue-800 block mb-1">Player 1 (Blue)</label>
          <select
            className="w-full border border-blue-300 rounded px-2 py-1 text-blue-800 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            value={player1Strategy.baseName} // Bind to baseName
            onChange={(e) => handleBaseStrategyChange(setPlayer1Strategy, e.target.value)}
            disabled={controlsDisabled}
          >
            {baseStrategies.map(strategyName => (
              <option key={`p1-${strategyName}`} value={strategyName}>{strategyName}</option>
            ))}
          </select>
          {/* Render Params for Player 1 */}
          <StrategyParams
              strategy={player1Strategy}
              setStrategy={setPlayer1Strategy}
              isDisabled={controlsDisabled}
              playerType="player1"
          />
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
            value={player2Strategy.baseName} // Bind to baseName
            onChange={(e) => handleBaseStrategyChange(setPlayer2Strategy, e.target.value)}
            disabled={controlsDisabled}
          >
             {baseStrategies.map(strategyName => (
              <option key={`p2-${strategyName}`} value={strategyName}>{strategyName}</option>
            ))}
          </select>
           {/* Render Params for Player 2 */}
           <StrategyParams
              strategy={player2Strategy}
              setStrategy={setPlayer2Strategy}
              isDisabled={controlsDisabled}
              playerType="player2"
          />
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
            {(openings || OPENINGS).map(opening => (
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