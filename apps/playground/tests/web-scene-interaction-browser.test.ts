import { afterAll, beforeAll, expect, test } from "bun:test"
import { readFile } from "node:fs/promises"
import { extname, resolve } from "node:path"
import { type Browser, chromium } from "playwright"
import { ensureSeseragiCli, runCommand } from "./cli-test-support"

const root = resolve(import.meta.dir, "../../..")
const fixture = resolve(
  root,
  "examples/spec/fixtures/projects/web-scene-interaction"
)
const output = resolve(root, "target/web-scene-interaction-browser")
let browser: Browser | undefined
let server: ReturnType<typeof Bun.serve> | undefined

beforeAll(async () => {
  const cli = await ensureSeseragiCli()
  await runCommand([cli, "build", fixture, "--out-dir", output])
  server = Bun.serve({
    hostname: "127.0.0.1",
    port: 0,
    async fetch(request) {
      const url = new URL(request.url)
      const relative =
        url.pathname === "/" ? "index.html" : url.pathname.slice(1)
      const path = resolve(output, relative)
      if (!path.startsWith(`${output}/`)) {
        return new Response("Forbidden", { status: 403 })
      }
      const types: Record<string, string> = {
        ".css": "text/css; charset=utf-8",
        ".html": "text/html; charset=utf-8",
        ".js": "text/javascript; charset=utf-8",
      }
      return new Response(await readFile(path), {
        headers: {
          "content-type": types[extname(path)] ?? "application/octet-stream",
        },
      })
    },
  })
  browser = await chromium.launch()
}, 30_000)

afterAll(async () => {
  await browser?.close()
  server?.stop(true)
}, 30_000)

