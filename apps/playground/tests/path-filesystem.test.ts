import { describe, expect, test } from "bun:test"
import { access } from "node:fs/promises"
import { createFileSystemProvider } from "../../../runtime/providers/filesystem"
import {
  createEffectExecution,
  type Effect,
  EffectCancellation,
  fail,
  run,
} from "../../../runtime/ts/src/effect"
import {
  type FileSystemEnvironment,
  Replace,
  readTextUtf8,
  withTemporaryDirectory,
  writeTextUtf8,
} from "../../../runtime/ts/src/filesystem"
import {
  AbsoluteChildPath,
  child,
  current,
  extension,
  fileName,
  isAbsolute,
  join,
  normalize,
  type Path,
  parent,
  parse,
  render,
} from "../../../runtime/ts/src/path"
import { createProviderFileSystem } from "../../../runtime/ts/src/provider-filesystem"
import { ProviderPackageLoader } from "../../../runtime/ts/src/provider-package"
import type { Either } from "../../../runtime/ts/src/sum"

describe("portable Path", () => {
  test("keeps POSIX, drive, UNC, and relative roots distinct", () => {
    const cases = [
      ["/a/./b/../c", "/a/c", true],
      ["C:/a/../../b", "C:/b", true],
      ["//server/share/a/../b", "//server/share/b", true],
      ["../../a/./b", "../../a/b", false],
    ] as const
    for (const [source, expected, absolute] of cases) {
      const value = right(parse(source))
      expect(render(normalize(value))).toBe(expected)
      expect(isAbsolute(value)).toBe(absolute)
    }
  })

  test("validates child paths and derives lexical components", () => {
    const root = right(parse("/"))
    const file = right(child("archive.tar.gz", root))
    expect(render(file)).toBe("/archive.tar.gz")
    expect(fileName(file)).toEqual({ tag: "Just", value: "archive.tar.gz" })
    expect(extension(file)).toEqual({ tag: "Just", value: "gz" })
    expect(parent(file)).toEqual({ tag: "Just", value: root })
    expect(parent(root)).toEqual({ tag: "Nothing" })
    expect(join(root, current())).toEqual({
      tag: "Left",
      value: AbsoluteChildPath,
    })
    expect(parse("bad\\path").tag).toBe("Left")
    expect(parse("C:relative").tag).toBe("Left")
    expect(parse("//server").tag).toBe("Left")
  })
})

describe("FileSystem application API", () => {
  for (const target of ["bun-process", "node-process"] as const) {
    test(`writes, reads, and cleans a temporary tree through ${target}`, async () => {
      const selected = await environment(target)
      let temporary: Path | undefined
      const effect = withTemporaryDirectory(
        "seseragi-test-",
        (directory) => async (services: FileSystemEnvironment, context) => {
          temporary = directory
          const file = right(child("message.txt", directory))
          await writeTextUtf8(Replace, "hello", file)(services, context)
          return readTextUtf8(file)(services, context)
        }
      )
      const result = await run(effect, { fileSystem: selected.fileSystem })
      expect(result).toEqual({ kind: "success", value: "hello" })
      expect(temporary).toBeDefined()
      await expect(access(render(temporary as Path))).rejects.toBeDefined()
      await selected.loader.shutdown()
    })
  }

  test("cleans temporary resources after typed failure and cancellation", async () => {
    const selected = await environment("bun-process")
    let failedPath: Path | undefined
    const failed = await run(
      withTemporaryDirectory("seseragi-fail-", (path) => {
        failedPath = path
        return fail("use-failed")
      }),
      { fileSystem: selected.fileSystem }
    )
    expect(failed).toEqual({
      kind: "failure",
      error: { tag: "Right", value: "use-failed" },
    })
    await expect(access(render(failedPath as Path))).rejects.toBeDefined()

    let resolveAcquired = (_path: Path): void => undefined
    const acquired = new Promise<Path>((resolve) => {
      resolveAcquired = resolve
    })
    const execution = createEffectExecution()
    const pending = run(
      withTemporaryDirectory(
        "seseragi-cancel-",
        (path) =>
          ((_environment, context) => {
            resolveAcquired(path)
            return new Promise<never>((_resolve, reject) => {
              context?.signal.addEventListener(
                "abort",
                () => reject(new EffectCancellation()),
                { once: true }
              )
            })
          }) as Effect<unknown, never, never>
      ),
      { fileSystem: selected.fileSystem },
      execution.context
    )
    const observed = pending.catch((error: unknown) => error)
    const cancelledPath = await acquired
    await execution.cancel()
    expect(await observed).toBeInstanceOf(EffectCancellation)
    await expect(access(render(cancelledPath))).rejects.toBeDefined()
    await selected.loader.shutdown()
  })
})

async function environment(target: "bun-process" | "node-process") {
  const providerIdentity = `fixture/${target}#filesystem`
  const entry = createFileSystemProvider(providerIdentity, target)
  const loader = new ProviderPackageLoader(target, [
    {
      provider: providerIdentity,
      service: "std/fs::FileSystem",
      target,
      module: `fixture/${target}/filesystem`,
      exportName: "provider",
      loadMode: "eager",
      importModule: async () => ({ provider: entry }),
    },
  ])
  await loader.start()
  return {
    loader,
    fileSystem: createProviderFileSystem(await loader.load(providerIdentity)),
  }
}

function right<Failure, Success>(value: Either<Failure, Success>): Success {
  if (value.tag !== "Right") throw new Error("expected Right")
  return value.value as Success
}
