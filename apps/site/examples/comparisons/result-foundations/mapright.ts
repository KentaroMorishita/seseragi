type Result = { ok: true; count: number } | { ok: false; reason: string }
function seatLabel(count: number): string {
  return `seats=${count}`
}
function describe(result: Result): string {
  return result.ok
    ? `OK: ${seatLabel(result.count)}`
    : `Error: ${result.reason}`
}
const accepted: Result = { ok: true, count: 2 }
const rejected: Result = { ok: false, reason: "seats must be positive" }
console.log(describe(accepted))
console.log(describe(rejected))
export {}
