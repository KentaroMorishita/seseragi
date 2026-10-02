export {}
const content = new Uint8Array([0, 15, 255])
console.log(`[${Array.from(content).join(", ")}]`)
console.log(content.length)
