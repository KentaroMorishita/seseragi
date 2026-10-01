const original = new Set([3, 2, 5, 4])
function parity(values: Set<number>): Set<number> {
  return new Set([...values].map((value) => value % 2))
}
console.log(JSON.stringify([...parity(original)]))
console.log(JSON.stringify([...original]))
console.log(JSON.stringify([...parity(new Set())]))
export {}
