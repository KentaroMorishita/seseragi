type Maybe<T> = { kind: "just"; value: T } | { kind: "nothing" }
type StateRead = (state: number) => Maybe<readonly [number, number]>
const readState: StateRead = (state) => ({
  kind: "just",
  value: [state, state],
})
function describe(result: Maybe<readonly [number, number]>): string {
  if (result.kind === "nothing") return "no result"
  const [value, finalState] = result.value
  return `value=${value}, state=${finalState}`
}
console.log(describe(readState(7)))
console.log(describe(readState(0)))
export {}
