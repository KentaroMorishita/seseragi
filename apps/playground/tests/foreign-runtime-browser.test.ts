import { beforeAll, expect, test } from "bun:test"
import type { ProjectCompileResponse } from "../src/compiler/types"
import { executeGeneratedProject } from "../src/runtime/browser-execution"
import {
  type BrowserRuntimeResponse,
  prepareBrowserForeignRuntime,
} from "../src/runtime/foreign-runtime"
import init, {
  compile_project,
  resolve_browser_runtime_modules,
} from "../src/wasm/pkg/seseragi_wasm"

beforeAll(async () => {
  await init({
    module_or_path: await Bun.file(
      new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
    ).arrayBuffer(),
  })
})
const manifest =
  '[package]\nname="fixture/browser-foreign"\nversion="1.0.0"\nlanguage="^0.1.0"\n[run]\nentry="main"\n[foreign.typescript]\nresolver="node"\nmanifest="host/package.json"\nlockfile="host/bun.lock"\n'
const source =
  'import * as effects from "std/effect"\nforeign "typescript" from "browser-api" {\n pure fn label value: String -> String\n}\nlet result = label "hello"\npub effect fn main -> Unit = effects.succeed ()\n'
const compile = (
  inputSource = source
): Extract<ProjectCompileResponse, { status: "success" }> => {
  const compiled = JSON.parse(
    compile_project(
      JSON.stringify({
        schema: 1,
        manifest,
        files: [{ path: "main.ssrg", source: inputSource }],
      })
    )
  ) as ProjectCompileResponse
  if (compiled.status !== "success") throw new Error(JSON.stringify(compiled))
  return compiled
}
const resolve = async (request: unknown): Promise<BrowserRuntimeResponse> =>
  JSON.parse(resolve_browser_runtime_modules(JSON.stringify(request)))
const files = [
  {
    path: "host/node_modules/browser-api/package.json",
    source:
      '{"name":"browser-api","version":"1.2.3","exports":{"browser":"./web.js","node":"./node.js"}}',
  },
  {
    path: "host/node_modules/browser-api/web.js",
    source:
      'import {suffix} from "./shared.js"; export function label(value) {globalThis.__browserForeignObserved = value + suffix; return value + suffix;}',
  },
  {
    path: "host/node_modules/browser-api/shared.js",
    source: 'export const suffix = "-browser";',
  },
  {
    path: "host/node_modules/browser-api/node.js",
    source:
      'import fs from "node:fs"; export function label(value) {return fs.readFileSync(value);}',
  },
]
test("ordinary compiled foreign binding resolves browser package and transitive imports before execution", async () => {
  const compiled = compile()
  const modules = compiled.modules.map(({ path, generated }) => ({
    path,
    typescript: generated.typescript,
  }))
  const foreignRuntime = await prepareBrowserForeignRuntime(
    modules,
    files,
    manifest,
    resolve
  )
  expect(Object.keys(foreignRuntime.modules)).toHaveLength(2)
  expect(
    foreignRuntime.modules["host/node_modules/browser-api/web.js"]
      ?.exactIdentity
  ).toBe("npm:browser-api@1.2.3/web.js")
  const result = await executeGeneratedProject(
    modules,
    compiled.entry.path,
    compiled.entry.contract!,
    "",
    { foreignRuntime }
  )
  expect(result.stdout).toBe("")
  expect(
    (globalThis as unknown as { __browserForeignObserved: string })
      .__browserForeignObserved
  ).toBe("hello-browser")
  delete (globalThis as unknown as { __browserForeignObserved?: string })
    .__browserForeignObserved
})
test("browser host mismatch fails before foreign module side effects", async () => {
  const compiled = compile()
  const modules = compiled.modules.map(({ path, generated }) => ({
    path,
    typescript: generated.typescript,
  }))
  const unsupported = files.map((file) =>
    file.path.endsWith("web.js")
      ? {
          ...file,
          source:
            'globalThis.__browserForeignObserved = "executed"; import fs from "node:fs"; export function label(x) {return x;}',
        }
      : file
  )
  await expect(
    prepareBrowserForeignRuntime(modules, unsupported, manifest, resolve)
  ).rejects.toThrow("SES-F0203")
  expect(
    (globalThis as unknown as { __browserForeignObserved?: string })
      .__browserForeignObserved
  ).toBeUndefined()
})
test("computed imports cannot escape runtime identity preflight", async () => {
  await expect(
    prepareBrowserForeignRuntime(
      [
        {
          path: "main.ssrg",
          typescript: 'const runtime = "browser-api"; import(runtime);',
        },
      ],
      files,
      manifest,
      resolve
    )
  ).rejects.toThrow("unresolved exact identity")
})

test("task module cache invalidates when only its transitive dependency changes", async () => {
  const inputSource =
    'import * as effects from "std/effect"\nforeign "typescript" from "browser-api" {\n task fn label value: String -> String\n}\ntype AppError deriving Show =\n | ConsoleFailure ConsoleError\n | ForeignFailure Js.Error\npub effect fn main -> Unit\nwith Console\nfails AppError = do {\n value <- label "hello" |> effects.mapError ForeignFailure\n println value |> effects.mapError ConsoleFailure\n}\n'
  const compiled = compile(inputSource)
  const modules = compiled.modules.map(({ path, generated }) => ({
    path,
    typescript: generated.typescript,
  }))
  const initial = await prepareBrowserForeignRuntime(
    modules,
    files,
    manifest,
    resolve
  )
  const first = await executeGeneratedProject(
    modules,
    compiled.entry.path,
    compiled.entry.contract!,
    "",
    { foreignRuntime: initial }
  )
  expect(first.stdout).toBe("hello-browser")
  const changedFiles = files.map((file) =>
    file.path.endsWith("shared.js")
      ? { ...file, source: 'export const suffix = "-updated";' }
      : file
  )
  const changed = await prepareBrowserForeignRuntime(
    modules,
    changedFiles,
    manifest,
    resolve
  )
  expect(changed.snapshotDigest).not.toBe(initial.snapshotDigest)
  const second = await executeGeneratedProject(
    modules,
    compiled.entry.path,
    compiled.entry.contract!,
    "",
    { foreignRuntime: changed }
  )
  expect(second.stdout).toBe("hello-updated")
  delete (globalThis as unknown as { __browserForeignObserved?: string })
    .__browserForeignObserved
})

test("unused generated declarations do not require an executable runtime package", async () => {
  const modules = [
    {
      path: "generated/api.ssrg",
      typescript:
        'import * as api from "absent-api"; export const call = () => api.call();',
    },
    {
      path: "main.ssrg",
      typescript:
        'import {call} from "./generated/api.ts"; export const value = 1;',
    },
  ]
  const bundle = await prepareBrowserForeignRuntime(
    modules,
    [],
    undefined,
    resolve,
    "main.ssrg"
  )
  expect(bundle.generatedPaths).toEqual(["main.js"])
  expect(Object.keys(bundle.modules)).toEqual([])
  expect(Object.keys(bundle.roots)).toEqual([])
})
