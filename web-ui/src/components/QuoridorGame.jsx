// --- File: web-ui/src/components/QuoridorGame.jsx ---
import React, { useState, useEffect, useCallback, useRef } from 'react';
import { AlertCircle } from 'lucide-react';

// Import Child Components
import QuoridorBoard from './QuoridorBoard';
import Controls from './Controls';
import History from './History';

// Import WASM Hook
import { useQuoridorWasm } from '../hooks/useQuoridorWasm'; // Adjust path if needed

const BOARD_SIZE = 9;
const INITIAL_WALLS = 10;

// Player enum - Define locally or import from a shared constants file
const Player = {
  PLAYER1: 'player1',
  PLAYER2: 'player2',
};

// Strategy names - Define locally or import
const STRATEGIES = [
    'Human', 'Random', 'ShortestPath', 'Defensive', 'Balanced', 'Adaptive',
    'Minimax1', 'Minimax2', 'Mirror', "MCTS10k", "MCTS60k", "MCTS1sec", // Example MCTS
    "SimulatedAnnealing0.5", "SimulatedAnnealing1.0", "SimulatedAnnealing2.0" // Example SA
];
// Opening names - Define locally or import
const OPENINGS = [
    'No Opening', 'Sidewall Opening', 'Standard Opening', 'Shiller Opening',
    'Stonewall', 'Ala Opening', 'Standard Opening (Symmetrical)', 'Rush Variation',
    'Gap Opening', 'Gap Opening (Mainline)', 'Anti-Gap', 'Sidewall',
    'Sidewall (Proper Counter)', 'Quick Box Variation', 'Shatranj Opening', 'Lee Inversion'
];

// Helper to get initial state
const getInitialBoardState = () => {
    const center = Math.floor(BOARD_SIZE / 2);
    return {
        size: BOARD_SIZE,
        hWalls: new Set(),
        vWalls: new Set(),
        player1Pos: { row: BOARD_SIZE - 1, col: center },
        player2Pos: { row: 0, col: center },
        player1Walls: INITIAL_WALLS,
        player2Walls: INITIAL_WALLS,
        activePlayer: Player.PLAYER1,
        moveHistory: [],
        lastMove: null,
    };
};


