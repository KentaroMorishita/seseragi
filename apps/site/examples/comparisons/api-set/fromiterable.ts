const values = new Set([3, 1, 3, 2, 1])
const otherSequence = [3, 1, 3, 2]
const noValues: number[] = []
console.log(JSON.stringify([...values]))
console.log(JSON.stringify([...new Set(otherSequence)]))
console.log(JSON.stringify([...new Set(noValues)]))
export {}
