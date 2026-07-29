import { fileURLToPath } from "node:url"
import { defineConfig } from "vite"

const appRoot = fileURLToPath(new URL(".", import.meta.url))

export default defineConfig({
  build: {
    target: "es2022",
    outDir: "dist",
    rollupOptions: {
      input: {
        playground: `${appRoot}index.html`,
        tour: `${appRoot}tour/index.html`,
      },
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
