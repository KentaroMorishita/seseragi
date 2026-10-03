import { expect, test } from "bun:test"
import type { CompileResponse } from "../src/compiler/types"
import { executeGeneratedModule } from "../src/runtime/browser-execution"
import init, { compile_single_file } from "../src/wasm/pkg/seseragi_wasm"

test("executes canonical Float rounding boundaries through WASM and the shared runtime", async () => {
  const fixture = new URL(
    "../../../examples/spec/fixtures/compile/float-rounding-boundaries.ssrg",
    import.meta.url
  )
  const source = await Bun.file(fixture).text()
  const expected = await Bun.file(
    new URL("./float-rounding-boundaries.stdout", fixture)
  ).text()
  await init({
    module_or_path: await Bun.file(
      new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
    ).arrayBuffer(),
  })
  const response = JSON.parse(
    compile_single_file(
      "float-rounding-boundaries.ssrg",
      "fixture/float-rounding-boundaries",
      source
    )
  ) as CompileResponse
  expect(response.status).toBe("success")
  if (response.status !== "success" || !response.entry) {
    throw new Error("missing Float rounding execution entry")
  }
  expect(
    await executeGeneratedModule(response.generated.typescript, response.entry)
  ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
})
