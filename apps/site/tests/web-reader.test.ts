import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import * as ts from "typescript"
import {
  type Effect,
  run,
  type Unit,
  unit,
} from "../../../runtime/ts/src/effect"
import {
  type ServiceResult,
  serviceFailure,
  serviceSuccess,
} from "../../../runtime/ts/src/service"
import type { Signal } from "../../../runtime/ts/src/signal"
import {
  webReaderCases,
  webReaderExamples,
  webReaderFailures,
  webReaderRoutes,
} from "../scripts/web-reader"

// Load the execution-only HTML/DOM boundary like the other reader tests.
// This keeps the site compiler's closure out of the entire runtime registry.
// These opaque handles are never constructed or inspected by the tests.
type Html<Action> = object & { readonly __action?: Action }
type DomTarget = object
type DomMount<Failure> = object & { readonly __failure?: Failure }
type DomOptions = unknown
type DomError = Readonly<{
  tag: "DomTargetNotFound" | "DomOperationFailed"
  value: string
}>
type DomDispatch<Failure, Action> = (
  action: Action
) => Promise<ServiceResult<Failure, Unit>>
interface Dom {
  query(selector: string): ServiceResult<DomError, DomTarget>
  mount<Failure, Action>(
    options: DomOptions,
    target: DomTarget,
    dispatch: DomDispatch<Failure, Action>,
    content: Signal<Html<Action>>
  ):
    | ServiceResult<DomError, DomMount<Failure>>
    | Promise<ServiceResult<DomError, DomMount<Failure>>>
  capturePointer(): never
  releasePointer(): never
  measure(): never
  observeResize(): never
}
type DomEnvironment = { readonly dom: Dom }
interface DomConstructors {
  createDomTarget(value: unknown): DomTarget
  createDomMount<Failure>(control: {
    awaitResult(): Promise<ServiceResult<unknown, Unit>>
    unmount(): Promise<void>
    bindCancellation(release: () => void): void
  }): DomMount<Failure>
}
interface HtmlRenderers {
  renderToString<Action>(value: Html<Action>): string
  renderForDom<Action>(value: Html<Action>): {
    readonly eventHandlers: ReadonlyMap<
      string,
      {
        readonly kind: string
        readonly message: Action
      }
    >
  }
}

const { createDomMount, createDomTarget } = (await import(
  new URL("../../../runtime/ts/src/dom.ts", import.meta.url).href
)) as DomConstructors
const { renderForDom, renderToString } = (await import(
  new URL("../../../runtime/ts/src/html.ts", import.meta.url).href
)) as HtmlRenderers
const { runtimeModules } = (await import(
  new URL("../../playground/src/runtime/runtime-modules.ts", import.meta.url)
    .href
)) as { readonly runtimeModules: Readonly<Record<string, unknown>> }
const { executeGeneratedModule } = (await import(
  new URL("../../playground/src/runtime/browser-execution.ts", import.meta.url)
    .href
)) as {
  executeGeneratedModule(
    source: string,
    entry: unknown
  ): Promise<{ stdout: string }>
}

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = webReaderExamples("https://seseragi.vercel.app/")
const pureCases = webReaderCases.filter((item) => item.target === "process")
const states = pureCases
  .find((item) => item.slug === "release-card-view")!
  .output.trimEnd()
  .split("\n")

function command(executable: string, args: string[], cwd = root) {
  return spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}

function sample(slug: string) {
  const result = examples.find((item) => item.id === `web-reader-${slug}`)
  if (!result) throw new Error(`missing canonical example: ${slug}`)
  return result
}

const wasm = (async () => {
  const bindings = await import(
    new URL("../../playground/src/wasm/pkg/seseragi_wasm.js", import.meta.url)
      .href
  )
  await bindings.default({
    module_or_path: await Bun.file(
      new URL(
        "../../playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
        import.meta.url
      )
    ).arrayBuffer(),
  })
  return bindings
})()

async function compile(slug: string) {
  const bindings = await wasm
  const example = sample(slug)
  const seed = new URL(example.playgroundUrl).searchParams.get("source")!
  expect(seed).toBe(example.source)
  const result = JSON.parse(
    bindings.compile_single_file("main.ssrg", "playground/main", seed)
  )
  expect(result.status, JSON.stringify(result)).toBe("success")
  expect(result.entry).toBeDefined()
  return result
}

// Evaluate exactly the generated app, with the same runtime module registry as
// the Playground. Only its Dom service is mocked: no browser is opened here.
async function compiledApp(): Promise<Effect<DomEnvironment, string, Unit>> {
  const result = await compile("release-card-app")
  expect(result.entry.environment).toEqual([{ field: "dom", service: "dom" }])
  const javascript = ts.transpileModule(result.generated.typescript, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
      strict: true,
    },
  }).outputText
  const module = { exports: {} as Record<string, unknown> }
  const requireRuntime = (specifier: string) => {
    const runtime = runtimeModules[specifier]
    if (!runtime) throw new Error(`unsupported runtime module: ${specifier}`)
    return runtime
  }
  new Function("require", "module", "exports", javascript)(
    requireRuntime,
    module,
    module.exports
  )
  expect(typeof module.exports.main).toBe("function")
  const main = module.exports.main as (
    value: Unit
  ) => Effect<DomEnvironment, string, Unit>
  return main(unit)
}

