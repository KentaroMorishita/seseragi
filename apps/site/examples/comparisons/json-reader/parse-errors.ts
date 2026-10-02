export {}
function inspect(source: string): string {
  try {
    return `Valid JSON: ${JSON.stringify(JSON.parse(source))}`
  } catch {
    return "Invalid JSON syntax"
  }
}
console.log(inspect('{"name":"Mio"}'))
console.log(inspect('{"name":}'))
