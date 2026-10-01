type NonEmpty<T> = readonly [T, ...T[]]
const single: NonEmpty<number> = [10]
const several: NonEmpty<number> = [10, 20, 30]
console.log(JSON.stringify(single.slice(1)))
console.log(JSON.stringify(several.slice(1)))
export {}
