function pieces(source: string, text: string): string {
  return JSON.stringify(text.split(new RegExp(source, "u")))
}
console.log(pieces(",", "a,,b,"))
console.log(pieces("(,)", ",a,"))
console.log(pieces("", "a👍"))
console.log(pieces(",", ""))
export {}
