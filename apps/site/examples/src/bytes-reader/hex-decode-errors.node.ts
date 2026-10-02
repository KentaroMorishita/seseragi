import { Buffer } from "node:buffer"

function read(input: string): string {
  if (input.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(input))
    return "Invalid hex"
  return `Decoded: ${Buffer.from(input, "hex").toString("hex")}`
}
for (const input of ["00FF", "0", "00xz", "é0"]) console.log(read(input))
