export {}
function describe(value: number): string {
  if (!Number.isInteger(value) || value < 0 || value > 255)
    return `Out of range: ${value}`
  return `Byte: ${value}`
}
for (const value of [0, 255, -1, 256]) console.log(describe(value))
