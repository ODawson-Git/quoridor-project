import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

export default defineConfig({
  plugins: [react()],
  // Make sure this exactly matches your repository name
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
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: true,
    assetsDir: 'assets',
    rollupOptions: {
      output: {
        manualChunks: undefined
      }
    }
  }
})