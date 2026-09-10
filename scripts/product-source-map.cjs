// Installed-product assertion for the composed O02 JS -> Seseragi source map.
function assertOriginalSourceMap(map, module, source) {
  const index = map.sources.indexOf(`seseragi://${module}`)
  if (map.version !== 3 || index < 0 || map.sourcesContent[index] !== source) {
    throw new Error(
      "production source map does not preserve original Seseragi source"
    )
  }
  const digits =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
  let sourceIndex = 0
  let mapped = false
  for (const row of map.mappings.split(";")) {
    for (const segment of row.split(",").filter(Boolean)) {
      const values = []
      let value = 0
      let shift = 0
      for (const character of segment) {
        const digit = digits.indexOf(character)
        if (digit < 0 || shift > 50) throw new Error("invalid source-map VLQ")
        value += (digit & 31) * 2 ** shift
        if (digit & 32) shift += 5
        else {
          values.push(value % 2 ? -Math.floor(value / 2) : value / 2)
          value = 0
          shift = 0
        }
      }
      if (shift || ![1, 4, 5].includes(values.length))
        throw new Error("invalid source-map segment")
      if (values.length >= 4) {
        sourceIndex += values[1]
        if (sourceIndex === index) mapped = true
      }
    }
  }
  if (!mapped)
    throw new Error("production source map has no mappings to original app")
}
module.exports = { assertOriginalSourceMap }
