import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@seseragi/core': path.resolve(__dirname, '../src'),
    }
  },
  define: {
    global: 'globalThis',
    'process.env': {}
  },
  build: {
    target: 'esnext',
    outDir: 'dist'
  },
  optimizeDeps: {
    include: ['monaco-editor', '@monaco-editor/react']
  },
  worker: {
    format: 'es'
  }
})