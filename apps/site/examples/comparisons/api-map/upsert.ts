const original = new Map([
  ["empty", ""],
  ["name", "Aki"],
])
function updated(
  source: Map<string, string>,
  key: string
): Map<string, string> {
  const value = source.has(key) ? "existing:" + source.get(key)! : "new"
  return new Map(source).set(key, value)
}
console.log(JSON.stringify([...updated(original, "empty").entries()]))
console.log(JSON.stringify([...updated(original, "name").entries()]))
console.log(JSON.stringify([...updated(original, "missing").entries()]))
console.log(JSON.stringify([...original.entries()]))
export {}
