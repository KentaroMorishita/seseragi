export {}
type Result<T> = { ok: true; value: T } | { ok: false; reason: string }
function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}
type Settings = { name: string; retries: number }
function encodeSettings(value: Settings): { name: string; retries: number } {
  return { name: value.name, retries: value.retries }
}
function decodeSettings(value: unknown): Result<Settings> {
  if (!isObject(value)) return { ok: false, reason: "$ | expected object" }
  for (const key of Object.keys(value))
    if (key !== "name" && key !== "retries")
      return { ok: false, reason: `$ | unknown ${key}` }
  if (!Object.hasOwn(value, "name"))
    return { ok: false, reason: "$ | missing name" }
  if (typeof value.name !== "string")
    return { ok: false, reason: "$.name | expected JsonString" }
  if (!Object.hasOwn(value, "retries"))
    return { ok: false, reason: "$ | missing retries" }
  if (typeof value.retries !== "number" || !Number.isSafeInteger(value.retries))
    return { ok: false, reason: "invalid retries" }
  return { ok: true, value: { name: value.name, retries: value.retries } }
}
console.log(JSON.stringify(encodeSettings({ name: "Mio", retries: 2 })))
const parsed: unknown = JSON.parse('{"name":"Mio","retries":2}')
const result = decodeSettings(parsed)
console.log(result.ok ? `Ready: ${result.value.name}` : result.reason)
