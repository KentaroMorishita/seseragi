export {}
type Settings = { name: string; retries: number }
function readSettings(source: string): Settings | null {
  let value: unknown
  try {
    value = JSON.parse(source)
  } catch {
    return null
  }
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return null
  if (Object.keys(value).some((key) => key !== "name" && key !== "retries"))
    return null
  if (!("name" in value) || typeof value.name !== "string") return null
  if (
    !("retries" in value) ||
    typeof value.retries !== "number" ||
    !Number.isSafeInteger(value.retries)
  )
    return null
  return { name: value.name, retries: value.retries }
}
function describe(source: string): string {
  const settings = readSettings(source)
  return settings === null ? "Settings rejected" : `Ready: ${settings.name}`
}
for (const source of [
  '{"name":"Mio","retries":2}',
  '{"name":false,"retries":2}',
  "{",
])
  console.log(describe(source))
