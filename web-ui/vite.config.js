// --- File: web-ui/vite.config.js ---
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path' // Import the 'resolve' function from 'path'

// Note: If using ES Modules (`type: "module"` in package.json),
// __dirname might not be available directly. Use import.meta.url instead.
// import url from 'url';
// const __filename = url.fileURLToPath(import.meta.url);
// const __dirname = path.dirname(__filename);

export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      // Allow serving files from one level up to access the sibling 'quoridor-wasm/pkg'
      allow: ['..']
    }
  },
  resolve: {
    alias: {
      // Create an alias '@wasm' pointing to the WASM package directory
      // Adjust the relative path if your structure differs slightly
      '@wasm': resolve(__dirname, '../quoridor-wasm/pkg')
    }
  },
  // Optional: Ensure WASM MIME type is handled correctly, although Vite often does this
  // optimizeDeps: {
  //  exclude: ['@wasm/quoridor_wasm.js'] // Might help?
  // }
})