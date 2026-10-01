type SeatResult = { ok: true; seats: number } | { ok: false; error: string }
function requireSeats(seats: number): SeatResult {
  return seats > 0
    ? { ok: true, seats }
    : { ok: false, error: "seats must be positive" }
}
function describe(result: SeatResult): string {
  return result.ok ? `Seats: ${result.seats}` : `Cannot book: ${result.error}`
}
console.log(describe(requireSeats(2)))
console.log(describe(requireSeats(0)))
export {}
