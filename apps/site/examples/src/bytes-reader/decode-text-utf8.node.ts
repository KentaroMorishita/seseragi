import { Buffer } from "node:buffer"

function readHexBytes(input: string): Uint8Array {
  if (input.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(input))
    throw new SyntaxError("Invalid hex")
  return Buffer.from(input, "hex")
}
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })
for (const input of ["636166c3a9", "f09f8c8a", "", "ff"]) {
  const content = readHexBytes(input)
  try {
    console.log(`Text: ${decoder.decode(content)}`)
  } catch {
    console.log("Invalid UTF-8")
  }
}
