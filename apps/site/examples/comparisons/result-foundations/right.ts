type Result = { ok: true; count: number } | { ok: false; reason: string }
function describe(result: Result): string {
  return result.ok ? `Accepted: ${result.count}` : `Rejected: ${result.reason}`
}
const accepted: Result = { ok: true, count: 2 }
const unchecked: Result = { ok: true, count: 0 }
console.log(describe(accepted))
console.log(describe(unchecked))
export {}
