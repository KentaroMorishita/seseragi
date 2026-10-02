import { Buffer } from "node:buffer"

function readBase64(input: string): string {
  try {
    const content = Buffer.from(input, "base64")
    return Buffer.from(content).toString("base64") === input
      ? `Accepted: ${Buffer.from(content).toString("hex")}`
      : "Rejected"
  } catch {
    return "Rejected"
  }
}
for (const input of [
  "Zg==",
  "",
  "Zg",
  "AA A",
  "AA=A",
  "AB==",
  "Z g==",
  "-_8",
  "Zm9v",
])
  console.log(readBase64(input))
