export {}
function extract(start: number, end: number, content: Uint8Array): string {
  if (
    !Number.isInteger(start) ||
    !Number.isInteger(end) ||
    start < 0 ||
    start > end ||
    end > content.length
  )
    return `Invalid range ${start}..${end} for ${content.length} bytes`
  return `[${Array.from(content.slice(start, end)).join(", ")}]`
}
const content = Uint8Array.of(10, 20, 30, 40)
console.log(extract(1, 3, content))
console.log(extract(2, 2, content))
console.log(extract(3, 1, content))
console.log(extract(-1, 2, content))
console.log(extract(0, 5, content))
console.log(`[${Array.from(content).join(", ")}]`)
