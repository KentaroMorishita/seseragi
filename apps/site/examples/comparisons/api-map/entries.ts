const values = new Map([
  ["b", 2],
  ["a", 2],
  ["c", 3],
])
console.log(JSON.stringify([...values.entries()]))
console.log(JSON.stringify([...new Map<string, number>().entries()]))
export {}
