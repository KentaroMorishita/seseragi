const original = new Map([
  ["b", 2],
  ["a", 1],
])
const removed = new Map(original)
removed.delete("b")
const missing = new Map(removed)
missing.delete("missing")
console.log(JSON.stringify([...removed.entries()]))
console.log(JSON.stringify([...missing.entries()]))
console.log(JSON.stringify([...new Map(removed).set("b", 8).entries()]))
console.log(JSON.stringify([...original.entries()]))
const empty = new Map<string, number>()
empty.delete("b")
console.log(JSON.stringify([...empty.entries()]))
export {}
