type Result = { ok: true; count: number } | { ok: false; reason: string }
function describe(result: Result): string {
  return result.ok ? `Accepted: ${result.count}` : `Rejected: ${result.reason}`
}
function requireSeats(count: number): Result {
  return count > 0
    ? { ok: true, count }
    : { ok: false, reason: "seats must be positive" }
}
console.log(describe(requireSeats(2)))
console.log(describe(requireSeats(0)))
export {}
