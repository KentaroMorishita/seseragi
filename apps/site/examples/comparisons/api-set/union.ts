const left = new Set([3, 1, 2])
const right = new Set([2, 4, 1])
const empty = new Set<number>()
function combined(left: Set<number>, right: Set<number>): number[] {
  return [...new Set([...left, ...right])]
}
console.log(JSON.stringify(combined(left, right)))
console.log(JSON.stringify(combined(right, left)))
console.log(JSON.stringify(combined(left, empty)))
console.log(JSON.stringify(combined(empty, right)))
console.log(JSON.stringify([...left]))
console.log(JSON.stringify([...right]))
export {}
