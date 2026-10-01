function replace(text: string): string {
  return text.replace(/x([0-9]+)?/gu, (_whole, digits: string | undefined) =>
    digits === undefined ? "[absent]" : `[${digits}]`
  )
}
console.log(replace("x12 x x3"))
console.log(replace("nothing to replace"))
console.log(`empty: [${replace("")}]`)
export {}
