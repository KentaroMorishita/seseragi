// One required number, followed by zero or more numbers.
type NonEmptyScores = readonly [number, ...number[]]

function fromArray(values: readonly number[]): NonEmptyScores | undefined {
  const [first, ...rest] = values
  return first === undefined ? undefined : [first, ...rest]
}

function firstOrNone(values: readonly number[]): string {
  const scores = fromArray(values)
  return scores === undefined ? "no scores" : String(scores[0])
}

const scores: NonEmptyScores = [10, 20, 30]
console.log(scores[0])
console.log(JSON.stringify(scores.slice(1)))
const single: NonEmptyScores = [0]
console.log(JSON.stringify(single.slice(1)))
console.log(firstOrNone([0, 20]))
console.log(firstOrNone([]))

export {}
