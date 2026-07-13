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