test("runs SVG drag, cancellation, wheel, pinch, resize, and cleanup", async () => {
  if (browser === undefined || server === undefined) {
    throw new Error("web scene browser harness did not start")
  }
  const page = await browser.newPage()
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.addInitScript(() => {
    const state = {
      captures: [] as number[],
      releases: [] as number[],
      order: [] as string[],
      suppressedClicks: 0,
      activeObservers: 0,
      resizeCallbacks: 0,
    }
    Object.defineProperty(window, "sceneAudit", { value: state })
    const nativeCapture = Element.prototype.setPointerCapture
    const nativeRelease = Element.prototype.releasePointerCapture
    const nativePreventDefault = Event.prototype.preventDefault
    const nativeStopPropagation = Event.prototype.stopPropagation
    Event.prototype.preventDefault = function () {
      if (this.type === "pointerdown") state.order.push("preventDefault")
      nativePreventDefault.call(this)
    }
    Event.prototype.stopPropagation = function () {
      if (this.type === "pointerdown") state.order.push("stopPropagation")
      nativeStopPropagation.call(this)
    }
    Element.prototype.setPointerCapture = function (pointerId: number) {
      state.captures.push(pointerId)
      state.order.push("capture")
      try {
        nativeCapture.call(this, pointerId)
      } catch {
        // Synthetic touch pointers have no UA pointer stream; the call itself
        // is the capability-boundary behavior under test.
      }
    }
    Element.prototype.releasePointerCapture = function (pointerId: number) {
      state.releases.push(pointerId)
      try {
        nativeRelease.call(this, pointerId)
      } catch {
        // See the synthetic-pointer note above.
      }
    }
    const NativeResizeObserver = window.ResizeObserver
    window.ResizeObserver = class extends NativeResizeObserver {
      private active = false
      constructor(callback: ResizeObserverCallback) {
        super((entries, observer) => {
          state.resizeCallbacks += 1
          callback(entries, observer)
        })
      }
      override observe(target: Element, options?: ResizeObserverOptions) {
        if (!this.active) {
          this.active = true
          state.activeObservers += 1
        }
        super.observe(target, options)
      }
      override disconnect() {
        if (this.active) {
          this.active = false
          state.activeObservers -= 1
        }
        super.disconnect()
      }
    }
  })
  await page.goto(`http://127.0.0.1:${server.port}/`)
  await page.locator("#scene").waitFor()
  await page.waitForFunction(() =>
    document.querySelector("#status")?.textContent?.includes("size=320")
  )
  await page.evaluate(() => {
    const audit = (
      window as typeof window & {
        sceneAudit: { order: string[]; suppressedClicks: number }
      }
    ).sceneAudit
    document.querySelector("#scene")?.addEventListener("click", () => {
      audit.suppressedClicks += 1
    })
    const status = document.querySelector("#status")
    if (status !== null) {
      new MutationObserver(() => audit.order.push("action")).observe(status, {
        childList: true,
      })
    }
  })

  expect(
    await page.locator("#scene").evaluate((element) => element.namespaceURI)
  ).toBe("http://www.w3.org/2000/svg")
  expect(
    await page.locator("#relation-hit-corridor").getAttribute("stroke-width")
  ).toBe("16")

  const dispatchPointer = async (
    type: string,
    pointerId: number,
    clientX: number,
    clientY: number,
    pointerType = "touch"
  ) => {
    await page.locator("#scene").dispatchEvent(type, {
      pointerId,
      pointerType,
      isPrimary: pointerId === 1,
      clientX,
      clientY,
      pressure: 0.5,
      bubbles: true,
      cancelable: true,
    })
  }

  await dispatchPointer("pointerdown", 1, 80, 70)
  await dispatchPointer("pointermove", 1, 130, 90)
  await dispatchPointer("pointerup", 1, 130, 90)
  const clickAllowed = await page.locator("#scene").evaluate((element) =>
    element.dispatchEvent(
      new PointerEvent("click", {
        pointerId: 1,
        pointerType: "touch",
        bubbles: true,
        cancelable: true,
      })
    )
  )
  expect(clickAllowed).toBe(false)
  await page.waitForFunction(() =>
    document.querySelector("#status")?.textContent?.includes("x=130")
  )
  const eventOrder = await page.evaluate(
    () =>
      (
        window as typeof window & {
          sceneAudit: { order: string[]; suppressedClicks: number }
        }
      ).sceneAudit
  )
  expect(eventOrder.order.slice(0, 4)).toEqual([
    "preventDefault",
    "stopPropagation",
    "capture",
    "action",
  ])
  expect(eventOrder.suppressedClicks).toBe(0)

  await dispatchPointer("pointerdown", 1, 140, 90, "pen")
  await dispatchPointer("pointermove", 1, 180, 90, "pen")
  await dispatchPointer("pointercancel", 1, 180, 90, "pen")
  await page.waitForFunction(() => {
    const status = document.querySelector("#status")?.textContent ?? ""
    return status.includes("x=130") && status.includes("cancelled=True")
  })

  await dispatchPointer("pointerdown", 1, 130, 90)
  await dispatchPointer("pointerdown", 2, 210, 90)
  await dispatchPointer("pointermove", 2, 230, 90)
  await dispatchPointer("pointerup", 2, 230, 90)
  await dispatchPointer("pointerup", 1, 130, 90)
  await page.waitForFunction(() =>
    document.querySelector("#status")?.textContent?.includes("zoom=1.25")
  )

  await page.locator("#scene").dispatchEvent("wheel", {
    deltaY: -25,
    deltaMode: 0,
    clientX: 120,
    clientY: 80,
    bubbles: true,
    cancelable: true,
  })
  await page.waitForFunction(() =>
    document.querySelector("#status")?.textContent?.includes("zoom=1.5")
  )

  await page.locator("#scene").evaluate((element) => {
    ;(element as SVGElement).style.width = "480px"
  })
  await page.waitForFunction(() =>
    document.querySelector("#status")?.textContent?.includes("size=480")
  )

  await dispatchPointer("pointerdown", 8, 150, 95)
  await page.locator("#scene").dispatchEvent("lostpointercapture", {
    pointerId: 8,
    pointerType: "touch",
    bubbles: true,
  })
  await dispatchPointer("pointerdown", 9, 155, 95, "mouse")
  await page.locator("#scene").evaluate((scene) => {
    scene.replaceWith(scene.cloneNode(true))
  })
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 0)))
  await dispatchPointer("pointerdown", 9, 160, 95, "mouse")
  await dispatchPointer("pointerup", 9, 160, 95, "mouse")
  await page.waitForFunction(() =>
    document.querySelector("#status")?.textContent?.includes("pointers=0")
  )

  await page
    .locator("#stop")
    .evaluate((element) => (element as HTMLButtonElement).click())
  await page.waitForFunction(
    () => document.documentElement.dataset.seseragiStatus === "completed"
  )
  expect(await page.locator("#app").innerHTML()).toBe("")
  const audit = await page.evaluate(
    () =>
      (
        window as typeof window & {
          sceneAudit: {
            captures: number[]
            releases: number[]
            order: string[]
            suppressedClicks: number
            activeObservers: number
            resizeCallbacks: number
          }
        }
      ).sceneAudit
  )
  expect(audit).toEqual({
    captures: [1, 1, 1, 2, 8, 9, 9],
    releases: [1, 1, 2, 1, 9],
    order: audit.order,
    suppressedClicks: 0,
    activeObservers: 0,
    resizeCallbacks: audit.resizeCallbacks,
  })
  expect(audit.resizeCallbacks).toBeGreaterThanOrEqual(2)
  expect(errors).toEqual([])
  await page.close()
}, 60_000)
