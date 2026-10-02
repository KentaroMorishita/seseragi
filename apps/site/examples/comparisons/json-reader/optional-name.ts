export {}
function report(source: string): string {
  const value: unknown = JSON.parse(source)
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return "Name must be text"
  if (!Object.hasOwn(value, "name")) return "Name is missing"
  if (!("name" in value) || typeof value.name !== "string")
    return "Name must be text"
  return `Present: ${value.name}`
}
for (const source of ["{}", '{"name":""}', '{"name":"Mio"}', '{"name":null}'])
  console.log(report(source))
