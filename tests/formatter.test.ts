import { expect, test } from "bun:test"
import { formatSeseragiCode } from "../src/formatter/relative-indent-formatter.js"

test("format simple record", () => {
  const input = `
let person = { name: "Alice", age: 30 }
`
  const expected = `let person = { name: "Alice", age: 30 }
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format multi-line record", () => {
  const input = `
let person = {
name: "Alice",
age: 30,
city: "Tokyo"
}
`
  const expected = `let person = {
  name: "Alice",
  age: 30,
  city: "Tokyo"
}
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format nested record", () => {
  const input = `
let employee = {
info: { name: "Bob", age: 25 },
department: "Engineering",
salary: 75000
}
`
  const expected = `let employee = {
  info: { name: "Bob", age: 25 },
  department: "Engineering",
  salary: 75000
}
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format struct destructuring", () => {
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

test("format struct with mixed spacing", () => {
  const input = `let point = {x:10,y:20}`
  const expected = `let point = { x: 10, y: 20 }
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format deeply nested record", () => {
  const input = `
let user = {
profile: {
personal: { firstName: "Carol", lastName: "Smith" },
contact: { email: "carol@example.com", phone: "090-1234-5678" }
},
preferences: {
theme: "dark",
language: "ja",
notifications: { email: True, push: False }
},
account: {
id: 12345,
created: "2024-01-15",
verified: True
}
}
`
  const expected = `let user = {
  profile: {
    personal: { firstName: "Carol", lastName: "Smith" },
    contact: { email: "carol@example.com", phone: "090-1234-5678" }
  },
  preferences: {
    theme: "dark",
    language: "ja",
    notifications: { email: True, push: False }
  },
  account: {
    id: 12345,
    created: "2024-01-15",
    verified: True
  }
}
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format code after record", () => {
  const input = `
let user = {
name: "Alice"
}

show user

let x = 42
`
  const expected = `let user = {
  name: "Alice"
}

show user

let x = 42
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format function block", () => {
  const input = `
fn processNumber x: Int -> Int {
let doubled = x * 2
let incremented = doubled + 1
incremented
}
`
  const expected = `fn processNumber x: Int -> Int {
  let doubled = x * 2
  let incremented = doubled + 1
  incremented
}
`
  expect(formatSeseragiCode(input.trim())).toBe(expected)
})

test("format function with expression continuation", () => {
  const input = `fn factorial n: Int -> Int =
if n <= 1 then 1 else n * factorial (n - 1)

fn getAge person :{ name: String, age: Int } -> Int =
person.age`

  const expected = `fn factorial n: Int -> Int =
  if n <= 1 then 1 else n * factorial (n - 1)

fn getAge person :{ name: String, age: Int } -> Int =
  person.age
`

  expect(formatSeseragiCode(input)).toBe(expected)
})
