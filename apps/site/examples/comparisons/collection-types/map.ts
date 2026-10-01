function priceLabel(prices: Map<string, number>): string {
  const price = prices.get("tea")
  return price === undefined ? "not listed" : String(price)
}

const prices: Map<string, number> = new Map([
  ["tea", 0],
  ["coffee", 5],
  ["tea", 2],
])
const updated = new Map(prices).set("tea", 3)
console.log(JSON.stringify([...prices]))
console.log(JSON.stringify([...updated]))
console.log(priceLabel(new Map([["tea", 0]])))
console.log(priceLabel(new Map()))
const reordered = new Map([
  ["coffee", 5],
  ["tea", 2],
])
console.log(
  prices.size === reordered.size &&
    [...prices].every(([key, value]) => reordered.get(key) === value)
)

export {}
