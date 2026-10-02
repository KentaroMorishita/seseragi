export {}
function read(input: string): string {
  try {
    return `Decoded: ${Uint8Array.fromHex(input).toHex()}`
  } catch {
    return "Invalid hex"
  }
}
for (const input of ["00FF", "0", "00xz", "é0"]) console.log(read(input))
