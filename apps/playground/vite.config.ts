import { defineConfig } from "vite"

export default defineConfig({
  build: {
    target: "es2022",
    outDir: "dist",
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("/typescript/")) return "typescript"
          if (id.includes("/node_modules/@codemirror/")) return "editor"
          return undefined
        },
      },
    },
  },
})
