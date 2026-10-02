import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { arrayIterable } from "../../../runtime/ts/src/array"
import {
  attempt,
  Break,
  Continue,
  createEffectExecution,
  defer,
  type Effect,
  EffectCancellation,
  fail,
  flatMap,
  forEachUntil,
  fromEither,
  fromMaybe,
  mapError,
  recover,
  run,
  succeed,
} from "../../../runtime/ts/src/effect"
import { unfold } from "../../../runtime/ts/src/iterator"
import { Just, Left, Nothing, Right } from "../../../runtime/ts/src/sum"
import {
  effectSequencingReaderCases,
  effectSequencingReaderExamples,
  effectSequencingReaderFailures,
  effectSequencingReaderRoutes,
} from "../scripts/effect-sequencing-reader"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const examples = effectSequencingReaderExamples("https://seseragi.vercel.app/")
const fixtures = join(root, "apps/site/tests/fixtures/effect-sequencing")
function command(executable: string, args: string[], cwd = root) {
  return spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}
function temporary<T>(work: (directory: string) => T): T {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-effect-sequencing-"))
  try {
    return work(directory)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}
function sourceIn(source: string, directory: string) {
  writeFileSync(join(directory, "main.ssrg"), source)
}

test("thirteen existing identities use nine exact sources and fair strict TypeScript comparisons", () => {
  expect(effectSequencingReaderRoutes).toHaveLength(13)
  expect(
    new Set(effectSequencingReaderRoutes.map((x) => x.identity)).size
  ).toBe(13)
  expect(effectSequencingReaderCases).toHaveLength(9)
  expect(examples).toHaveLength(18)
  temporary((directory) => {
    for (const item of effectSequencingReaderCases) {
      const source = examples.find(
        (x) => x.id === `effect-sequencing-${item.slug}`
      )!
      sourceIn(source.source, directory)
      const lint = command(
        cli,
        ["lint", "main.ssrg", "--deny-warnings"],
        directory
      )
      expect(lint.status, `${item.slug}: ${lint.stderr}`).toBe(0)
      const formatted = command(
        cli,
        ["format", "--check", "main.ssrg"],
        directory
      )
      expect(formatted.status, formatted.stderr).toBe(0)
      const native = command(cli, ["run", "main.ssrg"], directory)
      expect(native.status, `${item.slug}: ${native.stderr}`).toBe(0)
      expect(native.stderr).toBe("")
      expect(native.stdout).toBe(item.output)
      expect(new URL(source.playgroundUrl).searchParams.get("source")).toBe(
        source.source
      )
      expect(source.highlighted.map((x) => x.text).join("")).toBe(source.source)
      const comparison = examples.find(
        (x) => x.id === `effect-sequencing-${item.slug}-ts`
      )!
      expect(comparison.playgroundUrl).toBe("")
      expect(comparison.highlighted.map((x) => x.text).join("")).toBe(
        comparison.source
      )
      const ts = command(bun, [join(root, comparison.sourcePath)])
      expect(ts.status, ts.stderr).toBe(0)
      expect(ts.stdout).toBe(item.output)
    }
    const checked = command(bun, [
      join(root, "node_modules/typescript/bin/tsc"),
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
      ...effectSequencingReaderCases.map((x) =>
        join(
          root,
          `apps/site/examples/comparisons/effect-sequencing/${x.slug}.ts`
        )
      ),
    ])
    expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  })
})

test("the same nine Playground seeds compile and execute through committed WASM", async () => {
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
  const browser = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  for (const item of effectSequencingReaderCases) {
    const sample = examples.find(
      (x) => x.id === `effect-sequencing-${item.slug}`
    )!
    const seed = new URL(sample.playgroundUrl).searchParams.get("source")!
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    )
    expect(compiled.status, `${item.slug}: ${JSON.stringify(compiled)}`).toBe(
      "success"
    )
    expect(compiled.entry).toBeDefined()
    const result = await browser.executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(result.stdout).toBe(item.output.trimEnd())
  }
})

