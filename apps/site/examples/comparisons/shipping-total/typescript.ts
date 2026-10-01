function totalWithShipping(
  freeFrom: number,
  fee: number,
  subtotal: number
): number {
  return subtotal >= freeFrom ? subtotal : subtotal + fee
}

const standardTotal = (subtotal: number) =>
  totalWithShipping(5000, 500, subtotal)
const smallOrder = standardTotal(3200)
const largeOrder = standardTotal(5000)

console.log(`${smallOrder}, ${largeOrder}`)
