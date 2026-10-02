import { stdin, stdout } from "node:process"
import { createInterface } from "node:readline"

// Count bytes after readline decodes a complete line. This does not impose
// strict UTF-8 decoding or a bound on readline's buffered input.
const input = createInterface({ input: stdin, crlfDelay: Infinity })
const encoder = new TextEncoder()
const limit = 4
let count = 0
for await (const line of input) {
  const message =
    encoder.encode(line).length > limit
      ? `Line exceeds ${limit} bytes`
      : line === ""
        ? "Blank line"
        : `Line: ${line}`
  stdout.write(`${message}\n`)
  if (++count === 3) break
}
while (count++ < 3) stdout.write("EOF\n")
input.close()
stdin.pause()
