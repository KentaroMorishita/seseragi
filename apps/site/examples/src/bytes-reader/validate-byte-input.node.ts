export {}
function describe(values: number[]): string {
  const invalid = values.find(
    (value) => !Number.isInteger(value) || value < 0 || value > 255
  )
  if (invalid !== undefined) return `Out of range: ${invalid}`
  return `[${Array.from(Uint8Array.from(values)).join(", ")}]`
}
for (const values of [[0, 127, 255], [1, 256, -1], [-1], []])
  console.log(describe(values))
