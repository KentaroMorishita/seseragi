export {}
const content = new TextEncoder().encode("café")
console.log(content.length)
console.log(content.toHex())
console.log(content.toBase64())
console.log(content.toBase64({ alphabet: "base64url", omitPadding: true }))
console.log(
  new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(content)
)
