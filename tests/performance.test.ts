import { expect, test } from "bun:test"
import { generateTypeScript } from "../src/codegen.js"
import { Parser } from "../src/parser.js"

test("Performance test - Large factorial calculation", () => {
  const source = `
    fn factorial n: Int -> Int = if n <= 1 then 1 else n * factorial (n - 1)
    
    let result1 = factorial 5
    let result2 = factorial 10
    print result1
    print result2
  `

  const startTime = Date.now()
  const parser = new Parser(source)
  const program = parser.parse()
  const generated = generateTypeScript(program.statements)
  const endTime = Date.now()

  expect(generated).toContain("factorial")
  expect(generated).toContain("ssrgPrint")
  expect(endTime - startTime).toBeLessThan(1000) // Should complete within 1 second
})

test("Performance test - Complex nested expressions", () => {
  const source = `
    fn add x: Int -> y: Int -> Int = x + y
    fn multiply x: Int -> y: Int -> Int = x * y
    
    let result = add (multiply 5 3) (multiply 2 4) | add 10 | multiply 2
    print result
  `

  const startTime = Date.now()
  const parser = new Parser(source)
  const program = parser.parse()
  const generated = generateTypeScript(program.statements)
  const endTime = Date.now()

  expect(generated).toContain("add")
  expect(generated).toContain("multiply")
  expect(endTime - startTime).toBeLessThan(500) // Should complete within 0.5 seconds
})

test("Performance test - Currying and partial application", () => {
  const source = `
    fn add x: Int -> y: Int -> z: Int -> Int = x + y + z
    
    let addTen = add 10
    let addTenFive = addTen 5
    let result = addTenFive 3
    
    print result
  `

  const startTime = Date.now()
  const parser = new Parser(source)
  const program = parser.parse()
  const generated = generateTypeScript(program.statements)
  const endTime = Date.now()

  expect(generated).toContain("function(y: number)")
  expect(generated).toContain("add")
  expect(endTime - startTime).toBeLessThan(500)
})
