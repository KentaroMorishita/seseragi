function describe(nickname: string | undefined): string {
  return nickname === undefined
    ? "Nickname is missing"
    : `Nickname: [${nickname}]`
}
const nickname: string | undefined = undefined
console.log(describe(nickname))
console.log(describe("Ada"))
export {}
