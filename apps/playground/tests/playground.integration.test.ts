import { describe, expect, test } from "bun:test"
import type { CompileResponse } from "../src/compiler/types"
import { executeGeneratedModule } from "../src/runtime/browser-execution"

type WasmBindings = {
  readonly default: (input: {
    readonly module_or_path: ArrayBuffer
  }) => Promise<unknown>
  readonly compile_single_file: (
    sourceName: string,
    moduleId: string,
    source: string
  ) => string
}

let bindings: WasmBindings | undefined

async function loadBindings(): Promise<WasmBindings> {
  if (bindings) return bindings
  const bindingsUrl = new URL(
    "../src/wasm/pkg/seseragi_wasm.js",
    import.meta.url
  ).href
  bindings = (await import(bindingsUrl)) as WasmBindings
  const wasm = await Bun.file(
    new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
  ).arrayBuffer()
  await bindings.default({ module_or_path: wasm })
  return bindings
}

async function compile(
  sourceName: string,
  source: string
): Promise<CompileResponse> {
  const wasm = await loadBindings()
  return JSON.parse(
    wasm.compile_single_file(sourceName, `playground/${sourceName}`, source)
  ) as CompileResponse
}

describe("new Playground product gate", () => {
  test("executes lesson 01 through the shared driver and browser runtime", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/lessons/01-hello-world.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compile("hello-world.ssrg", source)

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing entry")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({
      stdout: "Hello, Seseragi!",
    })
  })

  test("passes user-provided stdin to the cumulative Phase 1 program", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compile("rock-paper-scissors.ssrg", source)

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing entry")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry,
        "rock\nscissors\n"
      )
    ).toEqual({ stdout: "Player 1 wins!" })
  })

  test("returns structured diagnostics for invalid source", async () => {
    const response = await compile("broken.ssrg", "pub let broken: Int =\n")
    expect(response.status).toBe("failure")
    expect(response.diagnostics.diagnostics.length).toBeGreaterThan(0)
    expect(response.diagnostics.diagnostics[0]?.primary).toBeDefined()
  })
})
