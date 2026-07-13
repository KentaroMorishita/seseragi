import { describe, expect, test } from "bun:test"
import { executeGeneratedModule } from "../playground/src/lib/browser-execution"
import type { EntryContract } from "../playground/src/lib/playground-types"

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

type SuccessResponse = {
  readonly status: "success"
  readonly generated: { readonly typescript: string }
  readonly entry: EntryContract
}

describe("Playground-0 WASM boundary", () => {
  test("compiles and executes lesson 01 through the browser host", async () => {
    const bindingsUrl = new URL(
      "../playground/src/wasm/pkg/seseragi_wasm.js",
      import.meta.url
    ).href
    const bindings = (await import(bindingsUrl)) as WasmBindings
    const wasm = await Bun.file(
      new URL(
        "../playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
        import.meta.url
      )
    ).arrayBuffer()
    await bindings.default({ module_or_path: wasm })
    const source = await Bun.file(
      new URL("../examples/spec/lessons/01-hello-world.ssrg", import.meta.url)
    ).text()
    const compiled = JSON.parse(
      bindings.compile_single_file(
        "lesson-01.ssrg",
        "playground/lesson-01",
        source
      )
    ) as SuccessResponse

    expect(compiled.status).toBe("success")
    expect(
      await executeGeneratedModule(
        compiled.generated.typescript,
        compiled.entry
      )
    ).toBe("Hello, Seseragi!")

    const phaseOneSource = await Bun.file(
      new URL(
        "../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg",
        import.meta.url
      )
    ).text()
    const phaseOne = JSON.parse(
      bindings.compile_single_file(
        "rock-paper-scissors.ssrg",
        "playground/rock-paper-scissors",
        phaseOneSource
      )
    ) as SuccessResponse
    expect(
      await executeGeneratedModule(
        phaseOne.generated.typescript,
        phaseOne.entry,
        "rock\nscissors\n"
      )
    ).toBe("Player 1 wins!")
  })
})
