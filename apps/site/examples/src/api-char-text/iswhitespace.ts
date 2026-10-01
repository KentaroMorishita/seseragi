const isWhitespace = (value: string): boolean => /\p{White_Space}/u.test(value)

// These known strings each contain exactly one Unicode scalar.
console.log(`space whitespace: ${isWhitespace(" ")}
tab whitespace: ${isWhitespace("\t")}
NEL whitespace: ${isWhitespace("\u{0085}")}
EM SPACE whitespace: ${isWhitespace("\u{2003}")}
U+FEFF whitespace: ${isWhitespace("\u{FEFF}")}
U+200B whitespace: ${isWhitespace("\u{200B}")}`)

export {}
