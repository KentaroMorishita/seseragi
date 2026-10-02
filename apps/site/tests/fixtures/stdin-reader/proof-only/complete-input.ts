import { stdin, stdout } from "node:process"

stdin.setEncoding("utf8")
let text = ""
for await (const chunk of stdin) text += chunk
stdout.write(`Complete input: [${text}]\n`)
