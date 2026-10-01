export function describe(value: string | undefined): string {
  return value === undefined ? "missing" : `found: [${value}]`
}
const names = new Map<string, string>([
  ["name", "Aki"],
  ["empty", ""],
])
console.log(describe(names.get("name")))
console.log(describe(names.get("empty")))
console.log(describe(names.get("unknown")))
console.log(describe(new Map<string, string>().get("name")))
