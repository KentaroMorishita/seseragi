function describe(nickname: string | undefined): string {
  return nickname === undefined
    ? "Nickname is missing"
    : `Nickname: [${nickname}]`
}
function countLabel(value: number | undefined): string {
  return value === undefined ? "Count is missing" : `Count: ${value}`
}
console.log(describe("Ada"))
console.log(describe(""))
console.log(countLabel(0))
export {}
