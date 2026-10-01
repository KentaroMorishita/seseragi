const capital: string = "Σ"
const finalForm: string = "ς"

// These comparisons use the same known one-scalar inputs.
console.log(`same scalar: ${capital === finalForm}
case-insensitive sigma: ${/^Σ$/iu.test(finalForm)}
case-insensitive sharp s: ${/^ß$/iu.test("ẞ")}
dotted I equals i: ${/^İ$/iu.test("i")}`)

export {}
