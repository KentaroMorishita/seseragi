const isMark = (value: string): boolean => /\p{Mark}/u.test(value)

// These known strings each contain exactly one Unicode scalar.
console.log(`acute accent: ${isMark("\u{0301}")}
U+093E mark: ${isMark("\u{093E}")}
U+20DD mark: ${isMark("\u{20DD}")}
A mark: ${isMark("A")}
é mark: ${isMark("é")}
🏽 mark: ${isMark("🏽")}
U+200D mark: ${isMark("\u{200D}")}`)

export {}
