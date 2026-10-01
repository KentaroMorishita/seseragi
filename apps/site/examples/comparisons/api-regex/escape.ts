function containsLiteral(wanted: string, text: string): boolean {
  return text.includes(wanted)
}
console.log(containsLiteral("a+b.txt", "file a+b.txt ready"))
console.log(containsLiteral("a+b.txt", "file aaabXtxt ready"))
console.log(containsLiteral("", "anything"))
export {}
