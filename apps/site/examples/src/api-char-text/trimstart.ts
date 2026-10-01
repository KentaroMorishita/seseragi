const label = "  Aki  "
const withBom = "\uFEFF Aki"
const trimUnicodeStart = (value: string) =>
  value.replace(/^\p{White_Space}+/u, "")
console.log(`result: [${label.trimStart()}]`)
console.log(`original: [${label}]`)
console.log(`JS removes NEL: ${"\u0085Aki".trimStart() === "Aki"}`)
console.log(`Unicode removes NEL: ${trimUnicodeStart("\u0085Aki") === "Aki"}`)
console.log(`JS preserves FEFF: ${withBom.trimStart() === withBom}`)
console.log(`Unicode preserves FEFF: ${trimUnicodeStart(withBom) === withBom}`)
console.log(`empty: [${trimUnicodeStart("")}]`)
console.log(`only whitespace: [${trimUnicodeStart("\n\t\u2003")}]`)

export {}
