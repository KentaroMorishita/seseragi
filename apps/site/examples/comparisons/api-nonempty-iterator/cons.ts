type NonEmpty<T> = readonly [T, ...T[]]
const remaining: readonly number[] = [20, 30]
const scores: NonEmpty<number> = [10, ...remaining]
console.log(JSON.stringify(scores))
console.log(JSON.stringify(remaining))
console.log(JSON.stringify([10, ...[]]))
export {}
