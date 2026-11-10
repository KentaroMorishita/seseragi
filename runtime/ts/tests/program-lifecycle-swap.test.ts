import { expect, test } from "bun:test"

// A fresh module instance: a page installs its reload listeners only once.
const { installProgramReload } = await import(
  "../src/browser/program-lifecycle.js?swap-event"
)
test("the dev swap event answers with the lifetime's swap", async () => {
  class Host extends EventTarget {
    location = {
      reload() {
        throw new Error("no reload expected")
      },
    }
  }
  const host = new Host()
  installProgramReload(host)
  let answered: Promise<void> | undefined
  const event = new CustomEvent("seseragi:program-update", {
    cancelable: true,
    detail: {
      version: "2",
      programs: ["missing"],
      respond(done: Promise<void>) {
        answered = done
      },
    },
  })
  expect(host.dispatchEvent(event)).toBe(false)
  await answered
})
