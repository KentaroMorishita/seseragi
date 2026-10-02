export {}
function readBase64Url(input: string): string {
  try {
    const content = Uint8Array.fromBase64(input, { alphabet: "base64url" })
    const canonical = content.toBase64({
      alphabet: "base64url",
      omitPadding: true,
    })
    return canonical === input ? `Accepted: ${content.toHex()}` : "Rejected"
  } catch {
    return "Rejected"
  }
}
console.log(
  new Uint8Array([251, 255]).toBase64({
    alphabet: "base64url",
    omitPadding: true,
  })
)
for (const input of ["-_8", "", "AA=", "+A", "AB", "A"])
  console.log(readBase64Url(input))
