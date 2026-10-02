export {}
function readBase64(input: string): string {
  try {
    const content = Uint8Array.fromBase64(input, {
      lastChunkHandling: "strict",
    })
    return content.toBase64() === input
      ? `Accepted: ${content.toHex()}`
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
