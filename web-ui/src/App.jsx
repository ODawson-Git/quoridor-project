// --- File: web-ui/src/App.jsx ---
import React from 'react';
import QuoridorGame from './components/QuoridorGame'; // Import the main game component
import './index.css'; // Import Tailwind styles

function App() {
  return (
    <div className="App items-center min-h-screen bg-gray-100">
      <QuoridorGame />
    </div>
  );
}

export default App;