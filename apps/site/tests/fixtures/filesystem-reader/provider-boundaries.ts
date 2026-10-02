// Supplemental runtime/provider evidence, not a reader example or browser test.
// Uses current source directly, without a copied runtime or lifecycle shortcuts.
// The first five cases use the real host. The final three inject write/flush/close
// behavior through FileSystemHost; both groups use the real adapter/lifecycle.
// Real-host faults only replace resources inside this test's owned root.
import assert from "node:assert/strict"
import {
  access,
  mkdir,
  mkdtemp,
  readFile,
  rename,
  rm,
  writeFile,
} from "node:fs/promises"
import { join } from "node:path"
import {
  createFileSystemProvider,
  type FileSystemHost,
} from "../../../../../runtime/providers/filesystem"
import { fromUint8Array } from "../../../../../runtime/ts/src/bytes"
import {
  attempt,
  createEffectExecution,
  type Effect,
  fail,
  run,
} from "../../../../../runtime/ts/src/effect"
import * as fs from "../../../../../runtime/ts/src/filesystem"
import { child, parse, render } from "../../../../../runtime/ts/src/path"
import { createProviderFileSystem } from "../../../../../runtime/ts/src/provider-filesystem"
import {
  ProviderPackageLoader,
  type ProviderRuntimeTarget,
} from "../../../../../runtime/ts/src/provider-package"

const base = process.argv[2]
const target = process.argv[3] as ProviderRuntimeTarget
assert.ok(base, "an owned test root is required")
assert.ok(target === "bun-process" || target === "node-process")
const rows: unknown[] = []
const exists = async (file: string) =>
  access(file).then(
    () => true,
    () => false
  )

async function load(name: string, host?: FileSystemHost) {
  const provider = `filesystem-reader/${name}`
  const entry = createFileSystemProvider(provider, target, host)
  const loader = new ProviderPackageLoader(target, [
    {
      provider,
      service: "std/fs::FileSystem",
      target,
      module: "reader/real-filesystem-provider",
      exportName: "provider",
      loadMode: "eager",
      importModule: async () => ({ provider: entry }),
    },
  ])
  await loader.start()
  return {
    loader,
    fileSystem: createProviderFileSystem(await loader.load(provider)),
  }
}

for (const scenario of [
  "success",
  "callback-failure",
  "acquisition-failure",
  "cleanup-failure",
  "combined-failure",
] as const) {
  const own = await mkdtemp(join(base, "scenario-"))
  const temporaryRoot = join(own, "temporary")
  if (scenario === "acquisition-failure")
    await writeFile(temporaryRoot, "not a directory")
  else await mkdir(temporaryRoot)
  const previous = process.env.TMPDIR
  process.env.TMPDIR = temporaryRoot
  const { loader, fileSystem } = await load(scenario)
  const execution = createEffectExecution()
  const callbackError = { message: "report rejected" }
  let temporary = ""
  let callbackRuns = 0
  const callback: (
    path: fs.FilePath
  ) => Effect<fs.FileSystemEnvironment, typeof callbackError, string> =
    (directory) => async (environment, context) => {
      callbackRuns++
      temporary = render(directory)
      assert.ok(temporary.startsWith(`${temporaryRoot}/`))
      if (scenario === "cleanup-failure" || scenario === "combined-failure") {
        await rename(temporary, `${temporary}.original`)
        await mkdir(temporary)
        await writeFile(join(temporary, "replacement"), "must survive cleanup")
      }
      if (scenario === "callback-failure" || scenario === "combined-failure")
        return fail(callbackError)(environment, context)
      if (scenario === "success") {
        const parsed = child("report.txt", directory)
        assert.equal(parsed.tag, "Right")
        const file = parsed.value
        const planned = fs.writeTextUtf8(
          fs.CreateNew,
          "café\nこんにちは\n",
          file
        )
        assert.equal(await exists(render(file)), false)
        await planned(environment, context)
        assert.deepEqual(
          await readFile(render(file)),
          Buffer.from("café\nこんにちは\n")
        )
        assert.equal(
          await fs.readTextUtf8(file)(environment, context),
          "café\nこんにちは\n"
        )
        const duplicate = await attempt(planned)(environment, context)
        assert.equal(duplicate.tag, "Left")
        assert.equal(duplicate.value.kind.tag, "FileAlreadyExists")
        assert.deepEqual(
          await readFile(render(file)),
          Buffer.from("café\nこんにちは\n")
        )
        await fs.writeTextUtf8(fs.Replace, "OK", file)(environment, context)
        assert.deepEqual(await readFile(render(file)), Buffer.from("OK"))
        await fs.writeTextUtf8(fs.Append, "!", file)(environment, context)
        assert.deepEqual(await readFile(render(file)), Buffer.from("OK!"))
        await fs.writeTextUtf8(fs.Replace, "", file)(environment, context)
        assert.deepEqual(await readFile(render(file)), Buffer.alloc(0))
        assert.equal(await fs.readTextUtf8(file)(environment, context), "")
        await fs.writeBytes(
          fs.Replace,
          fromUint8Array(new Uint8Array([65, 195, 40, 66])),
          file
        )(environment, context)
        assert.deepEqual(
          await readFile(render(file)),
          Buffer.from([65, 195, 40, 66])
        )
        const invalid = await attempt(fs.readTextUtf8(file))(
          environment,
          context
        )
        assert.deepEqual(invalid, {
          tag: "Left",
          value: {
            tag: "FileUtf8Failure",
            value: { tag: "InvalidUtf8", value: { offset: 1 } },
          },
        })
        await fs.writeBytes(
          fs.Replace,
          fromUint8Array(new Uint8Array([239, 187, 191, 65])),
          file
        )(environment, context)
        assert.equal(
          await fs.readTextUtf8(file)(environment, context),
          "\uFEFFA"
        )
        assert.deepEqual(
          await readFile(render(file)),
          Buffer.from([239, 187, 191, 65])
        )
      }
      return "ready"
    }
  try {
    const result = await run(
      fs.withTemporaryDirectory("fs16-provider-", callback),
      { fileSystem },
      execution.context
    )
    await execution.close()
    let shutdown = "success"
    try {
      await loader.shutdown()
    } catch (error) {
      assert.ok(error instanceof Error && "stage" in error)
      assert.equal(error.name, "ProviderPackageDefect")
      assert.equal(error.stage, "shutdown")
      shutdown = "separate ProviderPackageDefect"
    }
    if (scenario === "success") {
      assert.deepEqual(result, { kind: "success", value: "ready" })
      assert.equal(await exists(temporary), false)
    } else if (scenario === "callback-failure") {
      assert.deepEqual(result, {
        kind: "failure",
        error: { tag: "Right", value: callbackError },
      })
      assert.equal(await exists(temporary), false)
    } else {
      assert.equal(result.kind, "failure")
      if (scenario === "combined-failure") {
        assert.equal(result.error.tag, "Right")
        assert.equal(result.error.value, callbackError)
        assert.deepEqual(Reflect.ownKeys(result.error.value), ["message"])
      } else {
        assert.equal(result.error.tag, "Left")
        assert.equal(
          result.error.value.operation.tag,
          scenario === "acquisition-failure" ? "CreateTemporary" : "RemovePath"
        )
        assert.equal(
          result.error.value.kind.tag,
          scenario === "acquisition-failure"
            ? "NotADirectory"
            : "PermissionDenied"
        )
      }
      if (scenario === "acquisition-failure") assert.equal(callbackRuns, 0)
      else {
        assert.equal(await exists(join(temporary, "replacement")), true)
        assert.equal(await exists(`${temporary}.original`), true)
      }
    }
    const cleanupFailed =
      scenario === "cleanup-failure" || scenario === "combined-failure"
    assert.equal(
      shutdown,
      cleanupFailed ? "separate ProviderPackageDefect" : "success"
    )
    rows.push({
      scenario,
      callbackRuns,
      outcome: result.kind,
      failureSide: result.kind === "failure" ? result.error.tag : null,
      shutdown,
    })
  } finally {
    if (previous === undefined) delete process.env.TMPDIR
    else process.env.TMPDIR = previous
    await rm(own, { recursive: true, force: true })
  }
}

