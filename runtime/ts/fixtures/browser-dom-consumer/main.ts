import { createBrowserDom } from "@seseragi/runtime/browser/dom"
import { defaultOptions } from "@seseragi/runtime/dom"
import { unit } from "@seseragi/runtime/effect"
import { button } from "@seseragi/runtime/html"
import { serviceSuccess } from "@seseragi/runtime/service"
import { constant } from "@seseragi/runtime/signal"

declare global {
  interface Window {
    seseragiConsumer: Readonly<{
      actions: readonly string[]
      dispose: () => Promise<void>
    }>
  }
}

const actions: string[] = []
let announceMounted: () => void = () => undefined
const mounted = new Promise<void>((resolve) => {
  announceMounted = resolve
})
const browserDom = createBrowserDom(document, announceMounted)
const target = await browserDom.service.query("#app")
if (target.kind === "failure") throw new Error(target.error.tag)

const running = Promise.resolve(
  browserDom.service.run(
    defaultOptions(unit),
    target.value,
    async (action: string) => {
      actions.push(action)
      return serviceSuccess(unit)
    },
    constant(button({ onClick: "clicked", children: "Run action" }))
  )
)
await mounted
document.documentElement.dataset.ready = "true"

window.seseragiConsumer = Object.freeze({
  actions,
  async dispose() {
    await browserDom.dispose()
    await running
    document.documentElement.dataset.disposed = "true"
  },
})
