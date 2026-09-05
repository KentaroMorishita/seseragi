import { expect, test } from "bun:test"
import { arrayReducible } from "../src/array"
import { combine } from "../src/collection"
import { intAdd, intMul, intOne, intZero } from "../src/int"
import {
  Product,
  productMonoid,
  productSemigroup,
  Sum,
  sumMonoid,
  sumSemigroup,
} from "../src/sum"

test("Sum and Product combine through ordinary numeric dictionaries", () => {
  const sum = sumMonoid<number>(intZero, intAdd)
  const product = productMonoid<number>(intOne, intMul)
  expect(combine(arrayReducible, sum, [Sum(1), Sum(2), Sum(3)])).toEqual(Sum(6))
  expect(
    combine(arrayReducible, product, [Product(2), Product(3), Product(4)])
  ).toEqual(Product(24))
  expect(combine(arrayReducible, sum, [])).toEqual(Sum(0))
  expect(combine(arrayReducible, product, [])).toEqual(Product(1))
  expect(Object.isFrozen(Sum(1))).toBe(true)
  expect(Object.isFrozen(Product(1))).toBe(true)
})

test("custom dictionaries determine identity, operation, and left-to-right order", () => {
  const calls: string[] = []
  const operation = {
    mul: (left: string) => (right: string) => {
      calls.push(`${left}|${right}`)
      return left + right
    },
  }
  const identity = {
    one: () => {
      calls.push("one")
      return ""
    },
  }
  const dictionary = productMonoid<string>(identity, operation)
  expect(calls).toEqual([])
  expect(
    combine(arrayReducible, dictionary, [
      Product("a"),
      Product("b"),
      Product("c"),
    ])
  ).toEqual(Product("abc"))
  expect(calls).toEqual(["one", "|a", "a|b", "ab|c"])
  expect(
    productSemigroup<string>(operation).append(Product("x"))(Product("y"))
  ).toEqual(Product("xy"))
  expect(sumSemigroup<number>(intAdd).append(Sum(2))(Sum(3))).toEqual(Sum(5))
})

test("wrapper composition preserves checked numeric defects", () => {
  expect(() =>
    sumSemigroup<number>(intAdd).append(Sum(Number.MAX_SAFE_INTEGER))(Sum(1))
  ).toThrow()
  expect(() =>
    productSemigroup<number>(intMul).append(Product(Number.MAX_SAFE_INTEGER))(
      Product(2)
    )
  ).toThrow()
})
