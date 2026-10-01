const value: string = "Aé😀e\u0301"
console.log(`UTF-16 units: ${value.length}`)
console.log(`scalars: ${Array.from(value).length}`)
console.log(`UTF-8 bytes: ${new TextEncoder().encode(value).length}`)
const segments = new Intl.Segmenter("en", { granularity: "grapheme" })
console.log(`graphemes: ${Array.from(segments.segment(value)).length}`)
