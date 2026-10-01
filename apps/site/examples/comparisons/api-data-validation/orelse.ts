function describe(nickname: string | undefined): string {
  return nickname === undefined ? "No nickname" : `Present: [${nickname}]`
}
function choose(
  fallback: string | undefined,
  primary: string | undefined
): string | undefined {
  return primary ?? fallback
}
console.log(describe(choose("team", "Ada")))
console.log(describe(choose("team", undefined)))
console.log(describe(choose(undefined, undefined)))
console.log(describe(choose("team", "")))
export {}
