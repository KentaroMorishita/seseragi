import { describe, expect, test } from "bun:test"
import {
  type BrowserNavigationHost,
  createBrowserNavigationProvider,
} from "../../../runtime/ts/src/browser/provider-navigation"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "../../../runtime/ts/src/effect"
import {
  appendQuery,
  current,
  locationSignal,
  locationUrl,
  parseQuery,
  parseUrl,
  pathSegments,
  push,
  queryValues,
  renderQuery,
  renderUrl,
  replace,
  resolveUrl,
  urlFragment,
  urlQuery,
  withFragment,
  withPathSegments,
  withQuery,
} from "../../../runtime/ts/src/navigation"
import { createProviderNavigation } from "../../../runtime/ts/src/provider-navigation"
import { ProviderPackageLoader } from "../../../runtime/ts/src/provider-package"
import type { Either } from "../../../runtime/ts/src/sum"

function right<Failure, Value>(value: Either<Failure, Value>): Value {
  expect(value.tag).toBe("Right")
  if (value.tag !== "Right") throw new Error("expected Right")
  return value.value
}

function browserNavigation(initial: string) {
  let href = initial
  let pushes = 0
  let replaces = 0
  let backs = 0
  let forwards = 0
  let listener: (() => void) | undefined
  const host: BrowserNavigationHost = {
    currentHref: () => href,
    push: (value) => {
      pushes += 1
      href = value
    },
    replace: (value) => {
      replaces += 1
      href = value
    },
    back: () => {
      backs += 1
    },
    forward: () => {
      forwards += 1
    },
    listen: (next) => {
      listener = next
      return () => {
        listener = undefined
      }
    },
  }
  const provider = "seseragi/runtime-browser#navigation"
  const loader = new ProviderPackageLoader("browser", [
    {
      provider,
      service: "std/web/navigation::Navigation",
      target: "browser",
      module: "fixture/runtime-browser/navigation",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({
        provider: createBrowserNavigationProvider(host),
      }),
    },
  ])
  return {
    emit(value: string) {
      href = value
      listener?.()
    },
    loader,
    load: async () => createProviderNavigation(await loader.load(provider)),
    observed: () => ({ href, pushes, replaces, backs, forwards }),
  }
}

describe("browser Navigation provider vertical slice", () => {
  test("normalizes URL components and preserves repeated query keys", () => {
    const base = right(parseUrl("https://example.test/a/b?x=1&x=2#old"))
    const resolved = right(resolveUrl("../c?q=a%20b", base))
    expect(renderUrl(resolved)).toBe("https://example.test/c?q=a%20b")
    expect(pathSegments(resolved)).toEqual(["c"])
    expect(queryValues("q", urlQuery(resolved))).toEqual(["a b"])

    const query = right(parseQuery("?tag=one&tag=two"))
    const extended = appendQuery("sp ace", "a/b", query)
    expect(queryValues("tag", extended)).toEqual(["one", "two"])
    expect(renderQuery(extended)).toBe("tag=one&tag=two&sp+ace=a%2Fb")

    const rebuilt = withFragment(
      "section 1",
      withQuery(extended, withPathSegments(["docs", "日本語"], base))
    )
    expect(renderUrl(rebuilt)).toBe(
      "https://example.test/docs/%E6%97%A5%E6%9C%AC%E8%AA%9E?tag=one&tag=two&sp+ace=a%2Fb#section%201"
    )
    expect(urlFragment(rebuilt)).toEqual({ tag: "Just", value: "section 1" })
  })

  test("pushes and replaces only same-origin locations", async () => {
    const fixture = browserNavigation("https://example.test/start")
    const navigation = await fixture.load()
    const environment = { navigation }
    const next = right(parseUrl("https://example.test/next?x=1"))

    const initial = await run(current(), environment)
    expect(initial.kind).toBe("success")
    const pushed = await run(push(next), environment)
    const replaced = await run(
      replace(right(parseUrl("https://example.test/final"))),
      environment
    )
    expect(pushed.kind).toBe("success")
    expect(replaced.kind).toBe("success")
    expect(fixture.observed()).toMatchObject({ pushes: 1, replaces: 1 })
    if (replaced.kind === "success") {
      expect(renderUrl(locationUrl(replaced.value))).toBe(
        "https://example.test/final"
      )
    }

    const external = await run(
      push(right(parseUrl("https://outside.test/path"))),
      environment
    )
    expect(external).toEqual({
      kind: "failure",
      error: {
        tag: "CrossOriginNavigation",
        value: {
          expected: "https://example.test",
          actual: "https://outside.test",
        },
      },
    })
    expect(fixture.observed().pushes).toBe(1)
    await fixture.loader.shutdown()
  })

  test("feeds popstate into Signal and cancels a pending listener once", async () => {
    const fixture = browserNavigation("https://example.test/start")
    const navigation = await fixture.load()
    const execution = createEffectExecution()
    const signalResult = await run(
      locationSignal(),
      { navigation },
      execution.context
    )
    expect(signalResult.kind).toBe("success")
    await new Promise<void>((resolve) => setTimeout(resolve, 0))
    fixture.emit("https://example.test/back")
    await new Promise<void>((resolve) => setTimeout(resolve, 0))
    if (signalResult.kind === "success") {
      expect(renderUrl(locationUrl(signalResult.value.current()))).toBe(
        "https://example.test/back"
      )
    }

    const first = execution.cancel()
    const second = execution.cancel()
    expect(first).toBe(second)
    await first
    await fixture.loader.shutdown()

    const cancelled = browserNavigation("https://example.test/pending")
    const pendingNavigation = await cancelled.load()
    const pendingExecution = createEffectExecution()
    const pending = pendingNavigation
      .nextChange(pendingExecution.context)
      .catch((error: unknown) => error)
    await pendingExecution.cancel()
    expect(isEffectCancellation(await pending)).toBe(true)
    await cancelled.loader.shutdown()
  })
})
