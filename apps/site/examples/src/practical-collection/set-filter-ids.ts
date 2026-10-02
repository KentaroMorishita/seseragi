const ids = new Set([4, 1, 4, 3, 0])
console.log(
  JSON.stringify([...new Set([...ids].filter((value) => value >= 3))])
)
console.log(JSON.stringify([...ids]))
console.log(
  JSON.stringify([
    ...new Set([...new Set<number>()].filter((value) => value >= 3)),
  ])
)
export {}
