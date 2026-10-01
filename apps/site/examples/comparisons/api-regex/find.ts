function describe(text: string): string {
  const pattern = /(?<code>ORD-[0-9]+)(?:-(?<tag>[A-Z]+))?/u
  const found = pattern.exec(text)
  if (found === null) return "no match"
  const start = found.index
  const end = start + found[0].length
  const tag = found.groups?.tag ?? "absent"
  return `${found[0]}: UTF-16 ${start}..${end}; captures=${found.length - 1}; tag=${tag}`
}
console.log(describe("é ORD-42"))
console.log(describe("é ORD-42-X"))
console.log(describe("no order"))
export {}
