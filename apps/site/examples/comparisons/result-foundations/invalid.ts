type Result =
  | { ok: true; name: string }
  | { ok: false; errors: [string, ...string[]] }
function describe(result: Result): string {
  return result.ok
    ? `Accepted: [${result.name}]`
    : `Errors: ${result.errors.join("; ")}`
}
const one: Result = { ok: false, errors: ["name is required"] }
const two: Result = {
  ok: false,
  errors: ["name is required", "seats must be positive"],
}
console.log(describe(one))
console.log(describe(two))
console.log("Caller continues")
export {}
