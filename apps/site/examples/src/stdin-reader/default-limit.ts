import { stdin, stdout } from "node:process"
import { createInterface } from "node:readline"

// This checks decoded content bytes, not raw decoding or stream memory.
const input = createInterface({ input: stdin, crlfDelay: Infinity })
const limit = 1_048_576
let found = false
for await (const line of input) {
  const message =
    new TextEncoder().encode(line).length > limit
      ? `Line exceeds ${limit} bytes`
      : line === ""
        ? "Blank line"
        : "Line accepted"
  stdout.write(`${message}\n`)
  found = true
  break
}
if (!found) stdout.write("EOF\n")
input.close()
stdin.pause()
