import { stdout } from "node:process"
import { readLine, render } from "./strict-line-reader.js"

for (let count = 0; count < 3; count++) stdout.write(`${render(readLine())}\n`)
