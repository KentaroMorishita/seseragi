function tail<T>(values: readonly T[]): readonly T[] | undefined {
  return values.length === 0 ? undefined : values.slice(1)
}
function describe(rest: readonly string[] | undefined): string {
  return rest === undefined ? "missing" : `present: ${JSON.stringify(rest)}`
}
console.log(describe(tail<string>([])))
console.log(describe(tail(["Read"])))
console.log(describe(tail(["Read", "Review", "Ship"])))

export {}
