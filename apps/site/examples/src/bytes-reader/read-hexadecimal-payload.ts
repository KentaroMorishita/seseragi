export {}
function readHex(input: string): string {
  try {
    return `Accepted: ${Uint8Array.fromHex(input).toHex()}`
  } catch {
    return "Rejected"
  }
}
for (const input of ["00FF", "", "0", "00xz", "é0", "00é", "0x00", "00 00"])
  console.log(readHex(input))
