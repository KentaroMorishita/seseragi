function explain(nickname: string | undefined): string {
  return nickname === undefined ? "No nickname" : `Present: [${nickname}]`
}
function greet(nickname: string | undefined): string {
  return `Hello, ${nickname ?? "guest"}`
}
console.log(explain("Ada"))
console.log(explain(undefined))
console.log(explain(""))
console.log(greet(undefined))
export {}
