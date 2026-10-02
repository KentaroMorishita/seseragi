export {}
function describe(content: Uint8Array): string {
  return content.length === 0
    ? "Empty"
    : `Bytes: [${Array.from(content).join(", ")}]`
}
console.log(describe(new Uint8Array()))
const value = 65
if (!Number.isInteger(value) || value < 0 || value > 255)
  console.log(`Out of range: ${value}`)
else console.log(describe(Uint8Array.of(value)))
