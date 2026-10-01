function describe(results: (number | undefined)[]): string {
  if (results.some((seat) => seat === undefined)) return "Missing seat"
  return `Seats: [${results.join(",")}]`
}
const present = [2, 1]
const firstMissing = [undefined, 1]
const middleMissing = [2, undefined, 1]
const empty: (number | undefined)[] = []
console.log(describe(present))
console.log(describe(firstMissing))
console.log(describe(middleMissing))
console.log(describe(empty))
export {}
