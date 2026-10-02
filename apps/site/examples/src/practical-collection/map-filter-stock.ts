const stock = new Map<string, number>([
  ["tea", 2],
  ["hold", 5],
  ["coffee", 3],
  ["empty", 0],
])
const selected = new Map(
  [...stock].filter(([sku, quantity]) => sku !== "hold" && quantity > 0)
)
console.log(JSON.stringify([...selected]))
console.log(JSON.stringify([...stock]))
const none = new Map<string, number>()
console.log(
  JSON.stringify([
    ...new Map(
      [...none].filter(([sku, quantity]) => sku !== "hold" && quantity > 0)
    ),
  ])
)
export {}