test("four genuine mistakes have executable repairs and defects escape all typed-error wrappers", () => {
  temporary((directory) => {
    for (const item of effectSequencingReaderFailures) {
      sourceIn(
        readFileSync(join(fixtures, "invalid", `${item.slug}.ssrg`), "utf8"),
        directory
      )
      const rejected = command(cli, ["lint", "main.ssrg"], directory)
      expect(rejected.status).toBe(2)
      for (const diagnostic of item.diagnostics)
        expect(rejected.stderr).toContain(diagnostic)
      sourceIn(
        readFileSync(join(fixtures, "repairs", `${item.slug}.ssrg`), "utf8"),
        directory
      )
      const repaired = command(cli, ["run", "main.ssrg"], directory)
      expect(repaired.status, repaired.stderr).toBe(0)
      expect(repaired.stdout).toBe(item.output)
    }
    for (const slug of [
      "eager-defect",
      "attempt-defect",
      "recover-defect",
      "maperror-defect",
    ]) {
      sourceIn(
        readFileSync(join(fixtures, "runtime-errors", `${slug}.ssrg`), "utf8"),
        directory
      )
      const lint = command(
        cli,
        ["lint", "main.ssrg", "--deny-warnings"],
        directory
      )
      expect(lint.status, lint.stderr).toBe(0)
      const result = command(cli, ["run", "main.ssrg"], directory)
      expect(result.status).toBe(70)
      expect(result.stdout).toBe(
        slug === "eager-defect" ? "" : "Before execution\n"
      )
      expect(result.stderr).toContain("runtime defect")
    }
  })
})

test("saved values, typed error callbacks and deferred construction retain their distinct timing", async () => {
  let later = 0
  const failed = flatMap(fail("expected"), () => {
    later++
    return succeed(1)
  })
  expect(later).toBe(0)
  expect(await run(failed, {})).toEqual({ kind: "failure", error: "expected" })
  expect(later).toBe(0)
  let inputCalls = 0
  const evaluate = () => {
    inputCalls++
    return Right(7)
  }
  const saved = fromEither(evaluate())
  expect(inputCalls).toBe(1)
  expect(await run(saved, {})).toEqual({ kind: "success", value: 7 })
  expect(await run(saved, {})).toEqual({ kind: "success", value: 7 })
  expect(inputCalls).toBe(1)
  expect(await run(fromEither(Left("bad")), {})).toEqual({
    kind: "failure",
    error: "bad",
  })
  expect(await run(fromMaybe("missing", Just(0)), {})).toEqual({
    kind: "success",
    value: 0,
  })
  expect(await run(fromMaybe("missing", Just("")), {})).toEqual({
    kind: "success",
    value: "",
  })
  expect(await run(fromMaybe("missing", Nothing), {})).toEqual({
    kind: "failure",
    error: "missing",
  })
  expect(await run(attempt(fail("bad")), {})).toEqual({
    kind: "success",
    value: Left("bad"),
  })
  expect(await run(attempt(succeed(9)), {})).toEqual({
    kind: "success",
    value: Right(9),
  })
  let mapped = 0
  const mapper = (e: string) => {
    mapped++
    return `stage: ${e}`
  }
  const mappedWork = mapError(mapper, fail("missing"))
  expect(mapped).toBe(0)
  expect(await run(mappedWork, {})).toEqual({
    kind: "failure",
    error: "stage: missing",
  })
  expect(await run(mapError(mapper, succeed("same")), {})).toEqual({
    kind: "success",
    value: "same",
  })
  expect(mapped).toBe(1)
  let recovered = 0
  const fallback = (e: string): Effect<unknown, string, string> => {
    recovered++
    return e === "missing" ? succeed("Preview") : fail(e)
  }
  expect(await run(recover(fallback, fail("missing")), {})).toEqual({
    kind: "success",
    value: "Preview",
  })
  expect(await run(recover(fallback, succeed("Release")), {})).toEqual({
    kind: "success",
    value: "Release",
  })
  expect(await run(recover(fallback, fail("invalid")), {})).toEqual({
    kind: "failure",
    error: "invalid",
  })
  expect(recovered).toBe(2)
  let eagerCalls = 0
  const eager = succeed(++eagerCalls)
  expect(eagerCalls).toBe(1)
  expect(await run(eager, {})).toEqual({ kind: "success", value: 1 })
  expect(await run(eager, {})).toEqual({ kind: "success", value: 1 })
  let delayedCalls = 0
  const delayed = defer(() => succeed(++delayedCalls))
  expect(delayedCalls).toBe(0)
  expect(await run(delayed, {})).toEqual({ kind: "success", value: 1 })
  expect(await run(delayed, {})).toEqual({ kind: "success", value: 2 })
})

