import { Buffer } from "node:buffer"

const content = new TextEncoder().encode("café")
console.log(content.length)
console.log(Buffer.from(content).toString("hex"))
console.log(Buffer.from(content).toString("base64"))
console.log(Buffer.from(content).toString("base64url"))
console.log(
  new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(content)
)
