import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@seseragi/core': path.resolve(__dirname, '../src'),
      '@seseragi/core/module-resolver': path.resolve(__dirname, '../src/module-resolver.browser.ts'),
    }
  },
  define: {
    global: 'globalThis',
    'process.env': {}
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
    rollupOptions: {
      external: ['node:fs', 'node:path', 'fs', 'path']
    }
  },
  optimizeDeps: {
    include: ['monaco-editor', '@monaco-editor/react']
  },
  worker: {
    format: 'es'
  }
})