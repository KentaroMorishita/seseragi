export {}
function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}
function report(source: string): string {
  const value: unknown = JSON.parse(source)
  if (!isObject(value)) return "$ | expected object"
  if (!Object.hasOwn(value, "people")) return "$ | missing people"
  if (!Array.isArray(value.people)) return "$.people | expected array"
  const names: string[] = []
  for (let index = 0; index < value.people.length; index++) {
    const person: unknown = value.people[index]
    if (!isObject(person)) return `$.people[${index}] | expected object`
    if (!Object.hasOwn(person, "name"))
      return `$.people[${index}] | missing name`
    if (typeof person.name !== "string")
      return `$.people[${index}].name | expected JsonString`
    names.push(person.name)
  }
  return JSON.stringify(names)
}
for (const source of [
  '{"people":[{"name":"Mio"},{"name":"Aki"}]}',
  '{"people":[]}',
  '{"people":[{"name":"Mio"},{"name":false},{"name":null}]}',
  '{"people":[{"name":"Mio"},{}]}',
  '{"people":[false]}',
  '{"people":false}',
  "{}",
])
  console.log(report(source))
