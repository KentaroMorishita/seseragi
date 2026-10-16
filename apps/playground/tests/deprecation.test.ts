import { beforeAll, expect, test } from "bun:test"
import type { CompileResponse } from "../src/compiler/types"
import { executeGeneratedModule } from "../src/runtime/browser-execution"
import init, {
  compile_single_file,
  format_single_file,
} from "../src/wasm/pkg/seseragi_wasm"

beforeAll(async () => {
  await init({
    module_or_path: await Bun.file(
      new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
    ).arrayBuffer(),
  })
})

test("deprecated top-level declarations survive compilation and formatting", async () => {
  for (const separator of [" ", "\n", "\n// visibility\n"]) {
    const source = `pub deprecated "Use replacement" since "0.4.1"${separator}fn historical value: Int -> Int = value + 1\npub effect fn main = println (show (historical 41))\n`
    const response = JSON.parse(
      compile_single_file("λ.ssrg", "fixture/deprecated", source)
    ) as CompileResponse
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing deprecated function execution entry")
    }
    expect(response.generated.typescript).toContain("historical")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: "42", debug: "()" })
    const formatted = JSON.parse(format_single_file("λ.ssrg", source))
    expect(formatted.status).toBe("success")
    expect(
      JSON.parse(format_single_file("λ.ssrg", formatted.source)).changed
    ).toBe(false)
    const unused = source.replace("historical 41", "42")
    const retained = JSON.parse(
      compile_single_file("λ.ssrg", "fixture/deprecated", unused)
    ) as CompileResponse
    expect(retained.status).toBe("success")
    if (retained.status === "success") {
      expect(retained.generated.typescript).toContain("historical")
    }
  }
})

test("invalid deprecated prefixes fail with UTF-8 ranges before generation", () => {
  for (const clause of [
    'deprecated ""',
    "deprecated 41",
    'deprecated "old" since "01.2.3"',
    'deprecated "old" since "v1.2.3"',
    'deprecated "old" since',
    'deprecated "old";',
    'deprecated "old" pub',
  ]) {
    const source = `// λ\n${clause}\nfn historical -> Int = 41\npub effect fn main = println "must not run"\n`
    const response = JSON.parse(
      compile_single_file("λ.ssrg", "fixture/deprecated", source)
    ) as CompileResponse
    expect(response.status).toBe("failure")
    expect("generated" in response).toBe(false)
    expect(
      response.diagnostics.diagnostics.some((d) => d.code === "SES-P0001")
    ).toBe(true)
    const bytes = new TextEncoder().encode(source)
    for (const diagnostic of response.diagnostics.diagnostics) {
      const { start, end } = diagnostic.primary
      expect(start).toBeGreaterThanOrEqual(0)
      expect(end).toBeLessThanOrEqual(bytes.length)
      expect(() =>
        new TextDecoder("utf-8", { fatal: true }).decode(
          bytes.slice(start, end)
        )
      ).not.toThrow()
    }
    expect(JSON.parse(format_single_file("λ.ssrg", source)).status).toBe(
      "failure"
    )
  }
})
