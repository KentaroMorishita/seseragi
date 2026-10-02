export {}
function report(source: string): string {
  const value: unknown = JSON.parse(source)
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return "Labels rejected"
  if (Object.keys(value).some((name) => name !== "first" && name !== "second"))
    return "Labels rejected"
  if (!("first" in value) || typeof value.first !== "string")
    return "Labels rejected"
  if (!("second" in value) || typeof value.second !== "string")
    return "Labels rejected"
  const labels: [string, string][] = [
    ["first", value.first],
    ["second", value.second],
  ]
  return JSON.stringify(labels)
}
for (const source of [
  '{"second":"B","first":"A"}',
  '{"first":"A"}',
  '{"first":false,"extra":"x"}',
])
  console.log(report(source))
