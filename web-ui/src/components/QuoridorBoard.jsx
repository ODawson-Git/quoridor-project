import React, { useState, useEffect } from 'react';

const QuoridorBoard = ({ 
  boardState, 
  onCellClick, 
  onWallClick, 
  nextPawnMoves, 
  nextWallMoves,
  player1Strategy,
  player2Strategy
}) => {
  // State for showing ghost walls on hover
  const [ghostWall, setGhostWall] = useState(null);
  
  // Current player can make a move
  const canCurrentPlayerMove = (
    (boardState.activePlayer === 'player1' && player1Strategy === 'Human') ||
    (boardState.activePlayer === 'player2' && player2Strategy === 'Human')
  );
  
  // Convert to algebraic notation for display
  const toAlgebraicNotation = (row, col) => {
    const colLetter = String.fromCharCode(97 + col); // 'a' is 0
    const rowNumber = 9 - row; // Convert from array (0-8 from top) to algebraic (1-9 from bottom)
    return `${colLetter}${rowNumber}`;
  };
  
  // Get wall color based on wall position and move history
  const getWallColor = (player) => {
    if (player === 'player1') return 'bg-blue-500';
    if (player === 'player2') return 'bg-red-500';
    return 'bg-gray-600';
  };
  
  // Find which player placed a specific wall - improved version
  const getWallPlayer = (orientation, row, col) => {
    // Convert to algebraic notation for matching against move history
    const algebraicPosition = toAlgebraicNotation(row, col);
    const wallNotation = `${algebraicPosition}${orientation}`;
    
    // Find the move in history matching this wall
    const wallMove = boardState.moveHistory?.find(move => 
      move.type === 'wall' && 
      move.orientation === orientation &&
      move.move.startsWith(algebraicPosition)
    );
    
    // If found in history, return that player
    if (wallMove) {
      return wallMove.player;
    }
    
    // Fallback color assignment based on position in move sequence
    // This ensures walls always have a consistent color even if history matching fails
    const index = [...boardState.hWalls, ...boardState.vWalls].indexOf(`${row},${col}`);
    return index % 2 === 0 ? 'player1' : 'player2';
  };
  
  // Check if a cell is a legal move
  const isLegalMove = (row, col) => {
    // Get current player position
    const currentPos = boardState.activePlayer === 'player1' 
      ? boardState.player1Pos 
      : boardState.player2Pos;
    
    // Skip the current player's position as a legal move
    if (row === currentPos.row && col === currentPos.col) {
      return false;
    }
    
    return nextPawnMoves.some(move => 
      move.row === row && move.col === col
    );
  };

  // Check if a wall position is legal
  const isLegalWall = (row, col, type) => {
    return nextWallMoves[type].some(wall => 
      wall.row === row && wall.col === col
    );
  };

  // Get ghost wall color based on current player
  const getGhostWallColor = () => {
    return boardState.activePlayer === 'player1' ? 'bg-blue-400 bg-opacity-60' : 'bg-red-400 bg-opacity-60';
  };

  // Determine if a cell is in the target row (goal line)
  const isTargetRow = (row, col) => {
    // Player 1 (blue) target is row 0 (top row)
    // Player 2 (red) target is row 8 (bottom row)
    return row === 0 || row === 8;
  };

  // Get cell background color considering checkerboard pattern and highlighting
  const getCellBackgroundColor = (row, col) => {
    // Target row highlighting
    if (row === 0) return 'bg-red-100'; // Player 2's target (top row)
    if (row === 8) return 'bg-blue-100'; // Player 1's target (bottom row)
    
    // Checkerboard pattern for the rest of the board
    return (row + col) % 2 === 0 ? 'bg-gray-50' : 'bg-white';
  };

  return (
    <div className="relative w-[540px] h-[540px] bg-white border border-gray-300 rounded-lg shadow-xl overflow-hidden">
      {/* Top target line (Player 1's goal) */}
      <div className="absolute top-0 left-0 w-full h-1.5 bg-red-500 z-20"></div>
      
      {/* Bottom target line (Player 2's goal) */}
      <div className="absolute bottom-0 left-0 w-full h-1.5 bg-blue-500 z-20"></div>
      
      {/* Main grid - cells */}
      <div className="grid grid-cols-9 grid-rows-9 w-full h-full">
        {Array(9).fill(0).map((_, row) => (
          Array(9).fill(0).map((_, col) => {
            const isPlayer1 = boardState.player1Pos.row === row && boardState.player1Pos.col === col;
            const isPlayer2 = boardState.player2Pos.row === row && boardState.player2Pos.col === col;
            const cellIsLegalMove = isLegalMove(row, col);
            const cellNotation = toAlgebraicNotation(row, col);
            
            // Determine if this cell is on the edge (for styling)
            const isEdgeCell = row === 0 || row === 8 || col === 0 || col === 8;
            
            return (
              <div 
                key={`cell-${row}-${col}`} 
                className={`
                  relative flex items-center justify-center
                  ${getCellBackgroundColor(row, col)}
                  ${cellIsLegalMove && canCurrentPlayerMove ? 'bg-green-100 cursor-pointer hover:bg-green-200' : ''}
                  ${isPlayer1 || isPlayer2 ? 'bg-gray-200' : ''}
                  ${isEdgeCell ? 'border border-gray-300' : 'border border-gray-200'}
                  transition-colors duration-150
                `}
                onClick={() => {
                  if (canCurrentPlayerMove && cellIsLegalMove) {
                    onCellClick(row, col);
                  }
                }}
              >
                {isPlayer1 && (
                  <div className="h-10 w-10 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 z-10 shadow-lg border-2 border-blue-300 flex items-center justify-center text-white font-bold">
                    1
                  </div>
                )}
                {isPlayer2 && (
                  <div className="h-10 w-10 rounded-full bg-gradient-to-br from-red-400 to-red-600 z-10 shadow-lg border-2 border-red-300 flex items-center justify-center text-white font-bold">
                    2
                  </div>
                )}
                
                <div className="absolute text-xs text-gray-500 left-1 top-1 pointer-events-none">
                  {cellNotation}
                </div>
                
                {/* Visual indicator for legal moves */}
                {cellIsLegalMove && canCurrentPlayerMove && !isPlayer1 && !isPlayer2 && (
                  <div className={`h-3 w-3 rounded-full ${boardState.activePlayer === 'player1' ? 'bg-blue-500' : 'bg-red-500'} opacity-70`}></div>
                )}
              </div>
            );
          })
        ))}
      </div>
      
      {/* Horizontal wall areas - appear between rows */}
      <div className="absolute top-0 left-0 w-full h-full pointer-events-none">
        {Array(8).fill(0).map((_, row) => (
          Array(8).fill(0).map((_, col) => {
            // For horizontal walls, the reference point is the cell below the wall
            const wallRow = row + 1;
            const wallCol = col;
            const wallCoord = `${wallRow},${wallCol}`;
            const wallExists = boardState.hWalls.has(wallCoord);
            const wallIsLegal = isLegalWall(wallRow, wallCol, 'h');
            const showGhost = ghostWall && 
                            ghostWall.type === 'h' && 
                            ghostWall.row === wallRow && 
                            ghostWall.col === wallCol;
            
            return (
              <div 
                key={`hwall-${row}-${col}`}
                className={`
                  absolute pointer-events-auto z-10
                  ${wallIsLegal && canCurrentPlayerMove ? 'cursor-pointer' : ''}
                  ${wallIsLegal && canCurrentPlayerMove && !wallExists && !showGhost ? 'hover:bg-gray-200 hover:bg-opacity-50 rounded-full' : ''}
                `}
                style={{
                  top: `${(row + 1) * (100 / 9)}%`,
                  left: `${col * (100 / 9)}%`,
                  width: `${(100 / 9) * 2}%`,
                  height: '12px',
                  transform: 'translateY(-50%)'
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  if (canCurrentPlayerMove && wallIsLegal) {
                    onWallClick(wallRow, wallCol, 'h');
                  }
                }}
                onMouseEnter={() => {
                  if (canCurrentPlayerMove && wallIsLegal) {
                    setGhostWall({ type: 'h', row: wallRow, col: wallCol });
                  }
                }}
                onMouseLeave={() => setGhostWall(null)}
              >
                {(wallExists || showGhost) && (
                  <div 
                    className={`
                      absolute top-1/2 left-0 w-full h-4 -translate-y-1/2 shadow-md
                      ${wallExists 
                        ? getWallColor(getWallPlayer('h', wallRow, wallCol)) 
                        : getGhostWallColor()}
                      rounded-full
                    `}
                  />
                )}
              </div>
            );
          })
        ))}
      </div>
      
      {/* Vertical wall areas - appear between columns */}
      <div className="absolute top-0 left-0 w-full h-full pointer-events-none">
        {Array(8).fill(0).map((_, row) => (
          Array(8).fill(0).map((_, col) => {
            // For vertical walls, the reference point is the cell to the right of the wall
            const wallRow = row + 1;
            const wallCol = col;
            const wallCoord = `${wallRow},${wallCol}`;
            const wallExists = boardState.vWalls.has(wallCoord);
            const wallIsLegal = isLegalWall(wallRow, wallCol, 'v');
            const showGhost = ghostWall && 
                            ghostWall.type === 'v' && 
                            ghostWall.row === wallRow && 
                            ghostWall.col === wallCol;
            
            return (
              <div 
                key={`vwall-${row}-${col}`}
                className={`
                  absolute pointer-events-auto z-10
                  ${wallIsLegal && canCurrentPlayerMove ? 'cursor-pointer' : ''}
                  ${wallIsLegal && canCurrentPlayerMove && !wallExists && !showGhost ? 'hover:bg-gray-200 hover:bg-opacity-50 rounded-full' : ''}
                `}
                style={{
                  top: `${row * (100 / 9)}%`,
                  left: `${(col + 1) * (100 / 9)}%`,
                  height: `${(100 / 9) * 2}%`,
                  width: '12px',
                  transform: 'translateX(-50%)'
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  if (canCurrentPlayerMove && wallIsLegal) {
                    onWallClick(wallRow, wallCol, 'v');
                  }
                }}
                onMouseEnter={() => {
                  if (canCurrentPlayerMove && wallIsLegal) {
                    setGhostWall({ type: 'v', row: wallRow, col: wallCol });
                  }
                }}
                onMouseLeave={() => setGhostWall(null)}
              >
                {(wallExists || showGhost) && (
                  <div 
                    className={`
                      absolute top-0 left-1/2 h-full w-4 -translate-x-1/2 shadow-md
                      ${wallExists 
                        ? getWallColor(getWallPlayer('v', wallRow, wallCol)) 
                        : getGhostWallColor()}
                      rounded-full
                    `}
                  />
                )}
              </div>
            );
          })
        ))}
      </div>
    </div>
  );
};

export default QuoridorBoard;