const values = new Set([2, 1, 2])
const moved = new Set(values)
moved.delete(2)
moved.add(2)
console.log(JSON.stringify(Array.from(values)))
console.log(JSON.stringify(Array.from(moved)))
console.log(JSON.stringify(Array.from(new Set<number>())))
console.log(JSON.stringify(Array.from(values)))
export {}
