type NonEmpty<T> = readonly [T, ...T[]]
function fromArray<T>(values: readonly T[]): NonEmpty<T> | undefined {
  return values.length === 0 ? undefined : [values[0]!, ...values.slice(1)]
}
function total(values: readonly number[]): string {
  const scores = fromArray(values)
  return scores === undefined
    ? "no scores"
    : String(scores.reduce((sum, score) => sum + score))
}
console.log(total([10, 20, 30]))
console.log(total([]))
export {}
