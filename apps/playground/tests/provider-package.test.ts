import { describe, expect, test } from "bun:test"
import { providerRuntimeAbi } from "../../../runtime/ts/src/provider"
import {
  defineProviderPackage,
  type ProviderModuleSelection,
  ProviderPackageDefect,
  ProviderPackageLoader,
  type ProviderRuntimeTarget,
  providerPackageRuntime,
} from "../../../runtime/ts/src/provider-package"

let identity = 0

function provider(name: string): string {
  identity += 1
  return `fixture/${name}-${identity}#provider`
}

function entry(
  providerIdentity: string,
  service = "fixture/service::Clock",
  targets: ReadonlyArray<ProviderRuntimeTarget> = ["bun-process"],
  shutdown?: () => Promise<void>
) {
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: providerIdentity,
    service,
    targets,
    operations: {
      async now() {
        return { kind: "success", value: 42 }
      },
    },
    ...(shutdown === undefined ? {} : { shutdown }),
  })
}

function selection(
  providerIdentity: string,
  providerEntry: ReturnType<typeof entry>,
  options: Readonly<{
    loadMode?: "eager" | "lazy"
    target?: ProviderRuntimeTarget
    load?: () => Promise<unknown>
  }> = {}
): ProviderModuleSelection {
  return {
    provider: providerIdentity,
    service: "fixture/service::Clock",
    target: options.target ?? "bun-process",
    module: `fixture/${providerIdentity}/entry`,
    exportName: "provider",
    loadMode: options.loadMode ?? "lazy",
    importModule:
      options.load ?? (async () => Object.freeze({ provider: providerEntry })),
    source: { path: "src/main.ssrg", start: 20, end: 31 },
  }
}

describe("TypeScript provider package boundary", () => {
  test("publishes one versioned package identity and branded frozen entries", () => {
    expect(providerPackageRuntime).toEqual({
      identity: "seseragi/provider-package/typescript",
      version: 1,
      abi: providerRuntimeAbi,
    })
    const identity = provider("brand")
    const defined = entry(identity)
    expect(Object.isFrozen(defined)).toBe(true)
    expect(Reflect.ownKeys(defined)).toContain("now")
    expect(
      Reflect.ownKeys(defined).some((key) => typeof key === "symbol")
    ).toBe(true)
    expect(() =>
      defineProviderPackage({
        abi: { ...providerRuntimeAbi, abiMajor: 2 } as never,
        provider: identity,
        service: "fixture/service::Clock",
        targets: ["bun-process"],
        operations: {},
      })
    ).toThrow()
  })

  test("loads eager entries at start and lazy entries once on first use", async () => {
    const eagerIdentity = provider("eager")
    const lazyIdentity = provider("lazy")
    let eagerLoads = 0
    let lazyLoads = 0
    const eagerEntry = entry(eagerIdentity)
    const lazyEntry = entry(lazyIdentity)
    const loader = new ProviderPackageLoader("bun-process", [
      selection(eagerIdentity, eagerEntry, {
        loadMode: "eager",
        load: async () => {
          eagerLoads += 1
          return { provider: eagerEntry }
        },
      }),
      selection(lazyIdentity, lazyEntry, {
        load: async () => {
          lazyLoads += 1
          return { provider: lazyEntry }
        },
      }),
    ])

    await Promise.all([loader.start(), loader.start()])
    expect(eagerLoads).toBe(1)
    expect(lazyLoads).toBe(0)
    const [first, second] = await Promise.all([
      loader.load(lazyIdentity),
      loader.load(lazyIdentity),
    ])
    expect(lazyLoads).toBe(1)
    expect(first).toBe(second)
    expect(first.entry).toBe(lazyEntry)
    await loader.shutdown()
  })

  test("rejects process and web target mixing before module evaluation", () => {
    const identity = provider("target")
    let evaluated = false
    expect(
      () =>
        new ProviderPackageLoader("browser", [
          selection(identity, entry(identity), {
            target: "bun-process",
            load: async () => {
              evaluated = true
              return {}
            },
          }),
        ])
    ).toThrow()
    expect(evaluated).toBe(false)
  })

  test("enforces one runtime singleton entry for each provider identity", async () => {
    const identity = provider("singleton")
    const sharedEntry = entry(identity)
    const first = new ProviderPackageLoader("bun-process", [
      selection(identity, sharedEntry),
    ])
    const second = new ProviderPackageLoader("bun-process", [
      selection(identity, sharedEntry),
    ])
    await first.load(identity)
    const defect = await second.load(identity).catch((error) => error)
    expect(defect).toBeInstanceOf(ProviderPackageDefect)
    expect(defect.stage).toBe("validate")
    await first.shutdown()
    await second.shutdown()
  })

  test("shuts down loaded entries once in reverse order and keeps defects", async () => {
    const events: string[] = []
    const firstIdentity = provider("shutdown-first")
    const secondIdentity = provider("shutdown-second")
    const lazyIdentity = provider("shutdown-unused")
    const loader = new ProviderPackageLoader("bun-process", [
      selection(
        firstIdentity,
        entry(
          firstIdentity,
          "fixture/service::Clock",
          ["bun-process"],
          async () => {
            events.push("first")
            throw new Error("first shutdown failed")
          }
        ),
        { loadMode: "eager" }
      ),
      selection(
        secondIdentity,
        entry(
          secondIdentity,
          "fixture/service::Clock",
          ["bun-process"],
          async () => {
            events.push("second")
            throw new Error("second shutdown failed")
          }
        ),
        { loadMode: "eager" }
      ),
      selection(
        lazyIdentity,
        entry(
          lazyIdentity,
          "fixture/service::Clock",
          ["bun-process"],
          async () => {
            events.push("unused")
          }
        )
      ),
    ])
    await loader.start()
    const defect = await loader.shutdown().catch((error) => error)
    expect(events).toEqual(["second", "first"])
    expect(defect).toBeInstanceOf(ProviderPackageDefect)
    expect(defect.stage).toBe("shutdown")
    expect(defect.notes).toHaveLength(1)
    await expect(loader.shutdown()).rejects.toBe(defect)
    await expect(loader.load(firstIdentity)).rejects.toThrow(
      "provider package loader is shutting down"
    )
  })

  test("retains Seseragi and host frames across a module load defect", async () => {
    const identity = provider("stack")
    const loader = new ProviderPackageLoader("bun-process", [
      selection(identity, entry(identity), {
        load: async () => {
          throw new Error("host import failed")
        },
      }),
    ])
    const defect = await loader.load(identity).catch((error) => error)
    expect(defect).toBeInstanceOf(ProviderPackageDefect)
    expect(defect.frames[0]).toEqual({
      kind: "seseragi",
      path: "src/main.ssrg",
      start: 20,
      end: 31,
    })
    expect(defect.frames[1]).toEqual(
      expect.objectContaining({
        kind: "host",
        stack: expect.stringContaining("host import failed"),
      })
    )
    await loader.shutdown()
  })
})
