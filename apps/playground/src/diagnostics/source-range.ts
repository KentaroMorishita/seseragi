const encoder = new TextEncoder()

export function utf8ByteOffsetToUtf16(
  source: string,
  byteOffset: number
): number {
  const target = Math.max(0, byteOffset)
  let bytes = 0
  let utf16 = 0

  for (const scalar of source) {
    const width = encoder.encode(scalar).length
    if (bytes + width > target) return utf16
    bytes += width
    utf16 += scalar.length
    if (bytes === target) return utf16
  }

  return source.length
}

export function utf8RangeToUtf16(
  source: string,
  range: { readonly start: number; readonly end: number }
): { readonly from: number; readonly to: number } {
  const from = utf8ByteOffsetToUtf16(source, range.start)
  const to = utf8ByteOffsetToUtf16(source, range.end)
  return { from, to: Math.max(from, to) }
}

export function utf16OffsetToUtf8Byte(
  source: string,
  utf16Offset: number
): number {
  const target = Math.max(0, Math.min(source.length, utf16Offset))
  return encoder.encode(source.slice(0, target)).length
}

export type SourcePosition = {
  readonly line: number
  readonly column: number
}

export type SourceLocation = {
  readonly start: SourcePosition
  readonly end: SourcePosition
}

export function utf16OffsetToSourcePosition(
  source: string,
  utf16Offset: number
): SourcePosition {
  const target = Math.max(0, Math.min(source.length, utf16Offset))
  const before = source.slice(0, target)
  const lastNewline = before.lastIndexOf("\n")
  return {
    line: before.split("\n").length,
    column: Array.from(before.slice(lastNewline + 1)).length + 1,
  }
}

export function utf8RangeToSourceLocation(
  source: string,
  range: { readonly start: number; readonly end: number }
): SourceLocation {
  const utf16 = utf8RangeToUtf16(source, range)
  return {
    start: utf16OffsetToSourcePosition(source, utf16.from),
    end: utf16OffsetToSourcePosition(source, utf16.to),
  }
}

export function formatSourceLocation(
  filename: string,
  location: SourceLocation
): string {
  const start = `${filename}:${location.start.line}:${location.start.column}`
  return location.start.line === location.end.line
    ? `${start}–${location.end.column}`
    : `${start}–${location.end.line}:${location.end.column}`
}

export function describeSourceLocation(location: SourceLocation): string {
  return location.start.line === location.end.line
    ? `Line ${location.start.line}, columns ${location.start.column}–${location.end.column}`
    : `Line ${location.start.line}, column ${location.start.column} to line ${location.end.line}, column ${location.end.column}`
}
