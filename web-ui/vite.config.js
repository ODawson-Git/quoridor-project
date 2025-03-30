import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

export default defineConfig({
  plugins: [react()],
  // Add base path - use the name of your repository
  base: '/quoridor-game/',
  server: {
    fs: {
      allow: ['..']
    }
  },
  resolve: {
    alias: {
      '@wasm': resolve(__dirname, '../quoridor-wasm/pkg')
    }
  },
  // Ensure Vite correctly handles WASM files
  optimizeDeps: {
    exclude: ['@wasm/quoridor_wasm.js']
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: true
  }
})