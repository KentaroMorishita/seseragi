type Result =
  | { ok: true; name: string }
  | { ok: false; errors: [string, ...string[]] }
function describe(result: Result): string {
  return result.ok
    ? `Accepted: [${result.name}]`
    : `Errors: ${result.errors.join("; ")}`
}
const accepted: Result = { ok: true, name: "Ada" }
const unchecked: Result = { ok: true, name: "" }
console.log(describe(accepted))
console.log(describe(unchecked))
export {}
