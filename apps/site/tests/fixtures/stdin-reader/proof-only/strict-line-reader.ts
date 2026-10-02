// Proof-only matching byte policy, not the introductory TypeScript comparison.
// This synchronous reader is for small command-line pipes. It keeps one cursor,
// supports LF/CRLF, bounds each buffered line, and leaves later lines unread.
import { readSync } from "node:fs"

type Result =
  | { kind: "line"; text: string }
  | { kind: "eof" }
  | { kind: "invalidUtf8"; offset: number }
  | { kind: "tooLong"; limit: number }
let offset = 0
const chunk = Buffer.alloc(4096)
let position = 0
let available = 0
let ended = false
function byte(): number | undefined {
  if (position === available) {
    if (ended) return undefined
    available = readSync(0, chunk)
    position = 0
    if (available === 0) {
      ended = true
      return undefined
    }
  }
  offset += 1
  return chunk[position++]
}
function invalidOffset(bytes: Uint8Array): number | undefined {
  // TextDecoder is the validator. Walk UTF-8 lead widths only to find the
  // first failing sequence's start, since TextDecoder does not expose it.
  for (let at = 0; at < bytes.length; ) {
    const lead = bytes[at]!
    const width =
      lead < 0x80
        ? 1
        : lead >= 0xc2 && lead <= 0xdf
          ? 2
          : lead >= 0xe0 && lead <= 0xef
            ? 3
            : lead >= 0xf0 && lead <= 0xf4
              ? 4
              : 1
    try {
      new TextDecoder("utf-8", { fatal: true }).decode(
        bytes.subarray(at, at + width)
      )
    } catch {
      return at
    }
    at += width
  }
  return undefined
}
export function readLine(limit = 1048576): Result {
  if (!Number.isSafeInteger(limit) || limit < 1 || limit > 67108864) {
    throw new RangeError("Line limit must be 1–67108864 bytes")
  }
  const start = offset
  const bytes: number[] = []
  let seen = false
  let terminated = false
  let tooLong = false
  for (;;) {
    const value = byte()
    if (value === undefined) break
    seen = true
    if (value === 10) {
      terminated = true
      break
    }
    if (bytes.length <= limit) bytes.push(value)
    else tooLong = true
  }
  if (!seen) return { kind: "eof" }
  if (terminated && bytes.at(-1) === 13) bytes.pop()
  if (tooLong || bytes.length > limit) return { kind: "tooLong", limit }
  const data = Uint8Array.from(bytes)
  const invalid = invalidOffset(data)
  if (invalid !== undefined)
    return { kind: "invalidUtf8", offset: start + invalid }
  return {
    kind: "line",
    text: new TextDecoder("utf-8", { fatal: true }).decode(data),
  }
}
export function render(result: Result): string {
  switch (result.kind) {
    case "eof":
      return "EOF"
    case "line":
      return result.text === "" ? "Blank line" : `Line: ${result.text}`
    case "invalidUtf8":
      return `Invalid UTF-8 at byte ${result.offset}`
    case "tooLong":
      return `Line exceeds ${result.limit} bytes`
  }
}
