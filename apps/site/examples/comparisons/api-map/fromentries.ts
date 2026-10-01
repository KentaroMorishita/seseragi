const values = new Map([
  ["b", 2],
  ["a", 1],
  ["b", 9],
])
const noPairs: Array<[string, number]> = []
console.log(JSON.stringify([...values.entries()]))
console.log(JSON.stringify([...new Map(noPairs).entries()]))
export {}
