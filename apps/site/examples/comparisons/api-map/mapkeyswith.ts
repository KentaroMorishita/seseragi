const original = new Map([
  ["first", "A"],
  ["keep", "K"],
  ["second", "B"],
  ["third", "C"],
])
function normalized(source: Map<string, string>): Map<string, string> {
  const result = new Map<string, string>()
  for (const [key, incoming] of source) {
    const target = key === "keep" ? "keep" : "same"
    const value = result.has(target)
      ? result.get(target)! + "/" + incoming
      : incoming
    result.set(target, value)
  }
  return result
}
console.log(JSON.stringify([...normalized(original).entries()]))
console.log(JSON.stringify([...original.entries()]))
console.log(JSON.stringify([...normalized(new Map()).entries()]))
export {}
