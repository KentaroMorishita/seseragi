import { Buffer } from "node:buffer"

function readHexBytes(input: string): Uint8Array {
  if (input.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(input))
    throw new SyntaxError("Invalid hex")
  return Buffer.from(input, "hex")
}
function readHex(input: string): string {
  try {
    return `Accepted: ${Buffer.from(readHexBytes(input)).toString("hex")}`
  } catch {
    return "Rejected"
  }
}
for (const input of ["00FF", "", "0", "00xz", "é0", "00é", "0x00", "00 00"])
  console.log(readHex(input))