// Deterministic host injection checks write completion/failure meaning. These
// three cases do not claim to be actual disk faults or cancellation evidence.
for (const fault of ["none", "flush", "close"] as const) {
  const events: string[] = []
  let closeCalls = 0
  const host: FileSystemHost = {
    async openRead() {
      throw new Error("unused")
    },
    async openWrite() {
      events.push("open")
      return {
        async read() {
          throw new Error("unused")
        },
        async write(_bytes, _offset, length) {
          events.push("write")
          return { bytesWritten: length ?? 0 }
        },
        async sync() {
          events.push("flush")
          if (fault === "flush")
            throw Object.assign(new Error("injected flush"), { code: "EIO" })
        },
        async close() {
          events.push("close")
          if (fault === "close" && closeCalls++ === 0)
            throw Object.assign(new Error("injected close"), { code: "EIO" })
        },
      }
    },
  }
  const { loader, fileSystem } = await load(`host-${fault}`, host)
  const execution = createEffectExecution()
  const parsed = parse("test-owned-report.txt")
  assert.equal(parsed.tag, "Right")
  const work = fs.writeTextUtf8(fs.CreateNew, "OK", parsed.value)
  assert.deepEqual(events, [])
  const result = await run(work, { fileSystem }, execution.context)
  assert.deepEqual(events, ["open", "write", "flush", "close"])
  assert.equal(result.kind, fault === "none" ? "success" : "failure")
  if (result.kind === "failure") {
    assert.deepEqual(result.error.kind, {
      tag: "OtherFileSystemError",
      value: "EIO",
    })
    assert.equal(
      result.error.operation.tag,
      fault === "close" ? "ReadFile" : "WriteFile"
    )
  }
  await execution.close()
  let shutdown = "success"
  try {
    await loader.shutdown()
  } catch (error) {
    assert.equal(fault, "close")
    assert.ok(error instanceof Error && "stage" in error)
    assert.equal(error.name, "ProviderPackageDefect")
    assert.equal(error.stage, "shutdown")
    shutdown = "separate ProviderPackageDefect"
  }
  rows.push({
    scenario: `injected-${fault}`,
    outcome: result.kind,
    shutdown,
    events,
  })
}
console.log(JSON.stringify({ target, rows }))
