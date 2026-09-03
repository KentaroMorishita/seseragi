import { describe, expect, test } from "bun:test"
import { readdir, readFile } from "node:fs/promises"
import {
  assertBrowserRuntimeCoverage,
  browserRuntimeEntries,
  loadBrowserRuntimeEntries,
  type RuntimePackage,
  renderRuntimeRegistry,
  runtimeRegistryPath,
} from "../../../scripts/generate-playground-runtime"
import { runtimeModules } from "../src/runtime/runtime-modules"

const runtime = (await Bun.file(
  new URL("../../../runtime/ts/package.json", import.meta.url)
).json()) as RuntimePackage
const providers = (await Bun.file(
  new URL("../../../runtime/providers/package.json", import.meta.url)
).json()) as RuntimePackage

describe("canonical Playground runtime registry", () => {
  test("projects every browser-capable package export without a second list", async () => {
    const entries = await loadBrowserRuntimeEntries()
    assertBrowserRuntimeCoverage(entries, runtimeModules)
    expect(await readFile(runtimeRegistryPath, "utf8")).toBe(
      renderRuntimeRegistry(entries)
    )
    expect(runtimeModules["@seseragi/runtime/process"]).toBeUndefined()
    expect(runtimeModules["@seseragi/runtime"]).toBeUndefined()
    expect(
      Object.keys(runtimeModules).some(
        (name) =>
          name.startsWith("seseragi/") &&
          !name.startsWith("seseragi/runtime-browser/")
      )
    ).toBe(false)
  })

  test("detects a newly exported runtime or provider missing from Playground", () => {
    const addedRuntime = browserRuntimeEntries(
      {
        ...runtime,
        exports: {
          ...runtime.exports,
          "./future-collection": { default: "./src/future-collection.ts" },
        },
      },
      providers
    )
    expect(renderRuntimeRegistry(addedRuntime)).toContain(
      "runtime/ts/src/future-collection.ts"
    )
    expect(() =>
      assertBrowserRuntimeCoverage(addedRuntime, runtimeModules)
    ).toThrow("runtime registry drift")
    const addedProvider = browserRuntimeEntries(runtime, {
      ...providers,
      exports: {
        ...providers.exports,
        "./runtime-browser/future": { default: "./runtime-browser/future.ts" },
      },
    })
    expect(() =>
      assertBrowserRuntimeCoverage(addedProvider, runtimeModules)
    ).toThrow("runtime registry drift")
  })

  test("preserves browser host adapters instead of native process modules", () => {
    const entries = browserRuntimeEntries(runtime, providers)
    for (const name of ["console", "logger", "stdin"]) {
      expect(
        entries.find(
          ({ specifier }) => specifier === `@seseragi/runtime/${name}`
        )?.source
      ).toBe(`runtime/ts/src/browser/${name}.ts`)
    }
  })

  test("resolves compiler runtime ABI imports and their value exports", async () => {
    const abi = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/runtime-schema-1/core/abi.json",
        import.meta.url
      )
    ).json()
    for (const feature of abi.features) {
      const binding = feature.import
      if (binding === null) continue
      const name = `.${binding.module.slice(runtime.name.length)}`
      const target = runtime.exports[name]
      expect(target, binding.module).toBeDefined()
      if (target?.browser === null) continue
      const module = runtimeModules[binding.module] as Record<string, unknown>
      expect(module, binding.module).toBeDefined()
      expect(
        module[binding.export],
        `${binding.module}#${binding.export}`
      ).toBeDefined()
    }
  })

  test("connects canonical browser provider entries, excluding process providers", async () => {
    const directory = new URL(
      "../../../examples/spec/artifacts/provider-manifest-schema-1/",
      import.meta.url
    )
    for (const name of await readdir(directory)) {
      const manifest = await Bun.file(
        new URL(`${name}/provider.json`, directory)
      ).json()
      if (!manifest.targets.includes("browser")) continue
      const module = runtimeModules[manifest.entry.module] as Record<
        string,
        unknown
      >
      expect(module, manifest.entry.module).toBeDefined()
      expect(module[manifest.entry.export]).toBeDefined()
    }
  })
})
