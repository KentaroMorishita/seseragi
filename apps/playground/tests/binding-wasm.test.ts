import { beforeAll, expect, test } from "bun:test"
import { utf8RangeToUtf16 } from "../src/diagnostics/source-range"
import init, { convert_workspace_bindings } from "../src/wasm/pkg/seseragi_wasm"
import {
  type BindingRequest,
  type BindingResponse,
  bindingDiagnostics,
} from "../src/workspace/bindings"

beforeAll(async () => {
  await init({
    module_or_path: await Bun.file(
      new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
    ).arrayBuffer(),
  })
})
async function request(): Promise<BindingRequest> {
  const root = new URL(
    "../../../examples/spec/fixtures/projects/dts-basic-conversion/",
    import.meta.url
  )
  return {
    schema: 1,
    revision: "snapshot",
    files: await Promise.all(
      [
        "seseragi.toml",
        "seseragi.bindings.toml",
        "host/package.json",
        "host/index.d.ts",
      ].map(async (path) => ({
        path,
        source: await Bun.file(new URL(path, root)).text(),
      }))
    ),
  }
}
const convert = (input: BindingRequest): BindingResponse =>
  JSON.parse(convert_workspace_bindings(JSON.stringify(input)))

test("committed WASM returns generated artifacts and preserves a revision", async () => {
  const input = await request()
  const result = convert(input)
  expect(result.status).toBe("success")
  expect(result.revision).toBe(input.revision)
  expect(result.generated[0]?.source).toContain("foreign")
  const again = convert({
    ...input,
    files: [
      ...input.files,
      {
        path: ".seseragi/generated/fixture-api.binding.json",
        source: result.generated[0]?.metadata ?? "",
      },
    ],
  })
  expect(JSON.parse(again.generated[0]?.report ?? "{}").added).toEqual([])
})

test("a non-ASCII declaration diagnostic selects the exact source span", async () => {
  const input = await request()
  const source =
    "// 🙂 café\nexport declare function unsafe(value: any): string;"
  const snapshot = {
    ...input,
    files: input.files.map((file) =>
      file.path === "host/index.d.ts" ? { ...file, source } : file
    ),
  }
  const result = convert(snapshot)
  expect(result.status).toBe("failure")
  const diagnostic = bindingDiagnostics(snapshot, result)[0]
  expect(diagnostic?.path).toBe("host/index.d.ts")
  expect(diagnostic?.source).toBe(source)
  if (!diagnostic) throw new Error("missing diagnostic")
  const range = utf8RangeToUtf16(source, diagnostic.diagnostic.primary)
  expect(source.slice(range.from, range.to)).toBe("any")
})

test("resolved dependency metadata errors point to the artifact that failed", async () => {
  const input = await request()
  const dependency = "host/node_modules/fixture-api/package.json"
  for (const contents of [
    "{",
    '{"name":"wrong","version":"1.0.0"}',
    '{"name":"fixture-api"}',
  ]) {
    const result = convert({
      ...input,
      files: [
        ...input.files.map((file) =>
          file.path === "host/package.json"
            ? { ...file, source: '{"name":"app"}' }
            : file
        ),
        { path: dependency, source: contents },
      ],
    })
    expect(result.status).toBe("failure")
    expect(result.diagnostics[0]?.file).toBe(dependency)
  }
  const badRoot = convert({
    ...input,
    files: input.files.map((file) =>
      file.path === "host/package.json" ? { ...file, source: "{" } : file
    ),
  })
  expect(badRoot.diagnostics[0]?.file).toBe("host/package.json")
})
