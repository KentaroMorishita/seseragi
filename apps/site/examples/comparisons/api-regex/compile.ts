function check(source: string): string {
  try {
    return `matches: ${new RegExp(source, "u").test("ORD-42")}`
  } catch (error) {
    if (error instanceof SyntaxError) return "rejected pattern"
    throw error
  }
}
console.log(check("ORD-[0-9]+"))
console.log(check("é["))
console.log(check("a(?=b)"))
console.log(check("\\d+"))
export {}
