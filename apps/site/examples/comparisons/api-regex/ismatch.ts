function check(source: string, text: string): boolean {
  return new RegExp(source, "u").test(text)
}
console.log(check("ORD-[0-9]+", "memo ORD-42"))
console.log(check("^ORD-[0-9]+$", "memo ORD-42"))
console.log(check("\\d+", "123"))
console.log(check("\\d+", "٣"))
console.log(check("^\\w+$", "漢_٣"))
export {}
