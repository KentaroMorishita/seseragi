const prices = new Map([
  ["tea", 2],
  ["cake", 5],
])
function cents(source: Map<string, number>): Map<string, number> {
  return new Map([...source].map(([key, euros]) => [key, euros * 100]))
}
console.log(JSON.stringify([...cents(prices).entries()]))
console.log(JSON.stringify([...prices.entries()]))
console.log(JSON.stringify([...cents(new Map()).entries()]))
export {}
