import { describe, expect, test } from "bun:test"
import {
  type BrowserStorageArea,
  type BrowserStorageHost,
  createBrowserStorageProvider,
  createBudgetedStorageHost,
  createNamespacedStorageHost,
} from "../../../runtime/ts/src/browser/provider-storage"
import { run } from "../../../runtime/ts/src/effect"
import { ProviderPackageLoader } from "../../../runtime/ts/src/provider-package"
import { createProviderStorage } from "../../../runtime/ts/src/provider-storage"
import {
  clear,
  get,
  keys,
  Local,
  remove,
  Session,
  set,
} from "../../../runtime/ts/src/storage"

function memoryStorage() {
  const areas = {
    local: new Map<string, string>(),
    session: new Map<string, string>(),
  }
  let failure: unknown
  const access = (area: BrowserStorageArea) => {
    if (failure !== undefined) throw failure
    return areas[area]
  }
  const host: BrowserStorageHost = {
    get: (area, key) => access(area).get(key) ?? null,
    set: (area, key, value) => {
      access(area).set(key, value)
    },
    remove: (area, key) => {
      access(area).delete(key)
    },
    clear: (area) => access(area).clear(),
    keys: (area) => Object.freeze([...access(area).keys()].sort()),
  }
  return {
    fail(name: string, message: string) {
      failure = Object.freeze({ name, message })
    },
    recover() {
      failure = undefined
    },
    host,
  }
}

async function storageFixture(host: BrowserStorageHost) {
  const provider = "seseragi/runtime-browser#storage"
  const loader = new ProviderPackageLoader("browser", [
    {
      provider,
      service: "std/web/storage::Storage",
      target: "browser",
      module: "fixture/runtime-browser/storage",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({
        provider: createBrowserStorageProvider(host),
      }),
    },
  ])
  return {
    environment: {
      storage: createProviderStorage(await loader.load(provider)),
    },
    loader,
  }
}

describe("browser Storage provider vertical slice", () => {
  test("isolates a namespaced application from host-owned storage", () => {
    const memory = memoryStorage()
    memory.host.set("local", "playground:workspace", "keep")
    const namespace = "playground:application:"
    const application = createBudgetedStorageHost(
      createNamespacedStorageHost(memory.host, namespace),
      80,
      namespace.length
    )

    application.set("local", "profile", "Mio")
    application.set("local", "draft", "open")
    expect(application.keys("local")).toEqual(["draft", "profile"])
    expect(application.get("local", "profile")).toBe("Mio")
    expect(application.get("local", "playground:workspace")).toBeNull()
    expect(() => application.set("local", "large", "x".repeat(64))).toThrow(
      expect.objectContaining({
        name: "QuotaExceededError",
        message: expect.stringContaining("storage application budget exceeded"),
      })
    )
    expect(application.keys("local")).toEqual(["draft", "profile"])

    application.clear("local")
    expect(application.keys("local")).toEqual([])
    expect(memory.host.get("local", "playground:workspace")).toBe("keep")
  })

  test("keeps local and session values distinct and preserves missing", async () => {
    const memory = memoryStorage()
    const fixture = await storageFixture(memory.host)

    expect(await run(get(Local, "empty"), fixture.environment)).toEqual({
      kind: "success",
      value: { tag: "Nothing" },
    })
    expect(await run(set(Local, "empty", ""), fixture.environment)).toEqual({
      kind: "success",
      value: undefined,
    })
    expect(
      await run(set(Session, "empty", "session"), fixture.environment)
    ).toEqual({ kind: "success", value: undefined })
    expect(await run(get(Local, "empty"), fixture.environment)).toEqual({
      kind: "success",
      value: { tag: "Just", value: "" },
    })
    expect(await run(get(Session, "empty"), fixture.environment)).toEqual({
      kind: "success",
      value: { tag: "Just", value: "session" },
    })

    await run(set(Local, "z", "last"), fixture.environment)
    await run(set(Local, "a", "first"), fixture.environment)
    expect(await run(keys(Local), fixture.environment)).toEqual({
      kind: "success",
      value: ["a", "empty", "z"],
    })
    await run(remove(Local, "empty"), fixture.environment)
    expect(await run(get(Local, "empty"), fixture.environment)).toEqual({
      kind: "success",
      value: { tag: "Nothing" },
    })
    await run(clear(Session), fixture.environment)
    expect(await run(keys(Session), fixture.environment)).toEqual({
      kind: "success",
      value: [],
    })
    expect(await run(keys(Local), fixture.environment)).toEqual({
      kind: "success",
      value: ["a", "z"],
    })
    await fixture.loader.shutdown()
  })

  test("classifies quota, security, and unavailable host failures", async () => {
    const memory = memoryStorage()
    const fixture = await storageFixture(memory.host)

    memory.fail("QuotaExceededError", "storage quota reached")
    expect(
      await run(set(Local, "profile", "large"), fixture.environment)
    ).toEqual({
      kind: "failure",
      error: {
        tag: "StorageQuotaExceeded",
        value: {
          area: { tag: "Local" },
          key: "profile",
          message: "storage quota reached",
        },
      },
    })
    memory.fail("SecurityError", "storage blocked")
    expect(await run(get(Session, "profile"), fixture.environment)).toEqual({
      kind: "failure",
      error: {
        tag: "StorageSecurityFailure",
        value: {
          area: { tag: "Session" },
          message: "storage blocked",
        },
      },
    })
    memory.fail("InvalidStateError", "storage unavailable")
    expect(await run(keys(Local), fixture.environment)).toEqual({
      kind: "failure",
      error: {
        tag: "StorageUnavailable",
        value: {
          area: { tag: "Local" },
          message: "storage unavailable",
        },
      },
    })
    memory.recover()
    await fixture.loader.shutdown()
  })
})
