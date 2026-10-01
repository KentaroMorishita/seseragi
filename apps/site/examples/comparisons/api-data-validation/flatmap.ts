type SeatResult = { ok: true; seats: number } | { ok: false; error: string }
function requireSeats(seats: number): SeatResult {
  return seats > 0
    ? { ok: true, seats }
    : { ok: false, error: "seats must be positive" }
}
type QuoteResult = { ok: true; total: number } | { ok: false; error: string }
function quote(seats: number): QuoteResult {
  return seats > 5
    ? { ok: false, error: "too many seats" }
    : { ok: true, total: seats * 20 }
}
function checkedQuote(seats: number): QuoteResult {
  const checked = requireSeats(seats)
  if (!checked.ok) return checked
  return quote(checked.seats)
}
function describe(result: QuoteResult): string {
  return result.ok ? `Total: ${result.total}` : `Cannot book: ${result.error}`
}
console.log(describe(checkedQuote(2)))
console.log(describe(checkedQuote(0)))
console.log(describe(checkedQuote(6)))
export {}
