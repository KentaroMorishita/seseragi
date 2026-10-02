import { make, combine, read } from "../../../runtime/ts/src/signal"
const sources = await Promise.all(
  [1, 2, 3, 4, 5].map((value) => make(value)({}))
)
let evaluated = 0
function projected() {
  let result = combine(
    (a: number) => (b: number) => [a, b],
    sources[0],
    sources[1]
  )
  for (let i = 2; i < 4; i++)
    result = combine(
      (values: number[]) => (value: number) => [...values, value],
      result,
      sources[i]
    )
  return combine(
    (values: number[]) => (value: number) => {
      evaluated++
      return [...values, value]
    },
    result,
    sources[4]
  )
}
const count = 100000
const measurements = []
for (let round = 0; round < 6; round++) {
  const stable = projected()
  let sum = 0
  evaluated = 0
  let start = performance.now()
  for (let i = 0; i < count; i++) sum += (read(stable)({}) as number[])[0]
  const reused = {
    ms: performance.now() - start,
    evaluations: evaluated,
    checksum: sum,
  }
  sum = 0
  evaluated = 0
  start = performance.now()
  for (let i = 0; i < count; i++) sum += (read(projected())({}) as number[])[0]
  const oneshot = {
    ms: performance.now() - start,
    evaluations: evaluated,
    checksum: sum,
  }
  if (round) measurements.push({ reused, oneshot })
}
console.log(
  JSON.stringify(
    {
      bun: Bun.version,
      platform: process.platform,
      arch: process.arch,
      samples: count,
      derivedNodesPerProjection: 4,
      measurements,
    },
    null,
    2
  )
)
