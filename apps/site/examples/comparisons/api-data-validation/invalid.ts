type CheckedName =
  | { ok: true; value: string }
  | { ok: false; errors: [string, ...string[]] }
function describe(result: CheckedName): string {
  return result.ok
    ? `OK: ${result.value}`
    : `Errors: ${result.errors.join("; ")}`
}
function validateName(name: string): CheckedName {
  return name === ""
    ? { ok: false, errors: ["name is required"] }
    : { ok: true, value: name }
}
console.log(describe(validateName("")))
console.log(describe(validateName("Ada")))
export {}
