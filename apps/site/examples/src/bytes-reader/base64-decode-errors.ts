export {}
function read(input: string): string {
  try {
    const content = Uint8Array.fromBase64(input, {
      lastChunkHandling: "strict",
    })
    return content.toBase64() === input
      ? `Decoded: ${content.toHex()}`
      : "Invalid Base64"
  } catch {
    return "Invalid Base64"
  }
}
for (const input of ["Zg==", "Zg", "AA A", "AA=A", "AB=="])
  console.log(read(input))
