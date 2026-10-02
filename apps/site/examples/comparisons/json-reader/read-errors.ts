export {}
function describe(source: string): string {
  let value: unknown
  try {
    value = JSON.parse(source)
  } catch {
    return "JSON text rejected"
  }
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return "Settings shape rejected"
  if (Object.keys(value).some((key) => key !== "name" && key !== "retries"))
    return "Settings shape rejected"
  if (!("name" in value) || typeof value.name !== "string")
    return "Settings shape rejected"
  if (
    !("retries" in value) ||
    typeof value.retries !== "number" ||
    !Number.isSafeInteger(value.retries)
  )
    return "Settings shape rejected"
  return `Ready: ${value.name}`
}
for (const source of [
  '{"name":"Mio","retries":2}',
  '{"name":false,"retries":2}',
  "{",
])
  console.log(describe(source))
