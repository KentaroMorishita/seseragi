function describe(source: string, text: string): string {
  const matches = Array.from(
    text.matchAll(new RegExp(source, "gu")),
    (found) => `${found.index}..${found.index + found[0].length}:${found[0]}`
  )
  return `[${matches.join(", ")}]`
}
console.log(describe("ORD-[0-9]+", "ORD-1 ORD-2"))
console.log(describe("ORD-[0-9]+", "none"))
console.log(describe("", "a👍"))
console.log(describe("a|aa", "xaa"))
console.log(describe("a+", "xaa"))
console.log(describe(".", "👍🏽"))
export {}
