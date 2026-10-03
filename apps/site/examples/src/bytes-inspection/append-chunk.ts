export {}
const content = Uint8Array.of(10, 20)
const suffix = Uint8Array.of(30, 40)
const joined = new Uint8Array([...content, ...suffix])
console.log(`[${Array.from(joined).join(", ")}]`)
console.log(
  `[${Array.from(new Uint8Array([...content, ...suffix])).join(", ")}]`
)
console.log(
  `[${Array.from(new Uint8Array([...suffix, ...content])).join(", ")}]`
)
console.log(`[${Array.from(content).join(", ")}]`)
console.log(`[${Array.from(suffix).join(", ")}]`)
