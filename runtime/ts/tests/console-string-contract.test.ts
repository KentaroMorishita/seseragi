import { expect, test } from "bun:test"
import { createCapturedConsole } from "../src/browser/console"
import {
  type Console,
  error,
  errorLine,
  print,
  println,
  printValue,
} from "../src/console-service"
import {
  acquireRelease,
  flatMap,
  run,
  scoped,
  succeed,
  unit,
} from "../src/effect"
import { serviceFailure } from "../src/service"
import { boolShow, stringShow, unitShow } from "../src/show"

test("Console preserves String bytes, one newline and cold ordered reruns", async () => {
  const writes: string[] = []
  const environment = {
    console: createCapturedConsole((text) => writes.push(text)),
  }
  let evaluations = 0
  const text = () => {
    evaluations++
    return "雫🙂\n"
  }
  const work = flatMap(print(text()), () => println(""))
  expect(writes).toEqual([])
  expect(evaluations).toBe(1)
  for (let runIndex = 0; runIndex < 2; runIndex++) {
    expect(await run(work, environment)).toEqual({
      kind: "success",
      value: unit,
    })
  }
  expect(writes).toEqual(["雫🙂\n", "\n", "雫🙂\n", "\n"])
  expect(evaluations).toBe(1)
})

test("printValue keeps selected Show rendering separate from String output", async () => {
  const writes: string[] = []
  const environment = {
    console: createCapturedConsole((text) => writes.push(text)),
  }
  for (const work of [
    print("true"),
    printValue(true, boolShow),
    printValue(unit, unitShow),
    printValue("text", stringShow),
  ]) {
    expect((await run(work, environment)).kind).toBe("success")
  }
  expect(writes).toEqual(["true", "True", "()", "text"])
})

test("untyped Console inputs never inspect or coerce JS objects", async () => {
  let coercions = 0
  let hostCalls = 0
  const hostile = {
    toString() {
      coercions++
      throw new Error("must not inspect")
    },
    [Symbol.toPrimitive]() {
      coercions++
      throw new Error("must not coerce")
    },
  }
  const invalid: unknown[] = [
    true,
    42,
    1.5,
    undefined,
    null,
    [1],
    hostile,
    () => "text",
  ]
  const host = createCapturedConsole(() => hostCalls++)
  for (const output of [print, println, error, errorLine]) {
    for (const value of invalid) {
      // Exercise JavaScript callers beyond the statically checked String ABI.
      const work = Reflect.apply(output, undefined, [value])
      expect(coercions).toBe(0)
      expect(hostCalls).toBe(0)
      await expect(run(work, { console: host })).rejects.toBeInstanceOf(
        TypeError
      )
    }
  }
  expect(coercions).toBe(0)
  expect(hostCalls).toBe(0)
})

test("a raw String boundary Defect still releases resources before rejecting", async () => {
  const events: string[] = []
  const bad = Reflect.apply(println, undefined, [{ toString: () => "wrong" }])
  const work = scoped(
    flatMap(
      acquireRelease(succeed("resource"), () => async () => {
        events.push("release")
        return unit
      }),
      () => bad
    )
  )
  expect(events).toEqual([])
  await expect(
    run(work, { console: createCapturedConsole(() => events.push("write")) })
  ).rejects.toBeInstanceOf(TypeError)
  expect(events).toEqual(["release"])
})

test("host ConsoleError and host Defect retain distinct failure channels", async () => {
  const host = createCapturedConsole(() => {})
  const expected = { kind: "console-error", message: "typed" } as const
  const typed: Console = { ...host, print: () => serviceFailure(expected) }
  expect(await run(print("text"), { console: typed })).toEqual({
    kind: "failure",
    error: expected,
  })
  const defect = new Error("host defect")
  await expect(
    run(print("text"), {
      console: {
        ...host,
        print: () => {
          throw defect
        },
      },
    })
  ).rejects.toBe(defect)
})
