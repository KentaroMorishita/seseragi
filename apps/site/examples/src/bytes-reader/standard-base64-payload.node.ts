import { Buffer } from "node:buffer"

for (const label of ["", "f", "fo", "foo", "foob", "fooba", "foobar"])
  console.log(Buffer.from(new TextEncoder().encode(label)).toString("base64"))
