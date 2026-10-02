import { stdin, stdout } from "node:process"
import { createInterface } from "node:readline"

const input = createInterface({ input: stdin, crlfDelay: Infinity })
let found = false
for await (const line of input) {
  stdout.write(line === "" ? "Blank line\n" : `Line: ${line}\n`)
  found = true
  break
}
if (!found) stdout.write("EOF\n")
input.close()
stdin.pause()
