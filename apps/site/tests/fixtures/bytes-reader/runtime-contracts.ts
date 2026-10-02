import { readFileSync } from "node:fs"

const cases = JSON.parse(
  readFileSync(new URL("./contract-cases.json", import.meta.url), "utf8")
)
const strict = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })
const lossy = new TextDecoder("utf-8", { ignoreBOM: true })
function outcome(input: string, format: "hex" | "base64" | "url"): string {
  try {
    const bytes = decode(input, format)
    if (format !== "hex" && encode(bytes, format) !== input) return "rejected"
    return "ok:" + encode(bytes, "hex")
  } catch {
    return "rejected"
  }
}
for (const format of ["hex", "base64", "url"] as const) {
  for (const [index, row] of (cases[format] as [string, string][]).entries()) {
    const actual = outcome(row[0], format)
    const expected = row[1].startsWith("ok:") ? row[1] : "rejected"
    if (actual !== expected)
      throw new Error(`${format}-${index}: ${actual} != ${expected}`)
    console.log(`${format}-${index}|${actual}`)
  }
}
for (const [label, values, expectedStrict, expectedLossy] of cases.utf8 as [
  string,
  number[],
  string,
  string,
][]) {
  const bytes = Uint8Array.from(values)
  let result: string
  try {
    result =
      "ok:" + encode(new TextEncoder().encode(strict.decode(bytes)), "hex")
  } catch {
    result = "rejected"
  }
  const preview = encode(new TextEncoder().encode(lossy.decode(bytes)), "hex")
  const expected = expectedStrict.startsWith("ok:")
    ? expectedStrict
    : "rejected"
  if (result !== expected || preview !== expectedLossy)
    throw new Error(`UTF-8 ${label}: ${result}|${preview}`)
  console.log(`utf-${label}|${result}|${preview}`)
}
const bom = Uint8Array.from([239, 187, 191, 65])
console.log(
  `default-bom|${encode(new TextEncoder().encode(new TextDecoder().decode(bom)), "hex")}`
)
function encode(content: Uint8Array, format: "hex" | "base64" | "url"): string {
  return format === "hex"
    ? content.toHex()
    : content.toBase64({
        alphabet: format === "url" ? "base64url" : "base64",
        omitPadding: format === "url",
      })
}
function decode(input: string, format: "hex" | "base64" | "url"): Uint8Array {
  return format === "hex"
    ? Uint8Array.fromHex(input)
    : Uint8Array.fromBase64(input, {
        alphabet: format === "url" ? "base64url" : "base64",
        lastChunkHandling: format === "url" ? "loose" : "strict",
      })
}
