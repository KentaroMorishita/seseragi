type NonEmpty<T> = readonly [T, ...T[]]
const scores: NonEmpty<number> = [10]
console.log(scores[0])
console.log(JSON.stringify(scores))
console.log(JSON.stringify(scores.slice(1)))
export {}