test("attempt, mapError and recover preserve defects and actual pending cancellation", async () => {
  let handlers = 0
  const wrappers: Array<
    (
      source: Effect<unknown, string, number>
    ) => Effect<unknown, unknown, unknown>
  > = [
    (source) => attempt(source),
    (source) =>
      mapError((e: string) => {
        handlers++
        return e
      }, source),
    (source) =>
      recover(() => {
        handlers++
        return succeed(42)
      }, source),
  ]
  for (const wrap of wrappers) {
    const defect = new Error("programming defect")
    await expect(
      run(
        wrap(() => {
          throw defect
        }),
        {}
      )
    ).rejects.toBe(defect)
    const execution = createEffectExecution()
    let signalStarted!: () => void
    const started = new Promise<void>((resolveStarted) => {
      signalStarted = resolveStarted
    })
    const pending: Effect<unknown, string, number> = () => {
      signalStarted()
      return new Promise<number>(() => {})
    }
    const result = run(wrap(pending), {}, execution.context).catch(
      (error) => error
    )
    await started
    await execution.cancel()
    expect(await result).toBeInstanceOf(EffectCancellation)
    await execution.close()
  }
  expect(handlers).toBe(0)
})

test("ordinary loop controls stop successfully without pulling or starting a later item", async () => {
  expect(typeof Break).toBe("object")
  expect(typeof Continue).toBe("object")
  expect(await run(succeed(Break), {})).toEqual({
    kind: "success",
    value: Break,
  })
  const events: string[] = []
  const values = unfold((n: number) => {
    events.push(`pull:${n}`)
    if (n > 2) throw new Error("pulled after Break")
    return Just([n, n + 1] as const)
  }, 1)
  const work = forEachUntil(
    (n: number) => async () => {
      events.push(`start:${n}`)
      await Promise.resolve()
      events.push(`end:${n}`)
      return n === 2 ? Break : Continue
    },
    undefined,
    {
      iterate: () => {
        events.push("iterate")
        return values
      },
    }
  )
  expect(events).toEqual([])
  expect(await run(work, {})).toEqual({ kind: "success", value: undefined })
  expect(events).toEqual([
    "iterate",
    "pull:1",
    "start:1",
    "end:1",
    "pull:2",
    "start:2",
    "end:2",
  ])
  let emptyCalls = 0
  await run(
    forEachUntil(
      () => {
        emptyCalls++
        return succeed(Continue)
      },
      [],
      arrayIterable
    ),
    {}
  )
  expect(emptyCalls).toBe(0)
  const visited: number[] = []
  const failed = forEachUntil(
    (n: number) => {
      visited.push(n)
      return n === 2 ? fail("stopped") : succeed(Continue)
    },
    [1, 2, 3],
    arrayIterable
  )
  expect(await run(failed, {})).toEqual({ kind: "failure", error: "stopped" })
  expect(visited).toEqual([1, 2])
})
