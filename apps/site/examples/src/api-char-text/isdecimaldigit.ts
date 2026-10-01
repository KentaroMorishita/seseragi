const isDecimalDigit = (value: string): boolean =>
  /\p{Decimal_Number}/u.test(value)

// These known strings each contain exactly one Unicode scalar.
console.log(`3 decimal: ${isDecimalDigit("3")}
٣ decimal: ${isDecimalDigit("٣")}
３ decimal: ${isDecimalDigit("３")}
² decimal: ${isDecimalDigit("²")}
Ⅳ decimal: ${isDecimalDigit("Ⅳ")}`)

export {}
