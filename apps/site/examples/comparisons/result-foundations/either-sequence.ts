type Result = { ok: true; count: number } | { ok: false; reason: string }
function describe(results: Result[]): string {
  const seats: number[] = []
  for (const result of results) {
    if (!result.ok) return `Error: ${result.reason}`
    seats.push(result.count)
  }
  return `Seats: [${seats.join(",")}]`
}
const accepted: Result[] = [
  { ok: true, count: 2 },
  { ok: true, count: 1 },
]
const firstFailed: Result[] = [
  { ok: false, reason: "first" },
  { ok: true, count: 1 },
]
const twoFailed: Result[] = [
  { ok: true, count: 2 },
  { ok: false, reason: "first" },
  { ok: false, reason: "second" },
]
const empty: Result[] = []
console.log(describe(accepted))
console.log(describe(firstFailed))
console.log(describe(twoFailed))
console.log(describe(empty))
export {}
