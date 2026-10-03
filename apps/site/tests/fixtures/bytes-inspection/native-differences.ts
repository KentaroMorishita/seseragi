export {}
const show = (values: Uint8Array) => `[${Array.from(values).join(", ")}]`
console.log(`construction-wraps|${show(Uint8Array.from([-1, 256, 257, 1.9]))}`)
const content = Uint8Array.of(0, 15, 255)
console.log(`native-slice-clamps|${show(content.slice(0, 99))}`)
console.log(`native-slice-negative|${show(content.slice(-1, 3))}`)
console.log(`native-slice-reversed|${show(content.slice(2, 1))}`)
console.log(`content-after|${show(content)}`)
