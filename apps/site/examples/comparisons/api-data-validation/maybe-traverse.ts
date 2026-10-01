function findSeat(code: string): number | undefined {
  return code === "A" ? 1 : code === "B" ? 2 : undefined
}
function describe(codes: string[]): string {
  const results = codes.map(findSeat)
  if (results.some((seat) => seat === undefined)) return "Missing seat"
  return `Seats: [${results.join(",")}]`
}
console.log(describe(["B", "A"]))
console.log(describe(["unknown", "A"]))
console.log(describe(["B", "unknown", "A"]))
console.log(describe([]))
export {}
