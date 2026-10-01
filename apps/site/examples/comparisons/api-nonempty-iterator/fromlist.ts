type NonEmpty<T> = readonly [T, ...T[]]
function fromArray<T>(values: readonly T[]): NonEmpty<T> | undefined {
  return values.length === 0 ? undefined : [values[0]!, ...values.slice(1)]
}
function firstScore(values: readonly number[]): string {
  const scores = fromArray(values)
  return scores === undefined ? "no score" : `first: ${scores[0]}`
}
console.log(firstScore([0, 20]))
console.log(firstScore([]))
export {}
