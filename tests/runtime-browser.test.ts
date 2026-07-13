import { describe, expect, test } from "bun:test"
import { createBrowserEnvironment } from "../runtime/ts/src/browser/host"
import {
  println,
  type ConsoleEnvironment,
} from "../runtime/ts/src/browser/console"
import {
  readLine,
  type StdinEnvironment,
} from "../runtime/ts/src/browser/stdin"
import { run } from "../runtime/ts/src/effect"

describe("browser runtime host", () => {
  test("captures Console output through the shared Effect runner", async () => {
    let output = ""
    const environment = createBrowserEnvironment(
      [{ field: "console", service: "console" }],
      "",
      (value) => {
        output += value
      }
    ) as ConsoleEnvironment

    expect(await run(println("Hello, Seseragi!"), environment)).toEqual({
      kind: "success",
      value: undefined,
    })
    expect(output).toBe("Hello, Seseragi!\n")
  })

  test("provides deterministic input lines and sticky EOF", async () => {
    const environment = createBrowserEnvironment(
      [{ field: "stdin", service: "stdin" }],
      "rock\nscissors\n",
      () => {}
    ) as StdinEnvironment

    expect(await run(readLine(), environment)).toEqual({
      kind: "success",
      value: { tag: "Just", value: "rock" },
    })
    expect(await run(readLine(), environment)).toEqual({
      kind: "success",
      value: { tag: "Just", value: "scissors" },
    })
    expect(await run(readLine(), environment)).toEqual({
      kind: "success",
      value: { tag: "Nothing" },
    })
  })
})
