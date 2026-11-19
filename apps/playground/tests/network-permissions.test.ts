import { describe, expect, test } from "bun:test"
import type { EntryContract } from "../src/compiler/types"
import {
  contractNeedsNetwork,
  createNetworkPermissionStore,
  htmlNeedsNetwork,
  networkPermissionStorageKey,
} from "../src/runtime/network-permissions"
import {
  createNetworkGate,
  type NetworkConsent,
  type NetworkNotice,
} from "../src/ui/network-consent"

function memoryStorage(): Pick<Storage, "getItem" | "setItem"> {
  const values = new Map<string, string>()
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => void values.set(key, value),
  }
}

function contract(services: readonly string[]): EntryContract {
  return {
    environment: [],
    failureRenderer: { kind: "never" },
    providers: services.map((service) => ({
      provider: `seseragi/runtime-browser#${service}`,
      service,
      target: "browser",
      entryModule: "main",
      entryExport: "main",
    })),
  }
}

describe("network permissions (#1371)", () => {
  test("remembers allow and deny per scope, and can be removed", () => {
    const storage = memoryStorage()
    const store = createNetworkPermissionStore(storage)
    store.set("example:a", "A", "allowed")
    store.set("example:b", "B", "denied")
    expect(store.get("example:a")?.decision).toBe("allowed")
    expect(store.get("example:b")?.decision).toBe("denied")

    // 別の画面（Tour など）から開き直しても同じ答えを読む
    const reopened = createNetworkPermissionStore(storage)
    expect(reopened.list().map(({ scope }) => scope)).toEqual([
      "example:a",
      "example:b",
    ])
    reopened.remove("example:a")
    expect(reopened.get("example:a")).toBeUndefined()
  })

  test("ignores broken saved data", () => {
    const storage = memoryStorage()
    storage.setItem(networkPermissionStorageKey, "{not json")
    expect(createNetworkPermissionStore(storage).list()).toEqual([])
    storage.setItem(
      networkPermissionStorageKey,
      JSON.stringify([{ scope: "x", decision: "maybe" }])
    )
    expect(createNetworkPermissionStore(storage).list()).toEqual([])
  })

  test("finds network services in the entry contract", () => {
    expect(contractNeedsNetwork(contract(["storage", "dom"]))).toBe(false)
    expect(contractNeedsNetwork(contract(["http-client"]))).toBe(true)
    expect(contractNeedsNetwork(contract(["websocket"]))).toBe(true)
    expect(contractNeedsNetwork(contract(["sse"]))).toBe(true)
  })

  test("finds external images in static HTML", () => {
    expect(htmlNeedsNetwork("<img src='https://example.com/a.png'>")).toBe(true)
    expect(htmlNeedsNetwork('<div style="background: url(https://x/y)">')).toBe(
      true
    )
    expect(htmlNeedsNetwork('<img src="data:image/png;base64,AA">')).toBe(false)
    expect(htmlNeedsNetwork('<a href="https://example.com">link</a>')).toBe(
      false
    )
  })
})

describe("network gate (#1371)", () => {
  function setup(answer: boolean) {
    const store = createNetworkPermissionStore(memoryStorage())
    let asked = 0
    let shown = 0
    let reruns = 0
    const consent: NetworkConsent = {
      ask: async () => {
        asked += 1
        return answer
      },
    }
    const notice: NetworkNotice = {
      show: () => {
        shown += 1
      },
      hide: () => undefined,
    }
    const gate = createNetworkGate({
      store,
      consent,
      notice,
      current: () => ({ scope: "example:a", label: "A" }),
      rerun: () => {
        reruns += 1
      },
    })
    return {
      gate,
      store,
      counts: () => ({ asked, shown, reruns }),
    }
  }

  test("asks once, remembers Allow, and does not ask again", async () => {
    const { gate, store, counts } = setup(true)
    expect(await gate.resolve(false)).toBe(false)
    expect(counts().asked).toBe(0)
    expect(await gate.resolve(true)).toBe(true)
    expect(await gate.resolve(true)).toBe(true)
    expect(counts()).toEqual({ asked: 1, shown: 0, reruns: 0 })
    expect(store.get("example:a")?.decision).toBe("allowed")
  })

  test("remembers Cancel and shows the notice instead of asking again", async () => {
    const { gate, counts } = setup(false)
    expect(await gate.resolve(true)).toBe(false)
    expect(await gate.resolve(true)).toBe(false)
    expect(counts()).toEqual({ asked: 1, shown: 2, reruns: 0 })
  })

  test("asks after a block inside the Preview and reruns when allowed", async () => {
    const { gate, counts } = setup(true)
    gate.blocked()
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(counts()).toEqual({ asked: 1, shown: 0, reruns: 1 })
    gate.blocked()
    expect(counts().asked).toBe(1)
  })
})
