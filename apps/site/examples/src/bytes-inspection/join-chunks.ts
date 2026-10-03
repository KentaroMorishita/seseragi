export {}
const first = Uint8Array.of(10, 20)
const last = Uint8Array.of(30)
const chunks = [first, new Uint8Array(), last]
const joined = Uint8Array.from(chunks.flatMap((chunk) => Array.from(chunk)))
console.log(`[${Array.from(joined).join(", ")}]`)
const emptyChunks: Uint8Array[] = []
console.log(
  `[${Array.from(Uint8Array.from(emptyChunks.flatMap((chunk) => Array.from(chunk)))).join(", ")}]`
)
console.log(`[${Array.from(first).join(", ")}]`)
console.log(`[${Array.from(last).join(", ")}]`)
