import { access } from "node:fs/promises"
import { createEffectExecution, run } from "@seseragi/runtime/effect"
import { render } from "@seseragi/runtime/path"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
import { createProviderFileSystem } from "@seseragi/runtime/provider-filesystem"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { provider as bunProvider } from "seseragi/runtime-bun/filesystem"
import { provider as nodeProvider } from "seseragi/runtime-node/filesystem"
import {
  closeFixture,
  openFixture,
  readFixture,
  temporaryRoundTripFixture,
} from "./filesystem-application.ts"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const provider = requiredEnvironment("SESERAGI_FILESYSTEM_PROVIDER")
const service = requiredEnvironment("SESERAGI_FILESYSTEM_SERVICE")
const module = requiredEnvironment("SESERAGI_FILESYSTEM_MODULE")
const exportName = requiredEnvironment("SESERAGI_FILESYSTEM_EXPORT")
const target = requiredEnvironment("SESERAGI_FILESYSTEM_TARGET") as
  | "bun-process"
  | "node-process"
const path = requiredEnvironment("SESERAGI_FILESYSTEM_FIXTURE")
const loader = new ProviderPackageLoader(target, [
  {
    provider,
    service,
    target,
    module,
    exportName,
    loadMode: "lazy",
    importModule: selectedImport(module),
    source: { path: "src/main.ssrg", start: 0, end: 10 },
  },
])
const environment = Object.freeze({
  fileSystem: createProviderFileSystem(await loader.load(provider)),
})

const execution = createEffectExecution()
const opened = await run(openFixture(path), environment, execution.context)
assert(opened.kind === "success", "filesystem open must succeed")
const first = await run(
  readFixture(opened.value, 4),
  environment,
  execution.context
)
assert(first.kind === "success", "filesystem read must succeed")
assert(
  new TextDecoder().decode(first.value) === "sese",
  "filesystem read must preserve bytes and cursor position"
)
assertProviderConformanceCase({ id: "success", terminal: first.kind })
const second = await run(
  readFixture(opened.value, 64),
  environment,
  execution.context
)
assert(second.kind === "success", "filesystem second read must succeed")
assert(
  new TextDecoder().decode(second.value) === "ragi-filesystem",
  "filesystem second read must continue from the same handle"
)

await execution.cancel()
const repeatedClose = await run(closeFixture(opened.value), environment)
assert(repeatedClose.kind === "success", "filesystem close must be idempotent")
const afterCleanup = await run(readFixture(opened.value, 1), environment).catch(
  (error: unknown) => error
)
assert(
  afterCleanup instanceof Error && afterCleanup.message.includes("closed"),
  "filesystem cancellation must close the owned handle"
)
assertProviderConformanceCase({
  id: "cancellation",
  terminal: "cancellation",
  notifications: 1,
  lateCompletion: "discarded",
})

const explicit = await run(openFixture(path), environment)
assert(explicit.kind === "success", "filesystem reopen must succeed")
assert(
  (await run(closeFixture(explicit.value), environment)).kind === "success",
  "filesystem explicit close must succeed"
)
assert(
  (await run(closeFixture(explicit.value), environment)).kind === "success",
  "filesystem repeated explicit close must succeed"
)

const temporary = await run(
  temporaryRoundTripFixture("seseragi-provider-probe-"),
  environment
)
assert(
  temporary.kind === "success",
  "filesystem temporary round trip must succeed"
)
assert(
  new TextDecoder().decode(temporary.value.content) ===
    "seseragi-filesystem-round-trip",
  "filesystem temporary round trip must preserve bytes"
)
const cleanedTemporary = await access(render(temporary.value.directory)).then(
  () => false,
  () => true
)
assert(cleanedTemporary, "filesystem temporary directory must be cleaned")

await loader.shutdown()
assertProviderConformanceCase({
  id: "cleanup",
  acquired: 5,
  released: 5,
  active: 0,
})
assertProviderConformanceCase({ id: "leak", activeAfterCleanup: 0 })
process.stdout.write(`filesystem provider probe passed: ${target}\n`)

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0)
    throw new Error(`missing ${name}`)
  return value
}

function selectedImport(module: string): () => Promise<unknown> {
  if (module === "seseragi/runtime-bun/filesystem") {
    return async () => Object.freeze({ provider: bunProvider })
  }
  if (module === "seseragi/runtime-node/filesystem") {
    return async () => Object.freeze({ provider: nodeProvider })
  }
  throw new Error(`unexpected filesystem provider module: ${module}`)
}
