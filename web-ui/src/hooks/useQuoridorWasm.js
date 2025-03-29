// --- File: web-ui/src/hooks/useQuoridorWasm.js ---
import { useState, useEffect, useCallback, useRef } from 'react';
import init, { QuoridorGame as WasmQuoridor } from '@wasm/quoridor_wasm.js'; // Use alias

export function useQuoridorWasm() {
  const [wasmModuleLoaded, setWasmModuleLoaded] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);
  const gameInstanceRef = useRef(null);

  useEffect(() => {
    // ... (wasm loading logic - unchanged) ...
    const loadWasm = async () => {
        setIsLoading(true);
        setError(null);
        try {
            console.log('Initializing WASM module via alias...');
            await init();
             if (typeof WasmQuoridor === 'undefined') {
               throw new Error("WasmQuoridor constructor not found after init.");
            }
            console.log('WASM module initialized successfully.');
            setWasmModuleLoaded(true);
        } catch (err) {
            console.error('Failed to load WASM module:', err);
            setError(err);
        } finally {
            setIsLoading(false);
        }
    };
    loadWasm();
  }, []);

  // --- Interaction Functions ---
  const createGame = useCallback((size, walls) => {
    if (!wasmModuleLoaded || typeof WasmQuoridor === 'undefined') { setError(new Error('WASM not ready')); return null; }
    try {
         const game = new WasmQuoridor(size, walls);
         gameInstanceRef.current = game;
         return game; // Can return if needed, but ref is primary
    } catch (err) { setError(err); gameInstanceRef.current = null; return null; }
  }, [wasmModuleLoaded]); // Depend on module loaded

  const makeWasmMove = useCallback((moveStr) => {
    if (!gameInstanceRef.current) { console.error("makeWasmMove: No game instance"); return false; }
    try {
      return gameInstanceRef.current.make_move(moveStr);
    } catch (err) { console.error(`makeWasmMove error for ${moveStr}:`, err); setError(err); return false; }
  }, []); // No dependencies needed if gameInstanceRef is stable

  const getWasmGameState = useCallback(() => {
      if (!gameInstanceRef.current) { console.error("getWasmGameState: No game instance"); return null; }
      try {
          const stateJson = gameInstanceRef.current.getGameState(); // Use JS name
          return JSON.parse(stateJson);
      } catch (err) { console.error("getWasmGameState error:", err); setError(err); return null; }
   }, []);

   const getWasmAiMove = useCallback(() => {
       if(!gameInstanceRef.current) { console.error("getWasmAiMove: No game instance"); return ""; }
       try {
           return gameInstanceRef.current.get_ai_move();
       } catch (err) { console.error("getWasmAiMove error:", err); setError(err); return ""; }
   }, []);

   const setWasmStrategy = useCallback((playerNum, stratName, openingName) => {
       if(!gameInstanceRef.current) { console.error("setWasmStrategy: No game instance"); return false; }
       try {
           return gameInstanceRef.current.set_strategy(playerNum, stratName, openingName);
       } catch (err) { console.error("setWasmStrategy error:", err); setError(err); return false; }
   }, []);

   const resetWasmGame = useCallback(() => {
    if (!wasmModuleLoaded || typeof WasmQuoridor === 'undefined') {
      console.error("resetWasmGame: WASM not ready");
      setError(new Error('WASM not ready')); // Set error state
      return false;
    }
    try {
      console.log("Recreating WASM game instance on reset (Simplified)...");
      const size = 9; // Use constant BOARD_SIZE
      const walls = 10; // Use constant INITIAL_WALLS
      gameInstanceRef.current = new WasmQuoridor(size, walls); // Use imported constructor
      console.log("WASM game instance recreated successfully.");
      return true;
    } catch (err) {
      console.error("Error recreating WASM game instance:", err);
      setError(err); // Set error state
      return false;
    }
  }, [wasmModuleLoaded]);

   const getWasmLegalMoves = useCallback(() => {
        if (!gameInstanceRef.current) { console.error("getWasmLegalMoves: No game instance"); return { pawn: [], wall: [] }; }
        try {
            const pawnMovesResult = gameInstanceRef.current.getLegalMoves();
            const wallMovesResult = gameInstanceRef.current.getLegalWalls();
            const pawn = Array.isArray(pawnMovesResult) ? pawnMovesResult : [];
            const wall = Array.isArray(wallMovesResult) ? wallMovesResult : [];
            return { pawn, wall };
        } catch (err) { console.error("Error getting WASM legal moves:", err); setError(err); return { pawn: [], wall: [] }; }
   }, []);

   const checkWasmWin = useCallback((moveStr) => {
       if (!gameInstanceRef.current) { console.error("checkWasmWin: No game instance"); return false; }
       try {
           return gameInstanceRef.current.checkWin(moveStr); // Use JS name
       } catch (err) { console.error("checkWasmWin error:", err); setError(err); return false; }
   }, []);


  // --- RETURN THE FUNCTIONS ---
  return {
    wasmLoaded: wasmModuleLoaded,
    isLoadingWasm: isLoading,
    wasmError: error,
    createGame,
    // Add all the interaction functions here:
    makeWasmMove,
    getWasmGameState,
    getWasmAiMove,
    setWasmStrategy,
    resetWasmGame,
    getWasmLegalMoves,
    checkWasmWin,
  };
}