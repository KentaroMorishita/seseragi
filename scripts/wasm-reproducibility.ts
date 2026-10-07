import { createHash } from "node:crypto"
import { resolve } from "node:path"

const root = resolve(import.meta.dir, "..")
const path = "apps/playground/src/wasm/pkg/seseragi_wasm_bg.wasm"
const tracked = Bun.spawnSync(["git", "show", `HEAD:${path}`], { cwd: root })
if (tracked.exitCode !== 0) throw new Error("Cannot read committed WASM")
const rebuilt = new Uint8Array(
  await Bun.file(resolve(root, path)).arrayBuffer()
)

function sections(bytes: Uint8Array) {
  let offset = 8
  function leb() {
    let result = 0
    let shift = 0
    for (;;) {
      const byte = bytes[offset++]
      if (byte === undefined || shift > 28) throw new Error("Invalid WASM size")
      result |= (byte & 127) << shift
      if (byte < 128) return result
      shift += 7
    }
  }
  const result = new Map<string, Uint8Array>()
  while (offset < bytes.length) {
    const kind = bytes[offset++]
    const size = leb()
    const start = offset
    const end = start + size
    if (end > bytes.length) throw new Error("Truncated WASM section")
    let name = String(kind)
    if (kind === 0) {
      const length = leb()
      name = `custom:${new TextDecoder().decode(bytes.slice(offset, offset + length))}`
    }
    result.set(name, bytes.slice(start, end))
    offset = end
  }
  return result
}
const expected = sections(tracked.stdout)
const actual = sections(rebuilt)
const digest = (data: Uint8Array) =>
  createHash("sha256").update(data).digest("hex")
for (const [name, bytes] of actual) {
  const original = expected.get(name)
  console.log(
    JSON.stringify({
      section: name,
      expected: original ? digest(original) : null,
      actual: digest(bytes),
      expectedBytes: original?.length,
      actualBytes: bytes.length,
    })
  )
  if (original && digest(original) !== digest(bytes)) {
    const difference = bytes.findIndex(
      (byte, index) => byte !== original[index]
    )
    const start = Math.max(0, difference - 40)
    console.log(
      JSON.stringify({
        section: name,
        firstDifference: difference,
        expectedContext: new TextDecoder().decode(
          original.slice(start, start + 200)
        ),
        actualContext: new TextDecoder().decode(
          bytes.slice(start, start + 200)
        ),
      })
    )
  }
}
