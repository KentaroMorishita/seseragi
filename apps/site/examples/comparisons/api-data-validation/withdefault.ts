function greet(nickname: string | undefined): string {
  return `Hello, [${nickname ?? "guest"}]`
}
console.log(greet("Ada"))
console.log(greet(undefined))
console.log(greet(""))
const count: number | undefined = 0
console.log(count ?? 9)
export {}
