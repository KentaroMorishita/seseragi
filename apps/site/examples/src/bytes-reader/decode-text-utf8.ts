export {}
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })
for (const input of ["636166c3a9", "f09f8c8a", "", "ff"]) {
  const content = Uint8Array.fromHex(input)
  try {
    console.log(`Text: ${decoder.decode(content)}`)
  } catch {
    console.log("Invalid UTF-8")
  }
}
