const values = new Map([
  ["b", 2],
  ["a", 2],
  ["b", 9],
])
console.log(values.size)
console.log(new Map(values).set("b", 5).size)
console.log(new Map(values).set("c", 3).size)
console.log(new Map<string, number>().size)
export {}
