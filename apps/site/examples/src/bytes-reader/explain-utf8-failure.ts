export {}
const content = new Uint8Array([97, 226, 40, 161])
try {
  console.log(
    new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(content)
  )
} catch {
  console.log("Invalid UTF-8")
}
// TextDecoder's public error does not provide the invalid byte offset.
