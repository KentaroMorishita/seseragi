import { Buffer } from "node:buffer"

function read(input: string): string {
  const content = Buffer.from(input, "base64")
  return content.toString("base64") === input
    ? `Decoded: ${content.toString("hex")}`
    : "Invalid Base64"
}
for (const input of ["Zg==", "Zg", "AA A", "AA=A", "AB=="])
  console.log(read(input))
