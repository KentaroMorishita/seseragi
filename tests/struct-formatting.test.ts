import { expect, test } from "bun:test"
import { formatSeseragiCode } from "../src/formatter/relative-indent-formatter.js"

test("format struct spacing - basic", () => {
  const input = `let point = {x:10,y:20}`
  const expected = `let point = { x: 10, y: 20 }
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format vector destructuring", () => {
  const input = `let Vector {x,y} = v`
  const expected = `let Vector { x, y } = v
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format struct construction", () => {
  const input = `Vector {x:x*2,y:y*2}`
  const expected = `Vector { x: x * 2, y: y * 2 }
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format struct in operator", () => {
  const input = `
operator * self -> scalar: Int -> Vector {
  let Vector {x,y} = self
  Vector {x:x * scalar, y:y * scalar}
}
`
  const expected = `operator * self -> scalar: Int -> Vector {
  let Vector { x, y } = self
  Vector { x: x * scalar, y: y * scalar }
}
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format complex struct", () => {
  const input = `Person {name:"Alice",age:30,active:True}`
  const expected = `Person { name: "Alice", age: 30, active: True }
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format bind operator", () => {
  const input = `let result = list>>=func`
  const expected = `let result = list >>= func
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format monad operators", () => {
  const input = `let mapped = list<$>(\\x -> x * 2)
let applied = funcs<*>values`
  const expected = `let mapped = list <$> (\\x -> x * 2)
let applied = funcs <*> values
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

// Nested structs are complex and may require manual formatting
// The main goal is to handle simple cases like Vector {x,y} -> Vector { x, y }

test("format multiline struct unchanged", () => {
  const input = `let person = {
name: "Alice",
age: 30
}`
  const expected = `let person = {
  name: "Alice",
  age: 30
}
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})
