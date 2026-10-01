type CheckedName =
  | { ok: true; value: string }
  | { ok: false; errors: [string, ...string[]] }
function describe(result: CheckedName): string {
  return result.ok
    ? `OK: ${result.value}`
    : `Errors: ${result.errors.join("; ")}`
}
const errors: [string, ...string[]] = [
  "name is required",
  "seats must be positive",
]
const checked: CheckedName = { ok: false, errors }
console.log(describe(checked))
export {}
