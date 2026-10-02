const counts = new Map([
  ["one", 1],
  ["zero", 0],
])
const flags = new Map([["flag", false]])
const labels = new Map([["", ""]])
console.log(
  JSON.stringify([
    ...new Map(
      [...counts].filter(([key, value]) => key === "zero" && value === 0)
    ),
  ])
)
console.log(
  JSON.stringify([
    ...new Map(
      [...flags].filter(([key, value]) => key === "flag" && value === false)
    ),
  ])
)
console.log(
  JSON.stringify([
    ...new Map(
      [...labels].filter(([key, value]) => key === "" && value === "")
    ),
  ])
)
console.log(
  JSON.stringify([
    ...new Set([...new Set([1, 0])].filter((value) => value === 0)),
  ])
)
console.log(
  JSON.stringify([
    ...new Set([...new Set([true, false])].filter((value) => value === false)),
  ])
)
console.log(
  JSON.stringify([
    ...new Set([...new Set(["x", ""])].filter((value) => value === "")),
  ])
)
export {}
