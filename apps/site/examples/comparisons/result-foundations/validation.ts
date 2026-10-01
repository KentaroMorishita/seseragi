type Result =
  | { ok: true; name: string }
  | { ok: false; errors: [string, ...string[]] }
function describe(result: Result): string {
  return result.ok
    ? `Accepted: [${result.name}]`
    : `Errors: ${result.errors.join("; ")}`
}
const accepted: Result = { ok: true, name: "Ada" }
const rejected: Result = {
  ok: false,
  errors: ["name is required", "seats must be positive"],
}
console.log(describe(accepted))
console.log(describe(rejected))
export {}
