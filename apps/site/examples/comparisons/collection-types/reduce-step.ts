// The ordinary loop already expresses this task directly.
let total = 0
for (const score of [10, 20, -1, 100]) {
  if (score < 0) break
  total += score
}
console.log(total)

// A callback can instead return one of two named decisions.
type ReduceStep =
  | { kind: "next"; value: number }
  | { kind: "done"; value: number }

function step(total: number, score: number): ReduceStep {
  return score < 0
    ? { kind: "done", value: total }
    : { kind: "next", value: total + score }
}

function reduceUntil(
  seed: number,
  decide: (total: number, score: number) => ReduceStep,
  scores: readonly number[]
): number {
  let total = seed
  for (const score of scores) {
    const decision = decide(total, score)
    total = decision.value
    if (decision.kind === "done") return total
  }
  return total
}

console.log(reduceUntil(0, step, [10, 20, -1, 100]))
console.log(reduceUntil(0, step, [10, 20]))
console.log(reduceUntil(0, step, []))
const decision: ReduceStep = { kind: "done", value: 99 }
console.log(decision.value)
console.log("still running")

export {}
