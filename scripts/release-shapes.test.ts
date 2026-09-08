import { expect, test } from "bun:test"
import { checkShapes, type Inspection, type Predicate } from "./release-shapes"

const metadata: Inspection = {
  profile: "release",
  inspectionRoots: { "original::count": "renamed" },
  newtypeConstructors: ["Wrapped"],
  outputs: { typescript: "main.ts" },
}
function inspect(source: string, predicate: Predicate) {
  checkShapes(source, metadata, [
    { symbol: "original::count", require: [predicate] },
  ])
}
test("newtype proof follows binders and rejects residual wrappers, checks and calls", () => {
  expect(() =>
    inspect("const renamed = (n: number) => n + 1", "newtype-erased")
  ).not.toThrow()
  for (const body of [
    '({ tag: "Wrapped", value: n })',
    "n.value",
    'n["tag"]',
  ]) {
    expect(() =>
      inspect(`const renamed = (n: any) => ${body}`, "newtype-erased")
    ).toThrow()
  }
  expect(() =>
    inspect(
      "const Wrapped = (n: number) => n; const renamed = (n: number) => Wrapped(n)",
      "newtype-erased"
    )
  ).toThrow()
  expect(() =>
    inspect(
      "const helper = (n: any) => n.value; const renamed = (n: any) => helper(n)",
      "newtype-erased"
    )
  ).toThrow()
  expect(() =>
    inspect(
      "const renamed = (Wrapped: number) => Wrapped + 1",
      "newtype-erased"
    )
  ).not.toThrow()
})
test("tail proof checks emitted loop and rejects every remaining self reference", () => {
  const loop =
    "const renamed = (n: number) => { while (true) { if (n === 0) return n; n--; continue; } }"
  expect(() => inspect(loop, "self-tail-loop")).not.toThrow()
  expect(() =>
    inspect(loop.replace("n--;", "n = renamed(n - 1) + 1;"), "self-tail-loop")
  ).toThrow()
  expect(() =>
    inspect(
      "const renamed = (n: number) => n ? renamed(n - 1) : 0",
      "self-tail-loop"
    )
  ).toThrow()
  expect(() =>
    inspect("const renamed = (n: number) => n + 1", "self-tail-loop")
  ).toThrow()
})
test("sugar proof rejects residual dispatch objects and missing roots fail closed", () => {
  expect(() =>
    inspect("const renamed = (n: number) => effect(n)", "surface-sugar-erased")
  ).not.toThrow()
  expect(() =>
    inspect(
      'const renamed = () => ({ kind: "do", steps: [] })',
      "surface-sugar-erased"
    )
  ).toThrow()
  expect(() =>
    inspect("const count = () => 0", "surface-sugar-erased")
  ).toThrow()
})

test("Effect roots and nested local loops do not prove root self TCO", () => {
  const source =
    "const renamed = (n: number) => { while (true) { if (n === 0) return n; n--; continue; } }"
  expect(() =>
    checkShapes(source, { ...metadata, effectRoots: ["original::count"] }, [
      { symbol: "original::count", require: ["self-tail-loop"] },
    ])
  ).toThrow()
  expect(() =>
    inspect(
      "const renamed = () => { const local = () => { while(true) { continue; } }; return 42; }",
      "self-tail-loop"
    )
  ).toThrow()
})
