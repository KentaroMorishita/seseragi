import {
  defineProviderPackage,
  ProviderPackageLoader,
  providerPackageRuntime,
} from "@seseragi/runtime/provider-package"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const provider = "seseragi/runtime-bun#probe"
let loads = 0
let shutdowns = 0
const entry = defineProviderPackage({
  abi: providerPackageRuntime.abi,
  provider,
  service: "std/probe::Probe",
  targets: ["bun-process"],
  operations: {
    async ping() {
      return { kind: "success", value: undefined }
    },
  },
  shutdown: async () => {
    shutdowns += 1
  },
})
const loader = new ProviderPackageLoader("bun-process", [
  {
    provider,
    service: "std/probe::Probe",
    target: "bun-process",
    module: "@seseragi/runtime-bun/probe",
    exportName: "provider",
    loadMode: "lazy",
    importModule: async () => {
      loads += 1
      return { provider: entry }
    },
    source: { path: "src/probe.ssrg", start: 0, end: 5 },
  },
])

await loader.start()
assert(loads === 0, "lazy provider must not load during startup")
const [first, second] = await Promise.all([
  loader.load(provider),
  loader.load(provider),
])
assert(loads === 1, "lazy provider must load exactly once")
assert(first === second, "concurrent loads must share one completion")
await loader.shutdown()
assert(shutdowns === 1, "loaded provider must shut down exactly once")

process.stdout.write("provider package boundary probe passed\n")
