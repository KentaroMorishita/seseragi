export {}
function report(source: string): string {
  let value: unknown
  try {
    value = JSON.parse(source)
  } catch {
    return "Invalid JSON text"
  }
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return "Name must be text"
  if (!Object.hasOwn(value, "name")) return "Name is missing"
  if (!("name" in value) || typeof value.name !== "string")
    return "Name must be text"
  return `Name: ${value.name}`
}
for (const source of ['{"name":"Mio","active":true}', "{}", '{"name":false}'])
  console.log(report(source))
