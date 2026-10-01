const original = new Map([
  ["b", 2],
  ["a", 1],
])
const replaced = new Map(original).set("b", 9)
const added = new Map(replaced).set("c", 3)
console.log(JSON.stringify([...replaced.entries()]))
console.log(JSON.stringify([...added.entries()]))
console.log(JSON.stringify([...original.entries()]))
console.log(
  JSON.stringify([...new Map<string, number>().set("first", 1).entries()])
)
export {}
