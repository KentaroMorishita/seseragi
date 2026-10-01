const left = new Map([
  ["b", "L-b"],
  ["a", "L-a"],
])
const right = new Map([
  ["a", "R-a"],
  ["c", "R-c"],
])
function merged(
  left: Map<string, string>,
  right: Map<string, string>
): Map<string, string> {
  const result = new Map(left)
  for (const [key, incoming] of right) {
    result.set(
      key,
      result.has(key) ? result.get(key)! + "/" + incoming : incoming
    )
  }
  return result
}
console.log(JSON.stringify([...merged(left, right).entries()]))
console.log(JSON.stringify([...merged(right, left).entries()]))
console.log(JSON.stringify([...merged(left, new Map()).entries()]))
console.log(JSON.stringify([...merged(new Map(), right).entries()]))
console.log(JSON.stringify([...left.entries()]))
console.log(JSON.stringify([...right.entries()]))
export {}
