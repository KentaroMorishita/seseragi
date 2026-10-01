const values = new Map([
  ["zero", 0],
  ["tea", 2],
])
console.log(values.has("zero"))
console.log(values.has("missing"))
console.log(new Map<string, number>().has("tea"))
export {}