const QuoridorGameComponent = () => {
  // --- WASM Hook ---
  const {
    wasmLoaded,
    isLoadingWasm,
    wasmError,
    createGame,
    makeWasmMove,
    getWasmGameState,
    getWasmAiMove,
    setWasmStrategy,
    resetWasmGame,
    getWasmLegalMoves,
    checkWasmWin,
  } = useQuoridorWasm();

  // --- React State ---
  const [boardState, setBoardState] = useState(getInitialBoardState());
  const [player1Strategy, setPlayer1Strategy] = useState('Human');
  const [player2Strategy, setPlayer2Strategy] = useState('Adaptive'); // Default AI opponent
  const [selectedOpening, setSelectedOpening] = useState('No Opening');
  const [isGameActive, setIsGameActive] = useState(false);
  const [winner, setWinner] = useState(null);
  const [isThinking, setIsThinking] = useState(false); // For AI move progress
  const [message, setMessage] = useState('Select strategies and click Start Game.');
  const [nextPawnMoves, setNextPawnMoves] = useState([]); // Coords {row, col}
  const [nextWallMoves, setNextWallMoves] = useState({ h: [], v: [] }); // Coords {row, col}
  const [aiMoveSpeed, setAiMoveSpeed] = useState(500); // Default AI delay ms

  // Ref to prevent concurrent AI moves
  const isProcessingAiMove = useRef(false);

  // --- Coordinate Conversion Callbacks --- (memoized for stability)
  const toAlgebraicNotation = useCallback((row, col) => {
    if (row === undefined || col === undefined) return '??';
    const colLetter = String.fromCharCode(97 + col);
    const rowNumber = BOARD_SIZE - row;
    return `${colLetter}${rowNumber}`;
  }, []); // BOARD_SIZE is constant

  const fromAlgebraicNotation = useCallback((notation) => {
    if (!notation || typeof notation !== 'string' || notation.length < 2) return null;
    const colLetter = notation[0].toLowerCase();
    const col = colLetter.charCodeAt(0) - 97;
    if (col < 0 || col >= BOARD_SIZE) return null; // Invalid column
    const rowNumberStr = notation.substring(1).match(/^\d+/); // Match only digits at the start
    if (!rowNumberStr) return null;
    const rowNumber = parseInt(rowNumberStr[0], 10);
    if (isNaN(rowNumber) || rowNumber < 1 || rowNumber > BOARD_SIZE) return null; // Invalid row
    const row = BOARD_SIZE - rowNumber;
    return { row, col };
  }, []); // BOARD_SIZE is constant


  // --- Game State Update Functions ---

  // Fetches state from WASM and updates React state + legal moves
  const updateStateAndMovesFromWasm = useCallback(() => {
      if (!wasmLoaded) return false;
      console.log("Updating state and moves from WASM...");
      const wasmState = getWasmGameState(); // Fetch raw state first
      if (!wasmState) {
          console.error("Failed to get game state from WASM for update.");
          setMessage("Error: Could not retrieve game state.");
          return false;
      }

      // Update React board state
       const hWallsSet = new Set();
       const vWallsSet = new Set();
       wasmState.hWalls.forEach(alg => {
           const coord = fromAlgebraicNotation(alg);
           if (coord) hWallsSet.add(`${coord.row},${coord.col}`);
       });
       wasmState.vWalls.forEach(alg => {
           const coord = fromAlgebraicNotation(alg);
           if (coord) vWallsSet.add(`${coord.row},${coord.col}`);
       });

       setBoardState(prev => ({
          ...prev,
          player1Pos: wasmState.player1,
          player2Pos: wasmState.player2,
          player1Walls: wasmState.player1Walls,
          player2Walls: wasmState.player2Walls,
          hWalls: hWallsSet,
          vWalls: vWallsSet,
          activePlayer: wasmState.activePlayer === 1 ? Player.PLAYER1 : Player.PLAYER2,
          // Keep move history from React state, WASM doesn't track it
       }));

       // Update legal moves based on the *new* state from WASM
       const { pawn: legalPawnAlg, wall: legalWallAlg } = getWasmLegalMoves();

       const legalPawnCoords = legalPawnAlg
           .map(fromAlgebraicNotation)
           .filter(coord => coord !== null);
       setNextPawnMoves(legalPawnCoords);

       const legalHWallsCoords = legalWallAlg
           .filter(w => w.endsWith('h'))
           .map(w => fromAlgebraicNotation(w.slice(0, -1)))
           .filter(coord => coord !== null);
       const legalVWallsCoords = legalWallAlg
           .filter(w => w.endsWith('v'))
           .map(w => fromAlgebraicNotation(w.slice(0, -1)))
           .filter(coord => coord !== null);
       setNextWallMoves({ h: legalHWallsCoords, v: legalVWallsCoords });

        console.log("WASM state update complete. Active:", wasmState.activePlayer === 1 ? Player.PLAYER1 : Player.PLAYER2);
       return true;

  }, [wasmLoaded, getWasmGameState, getWasmLegalMoves, fromAlgebraicNotation]);


  // --- Game Action Handlers ---

  const handleStartGame = useCallback(() => {
      if (!wasmLoaded) {
          setMessage("WASM module not loaded yet.");
          return;
      }
      console.log("Starting game...");
      setMessage("Setting up new game...");
      setIsGameActive(false); // Ensure game is inactive during setup
      setWinner(null);
      isProcessingAiMove.current = false; // Reset AI processing flag

      // Reset WASM game state first
      if (!resetWasmGame()) {
           setMessage("Error resetting WASM game state.");
           return;
      }

       // Reset React state to initial
       setBoardState(getInitialBoardState());
       setNextPawnMoves([]);
       setNextWallMoves({ h: [], v: [] });


      // Set strategies in WASM *after* reset
      let success = true;
      if (player1Strategy !== 'Human') {
          if (!setWasmStrategy(1, player1Strategy, selectedOpening)) {
              console.error("Failed to set P1 strategy in WASM");
              success = false;
          }
      }
       if (player2Strategy !== 'Human') {
            if (!setWasmStrategy(2, player2Strategy, selectedOpening)) {
                console.error("Failed to set P2 strategy in WASM");
                success = false;
            }
       }

       if (!success) {
            setMessage("Error setting AI strategies. Please check console.");
            // Optionally prevent game start or proceed with Human defaults
       }

       // Short delay to allow state updates, then activate and update state/moves
       setTimeout(() => {
           console.log("Activating game and fetching initial state...");
           setIsGameActive(true);
            if (!updateStateAndMovesFromWasm()) { // Fetch initial state & legal moves
                 setMessage("Error fetching initial game state after start.");
                 setIsGameActive(false);
            } else {
                 setMessage(`Game started. ${selectedOpening !== 'No Opening' ? `Opening: ${selectedOpening}. ` : ''}Player 1's turn.`);
            }
       }, 100); // Small delay

  }, [wasmLoaded, player1Strategy, player2Strategy, selectedOpening, resetWasmGame, setWasmStrategy, updateStateAndMovesFromWasm]);

  const handleResetGame = useCallback(() => {
      console.log("Resetting game...");
      setMessage("Resetting game...");
      setIsGameActive(false);
      setWinner(null);
      setIsThinking(false);
      isProcessingAiMove.current = false;
      resetWasmGame(); // Reset WASM state
      setBoardState(getInitialBoardState()); // Reset React state
      setNextPawnMoves([]);
      setNextWallMoves({ h: [], v: [] });
      setMessage("Game reset. Select strategies and start.");
  }, [resetWasmGame]);


  const handleMakeMove = useCallback((moveStr, moveType, orientation = null) => {
       if (!isGameActive || winner) return false;

       console.log(`Attempting move: ${moveStr} (Type: ${moveType})`);

       // Check win condition *before* making move in WASM (which changes active player)
       const isWinningMove = moveType === 'pawn' && checkWasmWin(moveStr);

       const moveSuccess = makeWasmMove(moveStr); // Make move in WASM

       if (moveSuccess) {
            console.log("WASM move successful.");
            // Add to React history
            setBoardState(prev => ({
              ...prev,
              moveHistory: [...prev.moveHistory, {
                player: prev.activePlayer, // Player who *made* the move
                move: moveStr,
                type: moveType,
                orientation: orientation, // Null for pawn moves
                isWinningMove: isWinningMove
              }],
              lastMove: moveStr
            }));

            // Check for win immediately after successful WASM move
            if (isWinningMove) {
                const winningPlayer = boardState.activePlayer; // Player who made the winning move
                console.log(`${winningPlayer} wins!`);
                 setWinner(winningPlayer);
                 setIsGameActive(false);
                 setMessage(`${winningPlayer === Player.PLAYER1 ? 'Player 1 (Blue)' : 'Player 2 (Red)'} wins!`);
                 // Update state one last time, but don't fetch legal moves for winner
                  const wasmState = getWasmGameState();
                  if(wasmState) {
                       const hWallsSet = new Set(wasmState.hWalls.map(alg => { const c = fromAlgebraicNotation(alg); return c ? `${c.row},${c.col}` : null; }).filter(Boolean));
                       const vWallsSet = new Set(wasmState.vWalls.map(alg => { const c = fromAlgebraicNotation(alg); return c ? `${c.row},${c.col}` : null; }).filter(Boolean));
                       setBoardState(prev => ({
                            ...prev,
                             player1Pos: wasmState.player1, player2Pos: wasmState.player2,
                             player1Walls: wasmState.player1Walls, player2Walls: wasmState.player2Walls,
                             hWalls: hWallsSet, vWalls: vWallsSet,
                             activePlayer: wasmState.activePlayer === 1 ? Player.PLAYER1 : Player.PLAYER2, // Should be opponent now
                       }));
                  }
                 setNextPawnMoves([]); // Clear legal moves on win
                 setNextWallMoves({ h: [], v: [] });

            } else {
                 // Update React state and legal moves from WASM for the *next* player's turn
                 if (!updateStateAndMovesFromWasm()) {
                      setMessage("Error updating state after move.");
                      // Consider resetting or handling error
                 }
            }
            return true;
       } else {
           console.error(`WASM move failed: ${moveStr}`);
           setMessage(`Illegal move: ${moveStr}. Try again.`);
           // Refresh legal moves in case of discrepancy
            updateStateAndMovesFromWasm();
           return false;
       }
  }, [isGameActive, winner, makeWasmMove, checkWasmWin, updateStateAndMovesFromWasm, getWasmGameState, fromAlgebraicNotation, boardState.activePlayer]);

  const handleCellClick = useCallback((row, col) => {
      const currentStrategy = boardState.activePlayer === Player.PLAYER1 ? player1Strategy : player2Strategy;
      if (!isGameActive || winner || currentStrategy !== 'Human' || isThinking) return;

      const isLegal = nextPawnMoves.some(m => m.row === row && m.col === col);
      if (isLegal) {
          const moveStr = toAlgebraicNotation(row, col);
          handleMakeMove(moveStr, 'pawn');
      } else {
          setMessage("Invalid pawn move.");
      }
  }, [isGameActive, winner, boardState.activePlayer, player1Strategy, player2Strategy, isThinking, nextPawnMoves, toAlgebraicNotation, handleMakeMove]);

  const handleWallClick = useCallback((row, col, orientation) => {
      const currentStrategy = boardState.activePlayer === Player.PLAYER1 ? player1Strategy : player2Strategy;
      const wallsAvailable = boardState.activePlayer === Player.PLAYER1 ? boardState.player1Walls : boardState.player2Walls;
      if (!isGameActive || winner || currentStrategy !== 'Human' || isThinking || wallsAvailable <= 0) {
           if(wallsAvailable <= 0) setMessage("No walls left!");
           return;
      }

      // Check based on the bottom-left coord reference used by WASM/core logic
      const wallCoordRef = {row, col};
      const isLegal = nextWallMoves[orientation].some(w => w.row === wallCoordRef.row && w.col === wallCoordRef.col);

      if (isLegal) {
          const moveStr = toAlgebraicNotation(row, col) + orientation;
          handleMakeMove(moveStr, 'wall', orientation);
      } else {
          setMessage("Invalid wall placement.");
      }
  }, [isGameActive, winner, boardState.activePlayer, boardState.player1Walls, boardState.player2Walls, player1Strategy, player2Strategy, isThinking, nextWallMoves, toAlgebraicNotation, handleMakeMove]);

  // --- AI Move Execution ---
   const makeAiMove = useCallback(async () => {
      if (!isGameActive || winner || isProcessingAiMove.current) return;

       const currentStrategyName = boardState.activePlayer === Player.PLAYER1 ? player1Strategy : player2Strategy;
       if (currentStrategyName === 'Human') return; // Skip if human's turn

       console.log(`Requesting move for AI: ${currentStrategyName} (${boardState.activePlayer})`);
       setIsThinking(true);
       isProcessingAiMove.current = true;
       setMessage(`${currentStrategyName} is thinking...`);

       // Use setTimeout to allow UI update before potentially blocking WASM call
       await new Promise(resolve => setTimeout(resolve, 10)); // Short delay

       try {
           const moveStr = getWasmAiMove(); // Get move from WASM hook

           if (moveStr && moveStr.length > 0 && moveStr !== "resign") {
               console.log(`AI (${currentStrategyName}) chose: ${moveStr}`);
               setMessage(`${currentStrategyName} plays ${moveStr}`);
               // Determine move type based on string format
               const isWall = moveStr.length === 3 && (moveStr.endsWith('h') || moveStr.endsWith('v'));
               const moveType = isWall ? 'wall' : 'pawn';
               const orientation = isWall ? moveStr.substring(2, 3) : null;

               handleMakeMove(moveStr, moveType, orientation); // Use the common handler

           } else if (moveStr === "resign") {
               console.log(`AI (${currentStrategyName}) resigns.`);
               const winningPlayer = boardState.activePlayer === Player.PLAYER1 ? Player.PLAYER2 : Player.PLAYER1;
                setWinner(winningPlayer);
                setIsGameActive(false);
                setMessage(`${currentStrategyName} resigns. ${winningPlayer === Player.PLAYER1 ? 'Player 1' : 'Player 2'} wins!`);
           }
           else {
               console.error(`AI (${currentStrategyName}) returned invalid move: "${moveStr}"`);
               setMessage(`Error: AI (${currentStrategyName}) failed to find a valid move.`);
               // Potentially end game or revert? For now, just log.
                setIsGameActive(false); // Stop game on AI error?
           }
       } catch (error) {
            console.error("Error during AI move execution:", error);
            setMessage(`Critical AI Error: ${error.message || 'Unknown error'}`);
            setIsGameActive(false); // Stop game on critical error
       } finally {
            setIsThinking(false);
            isProcessingAiMove.current = false;
            console.log("AI move processing finished.");
       }

   }, [isGameActive, winner, boardState.activePlayer, player1Strategy, player2Strategy, getWasmAiMove, handleMakeMove]);


  // --- Effect for Triggering AI Moves ---
  useEffect(() => {
      if (isLoadingWasm || !isGameActive || winner || isThinking || isProcessingAiMove.current) {
          return; // Don't trigger AI if loading, game not active, ended, or already thinking
      }

      const currentStrategy = boardState.activePlayer === Player.PLAYER1 ? player1Strategy : player2Strategy;

      if (currentStrategy !== 'Human') {
           // Use setTimeout to schedule the AI move
           const timeoutId = setTimeout(() => {
               makeAiMove();
           }, aiMoveSpeed); // Use configurable speed

           // Cleanup function to clear timeout if component unmounts or state changes
           return () => clearTimeout(timeoutId);
      }
  }, [
      isLoadingWasm, isGameActive, winner, isThinking, boardState.activePlayer,
      player1Strategy, player2Strategy, makeAiMove, aiMoveSpeed
  ]);

  // --- Effect for Initial WASM Game Creation ---
   useEffect(() => {
       if (wasmLoaded && !isLoadingWasm) {
           console.log("WASM loaded, creating initial game instance...");
           createGame(BOARD_SIZE, INITIAL_WALLS);
           // Don't fetch state here yet, wait for startGame
       }
   }, [wasmLoaded, isLoadingWasm, createGame]);


  // --- Render Logic ---

  // Loading/Error states
  if (isLoadingWasm) {
    return <div className="flex justify-center items-center h-screen">Loading WebAssembly...</div>;
  }
  if (wasmError) {
    return <div className="text-red-600 p-4">Error loading WASM: {wasmError.message || JSON.stringify(wasmError)}</div>;
  }

  const isAiVsAi = player1Strategy !== 'Human' && player2Strategy !== 'Human';

  return (
    <div className="flex flex-col items-center p-4 md:p-6 min-h-screen bg-gray-100 font-sans">
      {/* Header */}
      <div className="w-full max-w-6xl bg-white rounded-t-lg shadow-md p-4 flex justify-between items-center border-b-2 border-gray-200 mb-4">
        <h1 className="text-2xl md:text-3xl font-bold text-gray-800">Quoridor AI Arena</h1>
        {isGameActive && (
             <div className={`flex items-center ${boardState.activePlayer === Player.PLAYER1 ? 'text-blue-600' : 'text-red-600'} font-semibold`}>
                 <div className={`h-4 w-4 rounded-full ${boardState.activePlayer === Player.PLAYER1 ? 'bg-blue-600' : 'bg-red-600'} mr-2 animate-pulse`}></div>
                 <span>{boardState.activePlayer === Player.PLAYER1 ? 'Player 1' : 'Player 2'}'s Turn</span>
             </div>
        )}
      </div>

      {/* Main Layout: Controls | Board + Status | History */}
      <div className="w-full max-w-6xl flex flex-col lg:flex-row bg-white shadow-xl rounded-b-lg">

        {/* Controls Sidebar */}
        <Controls
            strategies={STRATEGIES}
            openings={OPENINGS}
            player1Strategy={player1Strategy}
            setPlayer1Strategy={setPlayer1Strategy}
            player2Strategy={player2Strategy}
            setPlayer2Strategy={setPlayer2Strategy}
            selectedOpening={selectedOpening}
            setSelectedOpening={setSelectedOpening}
            onStartGame={handleStartGame}
            onResetGame={handleResetGame}
            isGameActive={isGameActive}
            isLoadingWasm={isLoadingWasm}
            isThinking={isThinking}
            aiMoveSpeed={aiMoveSpeed}
            setAiMoveSpeed={setAiMoveSpeed}
            isAiVsAiMode={isAiVsAi}
            player1Walls={boardState.player1Walls} // Pass wall counts
            player2Walls={boardState.player2Walls}
        />

        {/* Center Area: Board and Status Message */}
        <div className="flex-1 p-4 md:p-6 flex flex-col items-center">
           {/* Status Message Area */}
          <div className="w-full mb-4 min-h-[40px] flex items-center justify-center">
              {message && (
                  <div className={`px-4 py-2 rounded-lg text-center text-sm md:text-base transition-colors duration-300 ${
                      winner
                          ? (winner === Player.PLAYER1
                              ? 'bg-blue-100 text-blue-800 font-bold'
                              : 'bg-red-100 text-red-800 font-bold')
                          : 'bg-gray-100 text-gray-700'
                  }`}>
                      {message}
                  </div>
              )}
              {isThinking && (
                  <div className="ml-4 bg-purple-100 px-3 py-1 rounded-full text-purple-800 text-xs flex items-center animate-pulse">
                      <AlertCircle size={14} className="mr-1" />
                      Thinking...
                  </div>
              )}
          </div>

          {/* Game Board */}
          <div className="mb-4 md:mb-6">
              <QuoridorBoard
                  boardState={boardState}
                  onCellClick={handleCellClick}
                  onWallClick={handleWallClick}
                  nextPawnMoves={nextPawnMoves}
                  nextWallMoves={nextWallMoves}
                  player1Strategy={player1Strategy} // Pass strategy to board for disabling clicks
                  player2Strategy={player2Strategy}
              />
          </div>
        </div>

        {/* History Sidebar */}
        <History moveHistory={boardState.moveHistory} />

      </div>

       {/* Optional: Instructions or Footer */}
       <div className="mt-8 w-full max-w-6xl text-center text-xs text-gray-500">
            Quoridor Game Interface - Integrating Rust WASM
       </div>
    </div>
  );
};

export default QuoridorGameComponent;