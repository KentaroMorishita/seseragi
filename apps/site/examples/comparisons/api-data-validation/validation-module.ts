function bookingLabel(name: string, seats: number): string {
  return `${name}: ${seats}`
}
function validateBooking(name: string, seats: number): string {
  const errors: string[] = []
  if (name === "") errors.push("name is required")
  if (seats <= 0) errors.push("seats must be positive")
  return errors.length > 0
    ? `Errors: ${errors.join("; ")}`
    : `OK: ${bookingLabel(name, seats)}`
}
console.log(bookingLabel("Ada", 2))
console.log(validateBooking("Ada", 2))
console.log(validateBooking("", 2))
console.log(validateBooking("Ada", 0))
console.log(validateBooking("", 0))
export {}
