import process from "node:process"

const mode = process.env.SESERAGI_DOCS_MODE
console.log(mode === undefined ? "Mode: preview (default)" : `Mode: [${mode}]`)
