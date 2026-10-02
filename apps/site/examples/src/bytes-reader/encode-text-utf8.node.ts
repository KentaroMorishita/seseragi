import { Buffer } from "node:buffer"

for (const label of ["A", "café", "🌊", ""]) {
  const content = new TextEncoder().encode(label)
  console.log(`${Buffer.from(content).toString("hex")}:${content.length}`)
}
