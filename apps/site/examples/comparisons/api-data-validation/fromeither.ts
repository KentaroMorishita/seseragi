type CheckedName =
  | { ok: true; value: string }
  | { ok: false; errors: [string, ...string[]] }
function describe(result: CheckedName): string {
  return result.ok
    ? `OK: ${result.value}`
    : `Errors: ${result.errors.join("; ")}`
}
type NameResult = { ok: true; value: string } | { ok: false; error: string }
function requireName(name: string): NameResult {
  return name === ""
    ? { ok: false, error: "name is required" }
    : { ok: true, value: name }
}
function fromResult(result: NameResult): CheckedName {
  return result.ok ? result : { ok: false, errors: [result.error] }
}
console.log(describe(fromResult(requireName("Ada"))))
console.log(describe(fromResult(requireName(""))))
export {}
