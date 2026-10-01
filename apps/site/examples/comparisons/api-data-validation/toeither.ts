type Checked =
  | { valid: true; value: string }
  | { valid: false; errors: [string, ...string[]] }
type Result =
  | { ok: true; value: string }
  | { ok: false; error: [string, ...string[]] }
function toResult(checked: Checked): Result {
  return checked.valid
    ? { ok: true, value: checked.value }
    : { ok: false, error: checked.errors }
}
function describe(result: Result): string {
  return result.ok
    ? `OK: ${result.value}`
    : `Errors: ${result.error.join("; ")}`
}
const accepted: Checked = { valid: true, value: "Ada: 2" }
const rejected: Checked = {
  valid: false,
  errors: ["name is required", "seats must be positive"],
}
console.log(describe(toResult(accepted)))
console.log(describe(toResult(rejected)))
export {}
