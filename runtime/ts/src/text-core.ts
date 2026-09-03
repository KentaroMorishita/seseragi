const encoder = new TextEncoder()
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })

export function utf8Width(point: number): number {
  return point <= 0x7f ? 1 : point <= 0x7ff ? 2 : point <= 0xffff ? 3 : 4
}

/** A returned small substring must not keep a large input string alive. */
export function copySubstring(
  text: string,
  start: number,
  end: number
): string {
  if (start === end) return ""
  return decoder.decode(encoder.encode(text.slice(start, end)))
}

export function stringFromPoints(points: readonly number[]): string {
  const chunks: string[] = []
  // Bounded argument lists, including adversarially long combining sequences.
  for (let index = 0; index < points.length; index += 1024) {
    chunks.push(String.fromCodePoint(...points.slice(index, index + 1024)))
  }
  return chunks.join("")
}
