import { describe, expect, test } from "bun:test"
import { samples } from "../src/samples"

type ProjectResponse = Readonly<{
  readonly status: "success" | "failure"
  readonly problems?: readonly unknown[]
  readonly diagnostics?: readonly {
    readonly diagnostics: {
      readonly diagnostics: readonly {
        readonly code: string
        readonly messageKey: string
        readonly related: readonly { readonly message: string }[]
      }[]
    }
  }[]
}>

type FormatResponse = Readonly<{
  readonly status: "success" | "failure"
  readonly source?: string
  readonly diagnostics?: unknown
}>

type WasmBindings = Readonly<{
  readonly default: (input: {
    readonly module_or_path: ArrayBuffer
  }) => Promise<unknown>
  readonly compile_project: (request: string) => string
  readonly format_project_file: (request: string, path: string) => string
}>

let bindings: WasmBindings | undefined

async function loadBindings(): Promise<WasmBindings> {
  if (bindings !== undefined) return bindings
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

function requestFor(sample: (typeof samples)[number]) {
  return {
    schema: 1,
    entry: sample.workspace.entryFile,
    files: sample.workspace.files.map(({ path, source }) => ({ path, source })),
  } as const
}

describe("canonical sample compiler gate", () => {
  for (const sample of samples) {
    test(`compiles and formats ${sample.id}`, async () => {
      const wasm = await loadBindings()
      const request = requestFor(sample)
      const compiled = JSON.parse(
        wasm.compile_project(JSON.stringify(request))
      ) as ProjectResponse

      expect(compiled.status).toBe("success")
      if (compiled.status !== "success") {
        throw new Error(
          `sample ${sample.id} failed to compile: ${JSON.stringify(compiled)}`
        )
      }

      for (const file of request.files) {
        const formatted = JSON.parse(
          wasm.format_project_file(JSON.stringify(request), file.path)
        ) as FormatResponse
        expect(formatted.status).toBe("success")
        if (formatted.status !== "success") {
          throw new Error(
            `sample ${sample.id}/${file.path} failed to format: ${JSON.stringify(formatted)}`
          )
        }
        expect(formatted.source).toBe(file.source)
      }
    })
  }

  test("rejects a broken browser-interactive source", async () => {
    const wasm = await loadBindings()
    const broken = {
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "main.ssrg",
          source:
            'import * as dom from "std/web/dom"\n\npub effect fn main with Dom = do {\n',
        },
      ],
    } as const
    const compiled = JSON.parse(
      wasm.compile_project(JSON.stringify(broken))
    ) as ProjectResponse

    expect(compiled.status).toBe("failure")
  })

  test("reports parameterized compact failure conflicts through WASM", async () => {
    const wasm = await loadBindings()
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/fixtures/compile/effect-compact-parameterized-failure-conflict.ssrg",
        import.meta.url
      )
    ).text()
    const compiled = JSON.parse(
      wasm.compile_project(
        JSON.stringify({
          schema: 1,
          entry: "main.ssrg",
          files: [{ path: "main.ssrg", source }],
        })
      )
    ) as ProjectResponse

    expect(compiled.status).toBe("failure")
    const diagnostic = compiled.diagnostics?.[0]?.diagnostics.diagnostics[0]
    expect(diagnostic?.code).toBe("SES-E0001")
    expect(diagnostic?.messageKey).toBe("effect.compact-failure-conflict")
    expect(diagnostic?.related.map(({ message }) => message)).toEqual([
      "operation can fail with DomError",
      "operation can fail with DomRuntimeError<Never>",
    ])
  })

  test("reports explicit success and environment mismatches through WASM", async () => {
    const wasm = await loadBindings()
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/effect-explicit-contract-mismatch/main.ssrg",
        import.meta.url
      )
    ).text()
    const compiled = JSON.parse(
      wasm.compile_project(
        JSON.stringify({
          schema: 1,
          entry: "main.ssrg",
          files: [{ path: "main.ssrg", source }],
        })
      )
    ) as ProjectResponse

    expect(compiled.status).toBe("failure")
    expect(
      compiled.diagnostics?.[0]?.diagnostics.diagnostics.map(
        ({ messageKey }) => messageKey
      )
    ).toEqual([
      "effect.explicit-success-mismatch",
      "effect.explicit-environment-mismatch",
      "effect.explicit-environment-mismatch",
    ])
  })
})
