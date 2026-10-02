type Result = { ok: true; value: number } | { ok: false; reason: string }
function describe(result: Result): string {
  return result.ok ? `Ready: ${result.value}` : `Rejected: ${result.reason}`
}
const accepted = (): Result => ({ ok: true, value: 42 })
const rejected = (): Result => ({ ok: false, reason: "not approved" })
console.log(describe(accepted()))
console.log(describe(rejected()))
console.log("Caller continues")
export {}
