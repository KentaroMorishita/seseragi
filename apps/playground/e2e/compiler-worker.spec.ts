import { expect, test } from "@playwright/test"

test("compiler analysis leaves animation frames and editing responsive", async ({
  page,
}) => {
  await page.addInitScript(() => {
    type RequestAudit = { method: string; frames: number }
    const audit = {
      pending: new Map<number, RequestAudit>(),
      completed: [] as RequestAudit[],
    }
    Object.assign(window, { compilerAudit: audit })
    const tracked = new WeakSet<Worker>()
    const postMessage = Worker.prototype.postMessage
    Worker.prototype.postMessage = function (message, options?) {
      if (!tracked.has(this)) {
        tracked.add(this)
        this.addEventListener("message", (event) => {
          const request = audit.pending.get(event.data.id)
          if (request) audit.completed.push(request)
          audit.pending.delete(event.data.id)
        })
      }
      audit.pending.set(message.id, { method: message.method, frames: 0 })
      Reflect.apply(
        postMessage,
        this,
        options === undefined ? [message] : [message, options]
      )
    }
    const frame = () => {
      for (const request of audit.pending.values()) request.frames++
      requestAnimationFrame(frame)
    }
    requestAnimationFrame(frame)
  })
  const workers: string[] = []
  page.on("worker", (worker) => workers.push(worker.url()))
  await page.goto("/")
  const status = page.locator("#status-text")
  await expect(status).toHaveText("Analysis ready")
  expect(workers.some((url) => url.includes("compiler.worker"))).toBe(true)
  const cdp = await page.context().newCDPSession(page)
  await cdp.send("Emulation.setCPUThrottlingRate", { rate: 4 })
  try {
    const source = [
      ...Array.from(
        { length: 1500 },
        (_, i) => `pub fn step${i} value: Int -> Int = value + ${i}`
      ),
      'pub effect fn main = println "worker ready"',
    ].join("\n")
    const editor = page.getByRole("textbox", { name: "Seseragi source editor" })
    await editor.click()
    await page.keyboard.press("ControlOrMeta+A")
    await page.keyboard.insertText(source)
    await page.waitForFunction(() => {
      const audit = (
        window as unknown as {
          compilerAudit: { pending: Map<number, { method: string }> }
        }
      ).compilerAudit
      return [...audit.pending.values()].some(
        ({ method }) => method === "analyzeProject"
      )
    })
    await page.keyboard.insertText("\n// editing while analysis runs")
    await expect(editor).toContainText("// editing while analysis runs")
    await expect(status).toHaveText("Analysis ready", { timeout: 30_000 })
    const frames = await page.evaluate(() =>
      (
        window as unknown as {
          compilerAudit: { completed: { method: string; frames: number }[] }
        }
      ).compilerAudit.completed
        .filter(({ method }) => method === "analyzeProject")
        .slice(1)
        .map(({ frames }) => frames)
    )
    expect(frames.length).toBeGreaterThan(0)
    expect(Math.max(...frames)).toBeGreaterThan(2)
    await page.locator("#run-button").click()
    await expect(page.locator("#output")).toHaveText("worker ready", {
      timeout: 30_000,
    })
  } finally {
    await cdp.send("Emulation.setCPUThrottlingRate", { rate: 1 })
  }
})
