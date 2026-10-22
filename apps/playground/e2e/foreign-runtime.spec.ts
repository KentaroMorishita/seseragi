import { expect, test } from "@playwright/test"

test("browser executes a shared foreign binding through declared browser package export", async ({
  page,
}) => {
  await page.goto("/")
  await expect(page.locator("#run-button")).toBeEnabled()
  const result = await page.evaluate(async () => {
    const load = (path: string) => import(/* @vite-ignore */ path)
    const compiler = await load("/src/compiler/wasm-driver.ts")
    const { prepareBrowserForeignRuntime } = await load(
      "/src/runtime/foreign-runtime.ts"
    )
    const { startIsolatedProject } = await load(
      "/src/runtime/isolated-execution.ts"
    )
    const manifest =
      '[package]\nname="fixture/browser-linkage"\nversion="1.0.0"\nlanguage="^0.1.0"\n[run]\nentry="main"\n[foreign.typescript]\nresolver="node"\nmanifest="host/package.json"\nlockfile="host/bun.lock"\n'
    const source =
      'import * as effects from "std/effect"\nforeign "typescript" from "browser-api" {\n pure fn label value: String -> String\n}\nlet value = label "hello"\npub effect fn main -> Unit = effects.succeed ()\n'
    const compiled = await compiler.compileProject({
      schema: 1,
      manifest,
      files: [{ path: "main.ssrg", source }],
    })
    if (compiled.status !== "success") throw new Error(JSON.stringify(compiled))
    const modules = compiled.modules.map(
      ({
        path,
        generated,
      }: {
        path: string
        generated: { typescript: string }
      }) => ({ path, typescript: generated.typescript })
    )
    const files = [
      {
        path: "host/node_modules/browser-api/package.json",
        source:
          '{"name":"browser-api","version":"1.2.3","exports":{"browser":"./web.js","node":"./node.js"}}',
      },
      {
        path: "host/node_modules/browser-api/web.js",
        source:
          'import {suffix} from "./shared.js"; export function label(value) {globalThis.__interopBrowserObserved = value + suffix; return value + suffix;}',
      },
      {
        path: "host/node_modules/browser-api/shared.js",
        source: 'export const suffix = "-browser";',
      },
      {
        path: "host/node_modules/browser-api/node.js",
        source: 'import fs from "node:fs"; export const label = fs.readFile;',
      },
    ]
    const runtime = await prepareBrowserForeignRuntime(
      modules,
      files,
      manifest,
      compiler.resolveBrowserRuntimeModules
    )
    const frame = document.createElement("iframe")
    frame.name = "foreign-boundary"
    document.body.append(frame)
    const execution = await startIsolatedProject(
      frame,
      {
        modules,
        entryPath: compiled.entry.path,
        entry: compiled.entry.contract,
        input: "",
        foreignRuntime: runtime,
        initialHref: new URL("/runtime-preview.html", location.href).href,
      },
      { signal: new AbortController().signal }
    )
    await execution.completion
    const observed = (
      globalThis as unknown as { __interopBrowserObserved?: string }
    ).__interopBrowserObserved
    delete (globalThis as unknown as { __interopBrowserObserved?: string })
      .__interopBrowserObserved
    const unsupported = files.map((file) =>
      file.path.endsWith("web.js")
        ? {
            ...file,
            source:
              'globalThis.__interopBrowserObserved = "executed"; import fs from "node:fs"; export const label = fs.readFile;',
          }
        : file
    )
    let error = ""
    try {
      await prepareBrowserForeignRuntime(
        modules,
        unsupported,
        manifest,
        compiler.resolveBrowserRuntimeModules
      )
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught)
    }
    return {
      observed,
      identity:
        runtime.modules["host/node_modules/browser-api/web.js"].exactIdentity,
      error,
      sideEffect: (
        globalThis as unknown as { __interopBrowserObserved?: string }
      ).__interopBrowserObserved,
    }
  })
  expect(result.observed).toBeUndefined()
  const child = page
    .frames()
    .find((frame) => frame.name() === "foreign-boundary")!
  expect(
    await child.evaluate(
      () =>
        (globalThis as unknown as { __interopBrowserObserved?: string })
          .__interopBrowserObserved
    )
  ).toBe("hello-browser")
  expect(result.identity).toBe("npm:browser-api@1.2.3/web.js")
  expect(result.error).toContain("SES-F0203")
  expect(result.error).toContain("node:fs")
  expect(result.sideEffect).toBeUndefined()
})

test("Playground Run connects a workspace package and reports host mismatch before execution", async ({
  page,
}) => {
  const manifest =
    '[package]\nname="fixture/browser-linkage-ui"\nversion="1.0.0"\nlanguage="^0.1.0"\n[run]\nentry="main"\n[foreign.typescript]\nresolver="node"\nmanifest="host/package.json"\nlockfile="host/bun.lock"\n'
  const source =
    'import * as effects from "std/effect"\nforeign "typescript" from "browser-api" {\n task fn label value: String -> String\n}\ntype AppError deriving Show =\n | ConsoleFailure ConsoleError\n | ForeignFailure Js.Error\npub effect fn main -> Unit\nwith Console\nfails AppError = do {\n value <- label "hello" |> effects.mapError ForeignFailure\n println value |> effects.mapError ConsoleFailure\n}\n'
  const files = [
    { path: "main.ssrg", source },
    { path: "seseragi.toml", source: manifest },
    {
      path: "host/node_modules/browser-api/package.json",
      source:
        '{"name":"browser-api","version":"1.2.3","exports":{"browser":"./web.js","node":"./node.js"}}',
    },
    {
      path: "host/node_modules/browser-api/web.js",
      source:
        'import {suffix} from "./shared.js"; export function label(value) {globalThis.__interopRunObserved = value + suffix; return value + suffix;}',
    },
    {
      path: "host/node_modules/browser-api/shared.js",
      source: 'export const suffix = "-browser";',
    },
    {
      path: "host/node_modules/browser-api/node.js",
      source: 'import fs from "node:fs"; export const label = fs.readFile;',
    },
  ]
  await page.addInitScript(
    (seed) => {
      if (window !== window.top) return
      localStorage.setItem(
        "seseragi.playground.workspace.schema-1",
        JSON.stringify(seed)
      )
    },
    {
      schema: 1,
      sampleId: "playground-blank",
      sampleHash: "workspace:blank-v1",
      stdin: "",
      workspace: {
        files,
        folders: [],
        dirtyFiles: [],
        entryFile: "main.ssrg",
        activeFile: "main.ssrg",
        packageManifest: manifest,
        packageEntryFile: "main.ssrg",
        openFiles: ["main.ssrg"],
        expandedFolders: [
          "host",
          "host/node_modules",
          "host/node_modules/browser-api",
        ],
        explorer: { visible: true, width: 280 },
      },
    }
  )
  await page.goto("/")
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("hello-browser")
  await page
    .locator(
      '[data-explorer-path="host/node_modules/browser-api/web.js"] .explorer-row-label'
    )
    .click()
  await page
    .locator(".cm-content")
    .fill(
      'globalThis.__interopRunObserved = "invalid side effect"; import fs from "node:fs"; export const label = fs.readFile;'
    )
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toContainText("SES-F0203")
  await expect(page.locator("#output")).toContainText("node:fs")
  expect(
    await page.evaluate(
      () =>
        (globalThis as unknown as { __interopRunObserved?: string })
          .__interopRunObserved
    )
  ).toBeUndefined()
})
