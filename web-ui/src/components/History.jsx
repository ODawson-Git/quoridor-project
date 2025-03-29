// --- File: web-ui/src/components/History.jsx ---
import React from 'react';

const Player = { PLAYER1: 'player1', PLAYER2: 'player2' }; // Define locally if not imported

const History = ({ moveHistory }) => {

  // Group moves by turn number
  const turns = [];
  let currentTurn = { number: 1, player1Move: null, player2Move: null };

  moveHistory.forEach((move) => {
    if (move.player === Player.PLAYER1) {
      // Start a new turn if player1 already moved in current turn
      if (currentTurn.player1Move !== null) {
        turns.push({ ...currentTurn }); // Push copy of completed turn
        currentTurn = { number: currentTurn.number + 1, player1Move: move, player2Move: null };
      } else {
         currentTurn.player1Move = move;
      }
    } else { // Player 2's move
      currentTurn.player2Move = move;
      turns.push({ ...currentTurn }); // Push completed turn
      // Reset for next turn (P1 starts)
      currentTurn = { number: currentTurn.number + 1, player1Move: null, player2Move: null };
    }
  });

  // Add the last potentially incomplete turn
  if (currentTurn.player1Move !== null || currentTurn.player2Move !== null) {
    turns.push(currentTurn);
  }

  return (
    <div className="w-full md:w-64 p-4 bg-gray-50 rounded-br-lg border-l border-gray-200">
      <h3 className="font-bold text-gray-800 mb-2 pb-2 border-b border-gray-300">Move History</h3>
      <div className="border border-gray-300 rounded-lg p-2 h-96 overflow-y-auto bg-white text-sm">
        {moveHistory.length === 0 ? (
          <p className="text-gray-500 italic text-center mt-4">No moves yet.</p>
        ) : (
          <>
            {/* Header */}
            <div className="flex border-b-2 border-gray-300 py-1 font-semibold sticky top-0 bg-white">
              <div className="w-1/12 text-center">#</div>
              <div className="w-5/12 text-blue-600 pl-1">Player 1</div>
              <div className="w-5/12 text-red-600 pl-1">Player 2</div>
            </div>
            {/* Moves */}
            {turns.map((turn) => (
              <div key={`turn-${turn.number}`} className="flex border-b border-gray-200 py-1">
                <div className="w-1/12 text-gray-600 text-center">{turn.number}.</div>
                <div className="w-5/12 text-blue-600 pl-1">
                  {turn.player1Move ? (
                    <span title={`Type: ${turn.player1Move.type}`}>
                      {turn.player1Move.move}
                      {turn.player1Move.isWinningMove && <span className="font-bold text-green-600">*</span>}
                    </span>
                  ) : ''}
                </div>
                <div className="w-5/12 text-red-600 pl-1">
                  {turn.player2Move ? (
                     <span title={`Type: ${turn.player2Move.type}`}>
                      {turn.player2Move.move}
                      {turn.player2Move.isWinningMove && <span className="font-bold text-green-600">*</span>}
                     </span>
                  ) : (turn.player1Move ? '...' : '') /* Show '...' if P1 moved but P2 hasn't */ }
                </div>
              </div>
            ))}
          </>
        )}
      </div>
    </div>
  );
};

export default History;