const original = new Set([3, 1, 2])
console.log(JSON.stringify([...new Set(original).add(1)]))
console.log(JSON.stringify([...new Set(original).add(4)]))
console.log(JSON.stringify([...original]))
console.log(JSON.stringify([...new Set<number>().add(0)]))
export {}