const unusedSceneOperations: Pick<
  Dom,
  "capturePointer" | "releasePointer" | "measure" | "observeResize"
> = {
  capturePointer() {
    throw new Error("capturePointer is unused")
  },
  releasePointer() {
    throw new Error("releasePointer is unused")
  },
  measure() {
    throw new Error("measure is unused")
  },
  observeResize() {
    throw new Error("observeResize is unused")
  },
}

test("fifteen existing identities use thirteen canonical sources, with no phantom application routes", () => {
  expect(webReaderRoutes).toHaveLength(15)
  expect(new Set(webReaderRoutes.map((item) => item.identity)).size).toBe(15)
  expect(webReaderCases).toHaveLength(13)
  expect(examples).toHaveLength(16)
  for (const item of webReaderRoutes) {
    expect(item.route).toStartWith("/docs/library/web/")
    expect(item.identity.split("::")[0]).toBe(item.module)
    expect(sample(item.example).source).toContain("pub effect fn main")
  }
  for (const example of examples) {
    expect(example.source).toBe(
      readFileSync(join(root, example.sourcePath), "utf8")
    )
    expect(example.highlighted.map((part) => part.text).join("")).toBe(
      example.source
    )
    if (example.id.endsWith("-ts")) expect(example.playgroundUrl).toBe("")
    else
      expect(new URL(example.playgroundUrl).searchParams.get("source")).toBe(
        example.source
      )
  }
  const pureView =
    sample("release-card-view").source.split("pub effect fn main")[0]
  const appView = sample("release-card-app")
    .source.replace('import * as dom from "std/web/dom"\n', "")
    .split("pub effect fn main")[0]
  expect(appView).toBe(pureView)
})

