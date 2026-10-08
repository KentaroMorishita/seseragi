import { beforeAll, expect, test } from "bun:test"
import { utf8RangeToUtf16 } from "../src/diagnostics/source-range"
import init, {
  analyze_project,
  compile_project,
  convert_workspace_bindings,
} from "../src/wasm/pkg/seseragi_wasm"
import {
  type BindingRequest,
  type BindingResponse,
  bindingDiagnostics,
  publishWorkspaceBindings,
  workspaceBindingRequest,
} from "../src/workspace/bindings"
import {
  createWorkspace,
  updateWorkspaceFileSource,
} from "../src/workspace/model"
import { workspaceProjectRequest } from "../src/workspace/project-request"

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

test("converted bindings compile and analyze through the ordinary generated module graph", async () => {
  const inputs = await request()
  const state = createWorkspace({
    files: [
      ...inputs.files.map((file) =>
        file.path === "seseragi.toml"
          ? {
              ...file,
              source: `${file.source}\n[layout]\ngenerated = "custom/bindings"\n`,
            }
          : file
      ),
      {
        path: "main.ssrg",
        source:
          'import * as api from "gen/fixture-api"\npub fn identity config: api.Config -> api.Config = config\n\npub effect fn main -> Unit\nwith Console\nfails ConsoleError = println "generated"\n',
      },
    ],
    entryFile: "main.ssrg",
    activeFile: "main.ssrg",
    openFiles: ["main.ssrg"],
  })
  const input = workspaceBindingRequest(state)
  const response = convert(input)
  expect(response.status).toBe("success")
  expect(response.generatedRoot).toBe("custom/bindings")
  const published = publishWorkspaceBindings(state, input, response)
  const project = workspaceProjectRequest(published)
  const compiled = JSON.parse(compile_project(JSON.stringify(project)))
  const analyzed = JSON.parse(analyze_project(JSON.stringify(project)))
  expect(compiled.status).toBe("success")
  expect(analyzed.status).toBe("success")
  expect(
    analyzed.documents.some(
      (document: { path: string }) =>
        document.path === "custom/bindings/fixture-api.ssrg"
    )
  ).toBe(true)
  const stale = workspaceProjectRequest(
    updateWorkspaceFileSource(
      published,
      "host/index.d.ts",
      "export declare function replaced(): string;"
    )
  )
  const missing = JSON.parse(compile_project(JSON.stringify(stale)))
  expect(missing.status).toBe("failure")
  expect(missing.problems[0].code).toBe("SES-N0104")
})

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

test("Inspector consumes core metadata and reports, rejects stale results and tracks input revisions", async () => {
  const { inspectWorkspaceBindings, currentBindingInspection } = await import(
    "../src/workspace/binding-inspection"
  )
  const { renameWorkspacePath, deleteWorkspacePath } = await import(
    "../src/workspace/model"
  )
  const input = await request()
  const state = createWorkspace({
    files: [...input.files, { path: "main.ssrg", source: "pub let x = 1\n" }],
    entryFile: "main.ssrg",
    activeFile: "main.ssrg",
    openFiles: ["main.ssrg"],
  })
  const snapshot = workspaceBindingRequest(state)
  const result = convert(snapshot)
  const inspection = inspectWorkspaceBindings(state, snapshot, result)
  expect(inspection.bindings[0]?.declaration).toBe("host/index.d.ts")
  expect(inspection.bindings[0]?.path).toBe(
    ".seseragi/generated/fixture-api.ssrg"
  )
  expect(inspection.bindings[0]?.source).toContain("Config")
  expect(inspection.bindings[0]?.report.added).toContain(
    "fixture-api::Config::type"
  )
  expect(inspection.inputs.some((file) => file.path.endsWith(".ssrg"))).toBe(
    false
  )
  expect(
    currentBindingInspection(
      inspection,
      updateWorkspaceFileSource(state, "main.ssrg", "pub let x = 2\n")
    )
  ).toBe(inspection)
  for (const changed of [
    updateWorkspaceFileSource(state, "host/index.d.ts", "changed"),
    renameWorkspacePath(state, "host/index.d.ts", "host/new.d.ts"),
    deleteWorkspacePath(state, "host/index.d.ts"),
  ]) {
    expect(currentBindingInspection(inspection, changed)).toBeUndefined()
    expect(() => inspectWorkspaceBindings(changed, snapshot, result)).toThrow(
      "snapshot"
    )
  }
  expect(() =>
    inspectWorkspaceBindings(state, snapshot, { ...result, revision: "stale" })
  ).toThrow("snapshot")
  const generated = result.generated[0]!
  expect(() =>
    inspectWorkspaceBindings(state, snapshot, {
      ...result,
      generated: [
        {
          ...generated,
          report: JSON.stringify({
            ...JSON.parse(generated.report),
            entry: "wrong",
          }),
        },
      ],
    })
  ).toThrow("entry")
})
