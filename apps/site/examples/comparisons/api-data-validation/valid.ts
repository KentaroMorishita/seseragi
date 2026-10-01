type CheckedName =
  | { ok: true; value: string }
  | { ok: false; errors: [string, ...string[]] }
function describe(result: CheckedName): string {
  return result.ok
    ? `OK: ${result.value}`
    : `Errors: ${result.errors.join("; ")}`
}
function acceptName(name: string): CheckedName {
  return { ok: true, value: name }
}
console.log(describe(acceptName("Ada")))
console.log(describe(acceptName("")))
export {}
