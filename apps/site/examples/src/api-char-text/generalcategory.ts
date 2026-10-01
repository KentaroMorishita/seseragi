const isUppercaseLetter = (value: string): boolean =>
  /\p{Uppercase_Letter}/u.test(value)

// These known strings each contain exactly one Unicode scalar.
console.log(`A uppercase: ${isUppercaseLetter("A")}
٣ uppercase: ${isUppercaseLetter("٣")}
U+0301 uppercase: ${isUppercaseLetter("\u{0301}")}
U+FEFF uppercase: ${isUppercaseLetter("\u{FEFF}")}
U+10FFFF uppercase: ${isUppercaseLetter("\u{10FFFF}")}`)

export {}
