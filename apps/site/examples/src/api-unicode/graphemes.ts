const label = "A👍🏽e\u0301"
const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" })
const pieces = [...segmenter.segment(label)].map((item) => item.segment)
console.log(`visible pieces: ${pieces.length}
scalars: ${Array.from(label).length}
UTF-8 bytes: ${new TextEncoder().encode(label).length}
UTF-16 units: ${label.length}
empty: ${[...segmenter.segment("")].length}`)
