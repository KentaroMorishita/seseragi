export {}
function greet(name: string): string {
  return `Hello, ${name}`
}
function report(source: string): string {
  const value: unknown = JSON.parse(source)
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return "Name rejected"
  if (!("name" in value) || typeof value.name !== "string")
    return "Name rejected"
  return greet(value.name)
}
for (const source of ['{"name":"Mio"}', '{"name":""}', '{"name":false}'])
  console.log(report(source))
