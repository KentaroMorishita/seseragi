type SeatResult = { ok: true; seats: number } | { ok: false; error: string }
function requireSeats(seats: number): SeatResult {
  return seats > 0
    ? { ok: true, seats }
    : { ok: false, error: `seats must be positive: ${seats}` }
}
function describe(inputs: number[]): string {
  const results = inputs.map(requireSeats)
  const seats: number[] = []
  for (const result of results) {
    if (!result.ok) return `Error: ${result.error}`
    seats.push(result.seats)
  }
  return `Seats: [${seats.join(",")}]`
}
console.log(describe([2, 1]))
console.log(describe([0, 2]))
console.log(describe([2, 0, -1]))
console.log(describe([]))
export {}
