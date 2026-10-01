const isAlphabetic = (value: string): boolean => /\p{Alphabetic}/u.test(value)

// These known strings each contain exactly one Unicode scalar.
console.log(`A alphabetic: ${isAlphabetic("A")}
漢 alphabetic: ${isAlphabetic("漢")}
3 alphabetic: ${isAlphabetic("3")}
U+0345 alphabetic: ${isAlphabetic("\u{0345}")}
U+0301 alphabetic: ${isAlphabetic("\u{0301}")}`)

export {}
