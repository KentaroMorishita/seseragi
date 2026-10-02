export {}
function normalize(source: string): string {
  try {
    return JSON.stringify(JSON.parse(source))
  } catch {
    return "Invalid JSON text"
  }
}
for (const source of [
  ' {"name":"Mio","retries":2} ',
  '[true, null, "ready"]',
  "{",
])
  console.log(normalize(source))
