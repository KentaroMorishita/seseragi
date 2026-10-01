const required = new Set([2, 1])
const allowed = new Set([3, 1, 2])
const empty = new Set<number>()
function isSubset(values: Set<number>, superset: Set<number>): boolean {
  return [...values].every((value) => superset.has(value))
}
console.log(isSubset(required, allowed))
console.log(isSubset(allowed, required))
console.log(isSubset(empty, allowed))
console.log(isSubset(required, empty))
console.log(isSubset(empty, empty))
export {}
