const words = (value: string) =>
  value.split(/\p{White_Space}+/u).filter((part) => part.length > 0)
const sentence = "  hello,\u2003世界!  "
const pieces = words(sentence)
console.log(`pieces: ${JSON.stringify(pieces)}`)
console.log(`joined: ${pieces.join(" | ")}`)
console.log(`Japanese: ${JSON.stringify(words("東京都渋谷区"))}`)
console.log(`empty: ${JSON.stringify(words(""))}`)
console.log(`only whitespace: ${JSON.stringify(words("\n\t\u0085"))}`)
console.log(`FEFF preserved: ${words("A\uFEFF B")[0] === "A\uFEFF"}`)
console.log(`JS \\s treats NEL as whitespace: ${/\s/u.test("\u0085")}`)
console.log(`JS \\s treats FEFF as whitespace: ${/\s/u.test("\uFEFF")}`)

export {}
