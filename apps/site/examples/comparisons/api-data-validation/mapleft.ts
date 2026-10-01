type SeatResult = { ok: true; seats: number } | { ok: false; error: string }
function requireSeats(seats: number): SeatResult {
  return seats > 0
    ? { ok: true, seats }
    : { ok: false, error: "seats must be positive" }
}
function addField(error: string): string {
  return `seats field: ${error}`
}
function withField(result: SeatResult): SeatResult {
  return result.ok ? result : { ok: false, error: addField(result.error) }
}
function describe(result: SeatResult): string {
  return result.ok ? `Seats: ${result.seats}` : `Cannot book: ${result.error}`
}
console.log(describe(withField(requireSeats(0))))
console.log(describe(withField(requireSeats(2))))
export {}
