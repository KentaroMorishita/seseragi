function replace(source: string, replacement: string, text: string): string {
  return text.replace(new RegExp(source, "gu"), () => replacement)
}
console.log(replace("([0-9]+)", "$1\\", "a12b3"))
console.log(replace("[0-9]+", "#", "no digits"))
console.log(replace("", "|", "a👍"))
console.log(replace("", "|", ""))
console.log("a12b3".replace(/([0-9]+)/gu, "$1\\"))
export {}
