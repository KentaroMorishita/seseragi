import { Buffer } from "node:buffer"

function readBase64Url(input: string): string {
  try {
    const content = Buffer.from(input, "base64url")
    const canonical = Buffer.from(content).toString("base64url")
    return canonical === input
      ? `Accepted: ${Buffer.from(content).toString("hex")}`
      : "Rejected"
  } catch {
    return "Rejected"
  }
}
console.log(Buffer.from([251, 255]).toString("base64url"))
for (const input of ["-_8", "", "AA=", "+A", "AB", "A"])
  console.log(readBase64Url(input))
