export {}
const content = new Uint8Array([97, 226, 40, 161])
console.log(new TextDecoder("utf-8", { ignoreBOM: true }).decode(content))
