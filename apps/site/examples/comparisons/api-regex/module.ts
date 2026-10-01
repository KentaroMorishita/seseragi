function orderCode(text: string): string {
  const found = /ORD-[0-9]+/u.exec(text)
  return found === null ? "no order code" : `order: ${found[0]}`
}
console.log(orderCode("memo ORD-42 ready"))
console.log(orderCode("no reference yet"))
export {}
