// --- File: web-ui/src/components/QuoridorBoard.jsx ---
import React, { useState, memo } from 'react';

// Memoize the component to prevent unnecessary re-renders if props haven't changed significantly
const QuoridorBoard = memo(({
  boardState,
  onCellClick,
  onWallClick,
  nextPawnMoves,
  nextWallMoves,
  player1Strategy,
  player2Strategy
}) => {
  // State for showing ghost walls on hover
  const [ghostWall, setGhostWall] = useState(null); // { type: 'h'/'v', row: r, col: c }

  // Check if the current active player is human
  const isHumanTurn = (
    (boardState.activePlayer === 'player1' && player1Strategy === 'Human') ||
    (boardState.activePlayer === 'player2' && player2Strategy === 'Human')
  );

  // --- Helper Functions ---

  // Convert 0-based row/col to algebraic notation (e.g., 0,0 -> a9)
  const toAlgebraicNotation = (row, col) => {
    if (row === undefined || col === undefined) return '??';
    const colLetter = String.fromCharCode(97 + col);
    const rowNumber = boardState.size - row;
    return `${colLetter}${rowNumber}`;
  };

  // Determine the CSS background color for a cell
  const getCellBackgroundColor = (row, col) => {
    if (row === 0) return 'bg-red-100 hover:bg-red-200'; // Player 2 goal line (P1 target)
    if (row === boardState.size - 1) return 'bg-blue-100 hover:bg-blue-200'; // Player 1 goal line (P2 target)
    return (row + col) % 2 === 0 ? 'bg-gray-50 hover:bg-gray-100' : 'bg-white hover:bg-gray-100';
  };

  // Check if a coordinate represents a legal pawn move for the active player
  const isLegalPawnMove = (row, col) => {
      if (!isHumanTurn) return false; // Don't highlight if not human's turn
      // Ensure nextPawnMoves is an array before using `some`
      return Array.isArray(nextPawnMoves) && nextPawnMoves.some(move => move.row === row && move.col === col);
  };

  // Check if a wall placement slot is legal for the active player
  const isLegalWallPlacement = (row, col, orientation) => {
      if (!isHumanTurn) return false;
      const wallsAvailable = boardState.activePlayer === 'player1' ? boardState.player1Walls : boardState.player2Walls;
      if (wallsAvailable <= 0) return false; // No walls left

      const legalSlots = nextWallMoves[orientation];
      // Ensure legalSlots is an array
      return Array.isArray(legalSlots) && legalSlots.some(wall => wall.row === row && wall.col === col);
  };

  // Get the color for a placed wall based on move history
  const getWallOwner = (row, col, orientation) => {
      const wallKey = `${row},${col}`;
      const wallExists = orientation === 'h' ? boardState.hWalls.has(wallKey) : boardState.vWalls.has(wallKey);
      if (!wallExists) return null; // No wall here

      // Find the move in history that placed this specific wall
      // Need to convert back to algebraic for reliable history check
       const algebraicPos = toAlgebraicNotation(row, col);
       const fullWallNotation = `${algebraicPos}${orientation}`;

      const wallMove = boardState.moveHistory?.find(move => move.move === fullWallNotation && move.type === 'wall');

      return wallMove ? wallMove.player : null; // Return player or null if not found
  };

  const getWallColor = (owner) => {
      if (owner === 'player1') return 'bg-blue-500 border-blue-700';
      if (owner === 'player2') return 'bg-red-500 border-red-700';
      return 'bg-gray-400 border-gray-600'; // Fallback/Error color
  };


  // --- Render Logic ---

  return (
    // Container with relative positioning for walls
    <div className="relative w-[540px] h-[540px] bg-stone-200 border-4 border-stone-600 shadow-lg rounded-md p-2 box-content">

      {/* Grid for Cells */}
      <div className={`grid grid-cols-${boardState.size} grid-rows-${boardState.size} w-full h-full gap-1`}>
        {Array.from({ length: boardState.size }).map((_, row) =>
          Array.from({ length: boardState.size }).map((_, col) => {
            const isP1 = boardState.player1Pos.row === row && boardState.player1Pos.col === col;
            const isP2 = boardState.player2Pos.row === row && boardState.player2Pos.col === col;
            const isLegalMove = isLegalPawnMove(row, col);
            const cellId = `cell-${row}-${col}`;
            const cellAlg = toAlgebraicNotation(row, col);

            return (
              <div
                key={cellId}
                id={cellId}
                className={`
                  relative flex items-center justify-center rounded
                  ${getCellBackgroundColor(row, col)}
                  transition-colors duration-150
                  ${isLegalMove ? 'cursor-pointer ring-2 ring-green-500 ring-inset' : ''}
                  ${!isHumanTurn && !isP1 && !isP2 ? 'cursor-default' : ''}
                  ${isHumanTurn && !isLegalMove && !isP1 && !isP2 ? 'cursor-not-allowed' : ''}
                `}
                onClick={() => isLegalMove && onCellClick(row, col)}
                title={cellAlg} // Tooltip with algebraic notation
              >
                {/* Pawns */}
                {isP1 && (
                  <div className="absolute h-10 w-10 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 z-20 shadow-lg border-2 border-white flex items-center justify-center text-white font-bold text-lg select-none">
                    1
                  </div>
                )}
                {isP2 && (
                  <div className="absolute h-10 w-10 rounded-full bg-gradient-to-br from-red-400 to-red-600 z-20 shadow-lg border-2 border-white flex items-center justify-center text-white font-bold text-lg select-none">
                    2
                  </div>
                )}

                {/* Legal Move Indicator (only if human turn and legal) */}
                {isLegalMove && (
                  <div className={`absolute h-3 w-3 rounded-full ${boardState.activePlayer === 'player1' ? 'bg-blue-300' : 'bg-red-300'} opacity-80 z-10 pointer-events-none`}></div>
                )}

                 {/* Optional: Cell notation for debugging */}
                 {/* <span className="absolute bottom-0 right-1 text-gray-400 text-[8px] pointer-events-none">{cellAlg}</span> */}
              </div>
            );
          })
        )}
      </div>

      {/* Layer for Horizontal Walls and Placement Areas */}
      <div className="absolute inset-0 pointer-events-none">
        {Array.from({ length: boardState.size - 1 }).map((_, row) => // 8 rows of wall slots
          Array.from({ length: boardState.size - 1 }).map((_, col) => { // 8 cols of wall slots
            // Wall placement coordinate (bottom-left reference square)
            const wallRefRow = row + 1;
            const wallRefCol = col;
            const wallCoordKey = `${wallRefRow},${wallRefCol}`; // Key for checking placed walls
            const isPlaced = boardState.hWalls.has(wallCoordKey);
            const isLegal = isLegalWallPlacement(wallRefRow, wallRefCol, 'h');
            const owner = isPlaced ? getWallOwner(wallRefRow, wallRefCol, 'h') : null;

            // Ghost wall logic
             const showGhost = isHumanTurn && !isPlaced && ghostWall &&
                             ghostWall.type === 'h' &&
                             ghostWall.row === wallRefRow &&
                              // Ghost covers two columns
                             (ghostWall.col === wallRefCol || ghostWall.col === wallRefCol -1);


            return (
              // Clickable area for placing horizontal walls
              <div
                key={`hwall-area-${row}-${col}`}
                className={`absolute z-10 pointer-events-auto
                            ${isHumanTurn && isLegal && !isPlaced ? 'cursor-pointer group' : ''}
                            ${isHumanTurn && !isLegal && !isPlaced ? 'cursor-not-allowed' : ''}
                           `}
                style={{
                  // Position the center of the clickable area slightly above the grid line
                  top: `calc(${(row + 1) * (100 / boardState.size)}% - 6px)`, // 12px height / 2
                  left: `calc(${col * (100 / boardState.size)}% + (100 / ${boardState.size} / 2)%)`, // Center between cells horizontally
                  width: `calc(${(100 / boardState.size)}% * 1)`, // Span one cell width for clicking
                  height: '12px', // Clickable height
                }}
                onMouseEnter={() => isLegal && !isPlaced && setGhostWall({ type: 'h', row: wallRefRow, col: wallRefCol })}
                onMouseLeave={() => setGhostWall(null)}
                onClick={(e) => {
                    e.stopPropagation(); // Prevent clicks falling through to cells
                    if (isHumanTurn && isLegal && !isPlaced) {
                        onWallClick(wallRefRow, wallRefCol, 'h');
                    }
                }}
              >
                {/* Visual representation of placed wall or ghost wall */}
                {(isPlaced || showGhost) && (
                    <div
                        className={`absolute top-0 left-0 w-[calc(100%*2+4px)] h-2.5 rounded-full border shadow-md
                                    ${isPlaced ? getWallColor(owner) : 'bg-gray-400 bg-opacity-40 border-gray-500'}
                                   `}
                         // Wall spans 2 cells + the gap (adjust width slightly using calc)
                         // Left offset might need slight adjustment depending on gap/border size
                        style={{ transform: 'translateX(calc(-100%/4 - 1px))' }} // Center the visual wall over the gap
                    ></div>
                )}
                 {/* Optional: Legal placement indicator */}
                 {isLegal && !isPlaced && !showGhost && (
                     <div className="absolute top-1/2 left-1/2 w-2 h-2 bg-green-400 rounded-full opacity-0 group-hover:opacity-50 transform -translate-x-1/2 -translate-y-1/2 transition-opacity"></div>
                 )}
              </div>
            );
          })
        )}
      </div>

      {/* Layer for Vertical Walls and Placement Areas */}
       <div className="absolute inset-0 pointer-events-none">
           {Array.from({ length: boardState.size - 1 }).map((_, row) => // 8 rows of wall slots
               Array.from({ length: boardState.size - 1 }).map((_, col) => { // 8 cols of wall slots
                   const wallRefRow = row + 1; // Reference row (bottom-left)
                   const wallRefCol = col;     // Reference col (bottom-left)
                   const wallCoordKey = `${wallRefRow},${wallRefCol}`;
                   const isPlaced = boardState.vWalls.has(wallCoordKey);
                   const isLegal = isLegalWallPlacement(wallRefRow, wallRefCol, 'v');
                   const owner = isPlaced ? getWallOwner(wallRefRow, wallRefCol, 'v') : null;

                    // Ghost wall logic
                    const showGhost = isHumanTurn && !isPlaced && ghostWall &&
                                    ghostWall.type === 'v' &&
                                    ghostWall.col === wallRefCol &&
                                    // Ghost covers two rows
                                    (ghostWall.row === wallRefRow || ghostWall.row === wallRefRow - 1);


                   return (
                       <div
                           key={`vwall-area-${row}-${col}`}
                           className={`absolute z-10 pointer-events-auto
                                       ${isHumanTurn && isLegal && !isPlaced ? 'cursor-pointer group' : ''}
                                        ${isHumanTurn && !isLegal && !isPlaced ? 'cursor-not-allowed' : ''}
                                       `}
                           style={{
                               // Position the center of the clickable area slightly left of the grid line
                                top: `calc(${row * (100 / boardState.size)}% + (100 / ${boardState.size} / 2)%)`, // Center between cells vertically
                                left: `calc(${(col + 1) * (100 / boardState.size)}% - 6px)`, // 12px width / 2
                                height: `calc(${(100 / boardState.size)}% * 1)`, // Span one cell height for clicking
                                width: '12px', // Clickable width
                           }}
                           onMouseEnter={() => isLegal && !isPlaced && setGhostWall({ type: 'v', row: wallRefRow, col: wallRefCol })}
                           onMouseLeave={() => setGhostWall(null)}
                           onClick={(e) => {
                               e.stopPropagation();
                               if (isHumanTurn && isLegal && !isPlaced) {
                                   onWallClick(wallRefRow, wallRefCol, 'v');
                               }
                           }}
                       >
                           {(isPlaced || showGhost) && (
                               <div
                                   className={`absolute top-0 left-0 h-[calc(100%*2+4px)] w-2.5 rounded-full border shadow-md
                                                ${isPlaced ? getWallColor(owner) : 'bg-gray-400 bg-opacity-40 border-gray-500'}
                                               `}
                                    // Wall spans 2 cells + the gap
                                   style={{ transform: 'translateY(calc(-100%/4 - 1px))' }} // Center the visual wall over the gap
                               ></div>
                           )}
                            {/* Optional: Legal placement indicator */}
                            {isLegal && !isPlaced && !showGhost && (
                                <div className="absolute top-1/2 left-1/2 w-2 h-2 bg-green-400 rounded-full opacity-0 group-hover:opacity-50 transform -translate-x-1/2 -translate-y-1/2 transition-opacity"></div>
                            )}
                       </div>
                   );
               })
           )}
       </div>

    </div>
  );
}); // Wrap with memo

export default QuoridorBoard;