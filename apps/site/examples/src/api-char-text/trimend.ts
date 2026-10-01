const label = "  Aki  "
const withBom = "Aki \uFEFF"
const trimUnicodeEnd = (value: string) =>
  value.replace(/\p{White_Space}+$/u, "")
console.log(`result: [${label.trimEnd()}]`)
console.log(`original: [${label}]`)
console.log(`JS removes NEL: ${"Aki\u0085".trimEnd() === "Aki"}`)
console.log(`Unicode removes NEL: ${trimUnicodeEnd("Aki\u0085") === "Aki"}`)
console.log(`JS preserves FEFF: ${withBom.trimEnd() === withBom}`)
console.log(`Unicode preserves FEFF: ${trimUnicodeEnd(withBom) === withBom}`)
console.log(`empty: [${trimUnicodeEnd("")}]`)
console.log(`only whitespace: [${trimUnicodeEnd("\n\t\u2003")}]`)

export {}
