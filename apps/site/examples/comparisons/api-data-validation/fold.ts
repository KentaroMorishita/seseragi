type SeatResult = { ok: true; seats: number } | { ok: false; error: string }
function requireSeats(seats: number): SeatResult {
  return seats > 0
    ? { ok: true, seats }
    : { ok: false, error: "seats must be positive" }
}
function failureMessage(error: string): string {
  return `Cannot book: ${error}`
}
function successMessage(seats: number): string {
  return `Seats: ${seats}`
}
function describe(result: SeatResult): string {
  return result.ok ? successMessage(result.seats) : failureMessage(result.error)
}
console.log(describe(requireSeats(2)))
console.log(describe(requireSeats(0)))
export {}