test("twelve isolated native programs lint, format, and execute their documented outputs", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-web-reader-native-"))
  try {
    for (const item of pureCases) {
      writeFileSync(join(directory, "main.ssrg"), sample(item.slug).source)
      for (const args of [
        ["format", "--check", "main.ssrg"],
        ["lint", "main.ssrg"],
      ]) {
        const checked = command(cli, args, directory)
        expect(
          checked.status,
          `${item.slug}: ${checked.stdout}${checked.stderr}`
        ).toBe(0)
      }
      const result = command(cli, ["run", "main.ssrg"], directory)
      expect(result.status, `${item.slug}: ${result.stderr}`).toBe(0)
      expect(result.stderr, item.slug).toBe("")
      expect(result.stdout, item.slug).toBe(item.output)
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("native diagnostics reject wrong props and children, and each named correction runs", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-web-reader-invalid-"))
  try {
    for (const item of webReaderFailures) {
      const invalid = readFileSync(
        join(root, `apps/site/tests/fixtures/web-reader/${item.slug}.ssrg`),
        "utf8"
      )
      writeFileSync(join(directory, "main.ssrg"), invalid)
      const result = command(cli, ["lint", "main.ssrg"], directory)
      expect(result.status, item.slug).toBe(2)
      for (const diagnostic of item.diagnostics)
        expect(result.stderr, item.slug).toContain(diagnostic)
      writeFileSync(
        join(directory, "main.ssrg"),
        sample(item.correction).source
      )
      const corrected = command(cli, ["run", "main.ssrg"], directory)
      expect(corrected.status, corrected.stderr).toBe(0)
      expect(corrected.stdout).toBe(
        pureCases.find((entry) => entry.slug === item.correction)!.output
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("strict TypeScript comparators preserve card output and pure collapse-expand-collapse snapshots", () => {
  const comparators = examples.filter((example) => example.id.endsWith("-ts"))
  const checked = command(join(root, "node_modules/.bin/tsc"), [
    "--noEmit",
    "--strict",
    "--skipLibCheck",
    "--target",
    "ES2022",
    "--lib",
    "ESNext,DOM",
    "--module",
    "ESNext",
    "--moduleResolution",
    "bundler",
    ...comparators.map((example) => join(root, example.sourcePath)),
  ])
  expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  for (const slug of ["cards", "release-card-view"]) {
    const result = command("bun", [join(root, sample(`${slug}-ts`).sourcePath)])
    expect(result.status, result.stderr).toBe(0)
    expect(result.stderr).toBe("")
    expect(result.stdout).toBe(
      pureCases.find((item) => item.slug === slug)!.output
    )
  }
  // The DOM comparator is typechecked, not executed in a pretend browser.
})

test("committed WASM compiles exact Playground seeds and the in-process harness runs all pure results", async () => {
  for (const item of pureCases) {
    const compiled = await compile(item.slug)
    expect(
      compiled.entry.environment.some(
        (entry: { service: string }) => entry.service === "dom"
      )
    ).toBe(false)
    const executed = await executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(executed.stdout, item.slug).toBe(item.output.trimEnd())
  }
})

test("the standalone app builds a web artifact with an app target, without serving or opening it", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-web-reader-web-"))
  try {
    writeFileSync(
      join(directory, "main.ssrg"),
      sample("release-card-app").source
    )
    const checked = command(cli, ["lint", "main.ssrg"], directory)
    expect(checked.status, checked.stderr).toBe(0)
    const built = command(
      cli,
      ["build", "main.ssrg", "--target", "web", "--out-dir", "output"],
      directory
    )
    expect(built.status, built.stdout + built.stderr).toBe(0)
    const output = join(directory, "output")
    const manifest = JSON.parse(
      readFileSync(join(output, "artifact-manifest.json"), "utf8")
    )
    expect(manifest.target).toBe("web")
    expect(manifest.entry).toBe("assets/app.js")
    expect(manifest.generatedModules).toHaveLength(1)
    expect(manifest.generatedModules[0].runtimeRequirements).toContain(
      "web.dom.app"
    )
    expect(manifest.generatedModules[0].runtimeRequirements).toContain(
      "web.dom.service"
    )
    const index = readFileSync(join(output, "index.html"), "utf8")
    expect(index).toContain('<div id="app"></div>')
    expect(index).toContain('type="module" src="./assets/app.js"')
    expect(
      readFileSync(join(output, manifest.entry), "utf8").length
    ).toBeGreaterThan(0)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("compiled app provider mock dispatches actual click actions twice and invokes owned cleanup", async () => {
  const snapshots: string[] = []
  const selectors: string[] = []
  let unmounts = 0
  const target = createDomTarget({ selector: "#app" })
  const service: Dom = {
    ...unusedSceneOperations,
    query(selector) {
      selectors.push(selector)
      return serviceSuccess(target)
    },
    async mount<Failure, Action>(
      _options: DomOptions,
      mountedTarget: DomTarget,
      dispatch: DomDispatch<Failure, Action>,
      content: Signal<Html<Action>>
    ): Promise<ServiceResult<DomError, DomMount<Failure>>> {
      expect(mountedTarget).toBe(target)
      snapshots.push(renderToString(content.current()))
      for (let click = 0; click < 2; click++) {
        const rendered = renderForDom(content.current())
        const handlers = [...rendered.eventHandlers.values()]
        expect(handlers).toHaveLength(1)
        const handler = handlers[0]
        if (handler.kind !== "click")
          throw new Error("expected ToggleDetails click")
        const dispatched = await dispatch(handler.message)
        expect(handler.message as unknown).toEqual({ tag: "ToggleDetails" })
        expect(dispatched).toEqual({
          kind: "success",
          value: unit,
        })
        snapshots.push(renderToString(content.current()))
      }
      let releaseCancellation: (() => void) | undefined
      return serviceSuccess(
        createDomMount<Failure>({
          awaitResult: async () => serviceSuccess(unit),
          async unmount() {
            unmounts += 1
            releaseCancellation?.()
            releaseCancellation = undefined
          },
          bindCancellation(release) {
            releaseCancellation = release
          },
        })
      )
    },
  }
  const effect = await compiledApp()
  expect(selectors).toEqual([]) // Constructing the Effect did not query the DOM.
  expect(await run(effect, { dom: service })).toEqual({
    kind: "success",
    value: unit,
  })
  expect(selectors).toEqual(["#app"])
  expect(snapshots).toEqual(states)
  expect(unmounts).toBe(1)
  for (const snapshot of snapshots) {
    expect(snapshot).not.toContain("ToggleDetails")
    expect(snapshot).not.toContain("onclick")
    expect(snapshot).not.toContain("data-ssrg")
  }
  // This checks provider ownership, not real browser event delivery or teardown.
})

test("compiled app provider mock exposes missing-target and mounting failures as Strings", async () => {
  let mounts = 0
  const missing: Dom = {
    ...unusedSceneOperations,
    query(selector) {
      expect(selector).toBe("#app")
      return serviceFailure({ tag: "DomTargetNotFound", value: selector })
    },
    mount() {
      mounts += 1
      throw new Error("mount must not run for a missing target")
    },
  }
  expect(await run(await compiledApp(), { dom: missing })).toEqual({
    kind: "failure",
    error: "DOM target unavailable: #app",
  })
  expect(mounts).toBe(0)
  const failedMount: Dom = {
    ...unusedSceneOperations,
    query: () => serviceSuccess(createDomTarget({ selector: "#app" })),
    mount: () =>
      serviceFailure({
        tag: "DomOperationFailed",
        value: "test mount failure",
      }),
  }
  expect(await run(await compiledApp(), { dom: failedMount })).toEqual({
    kind: "failure",
    error: "DOM runtime failed",
  })
})
