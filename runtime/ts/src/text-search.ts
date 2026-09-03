/** Non-overlapping literal matches in O(input + needle + matches). */
export function* literalMatches(
  needle: string,
  text: string
): Generator<number> {
  if (needle.length === 0) {
    let index = 0
    yield index
    for (const scalar of text) {
      index += scalar.length
      yield index
    }
    return
  }
  const fallback = new Uint32Array(needle.length)
  for (let index = 1, matched = 0; index < needle.length; index++) {
    while (matched > 0 && needle[index] !== needle[matched])
      matched = fallback[matched - 1]!
    if (needle[index] === needle[matched]) matched++
    fallback[index] = matched
  }
  for (let index = 0, matched = 0; index < text.length; index++) {
    while (matched > 0 && text[index] !== needle[matched])
      matched = fallback[matched - 1]!
    if (text[index] === needle[matched]) matched++
    if (matched === needle.length) {
      yield index + 1 - needle.length
      matched = 0
    }
  }
}
