type SizeError = { kind: "non-positive-size"; size: number }
type Batches =
  | { kind: "failure"; problem: SizeError }
  | { kind: "success"; groups: number[][] }

// These examples use integer sizes, like the Seseragi inputs.
function chunksOf(size: number, values: readonly number[]): Batches {
  if (size <= 0) {
    return { kind: "failure", problem: { kind: "non-positive-size", size } }
  }
  const groups: number[][] = []
  for (let start = 0; start < values.length; start += size) {
    groups.push(values.slice(start, start + size))
  }
  return { kind: "success", groups }
}

function describe(problem: SizeError): string {
  return `size must be positive: ${problem.size}`
}

function batches(size: number, values: readonly number[]): string {
  const result = chunksOf(size, values)
  return result.kind === "failure"
    ? describe(result.problem)
    : JSON.stringify(result.groups)
}

console.log(batches(2, [10, 20, 30]))
console.log(batches(2, []))
console.log(batches(0, []))
console.log(batches(-2, [10]))
const unchecked: SizeError = { kind: "non-positive-size", size: 2 }
console.log(JSON.stringify(unchecked))

export {}
