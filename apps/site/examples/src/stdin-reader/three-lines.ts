import { stdin, stdout } from "node:process"
import { createInterface } from "node:readline"

const input = createInterface({ input: stdin, crlfDelay: Infinity })
let count = 0
for await (const line of input) {
  stdout.write(line === "" ? "Blank line\n" : `Line: ${line}\n`)
  if (++count === 3) break
}
while (count++ < 3) stdout.write("EOF\n")
input.close()
stdin.pause()
