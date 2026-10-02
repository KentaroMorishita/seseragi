import { Buffer } from "node:buffer"

for (const values of [[0, 15, 16, 171, 255], []])
  console.log(Buffer.from(values).toString("hex"))
