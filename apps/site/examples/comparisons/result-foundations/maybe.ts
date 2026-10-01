function describe(nickname: string | undefined): string {
  return nickname === undefined
    ? "Nickname is missing"
    : `Nickname: [${nickname}]`
}
const present: string | undefined = "Ada"
const missing: string | undefined = undefined
console.log(describe(present))
console.log(describe(missing))
console.log(describe(""))
export {}
