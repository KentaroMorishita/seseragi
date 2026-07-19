import { describe, expect, test } from "bun:test"
import {
  app,
  createDomTarget,
  type Dom,
  type DomDispatch,
  type DomOptions,
  type DomRuntimeError,
  type DomTarget,
} from "../../../runtime/ts/src/dom"
import { run, unit, type Unit } from "../../../runtime/ts/src/effect"
import {
  button,
  p,
  renderToString,
  type Html,
} from "../../../runtime/ts/src/html"
import {
  serviceFailure,
  serviceSuccess,
  type ServiceOperation,
} from "../../../runtime/ts/src/service"
import type { Signal } from "../../../runtime/ts/src/signal"

type Mode = "Ready" | "Active"
type Msg = "Activate"

describe("high-level DOM app runtime", () => {
  test("owns Signal setup and applies messages through a pure reducer", async () => {
    const snapshots: string[] = []
    const service: Dom = {
      query(selector) {
        expect(selector).toBe("#app")
        return serviceSuccess(createDomTarget({ selector }))
      },
      async run<Failure, Message>(
        _options: DomOptions,
        _target: DomTarget,
        dispatch: DomDispatch<Failure, Message>,
        content: Signal<Html<Message>>
      ): Promise<Awaited<ServiceOperation<DomRuntimeError<Failure>, Unit>>> {
        snapshots.push(renderToString(content.current()))
        const result = await dispatch("Activate" as Message)
        if (result.kind === "failure") {
          return serviceFailure({
            tag: "DispatchFailure",
            value: result.error,
          })
        }
        snapshots.push(renderToString(content.current()))
        return serviceSuccess(unit)
      },
    }

    const result = await run(
      app<Mode, Msg>({
        target: "#app",
        initial: "Ready",
        update: (message) => (_mode) =>
          message === "Activate" ? "Active" : "Ready",
        view: (mode) =>
          mode === "Ready"
            ? button({ onClick: "Activate" as Msg, children: "Start" })
            : p({ children: "Active" }),
      }),
      { dom: service }
    )

    expect(result).toEqual({ kind: "success", value: unit })
    expect(snapshots).toEqual([
      '<button type="button">Start</button>',
      "<p>Active</p>",
    ])
  })

  test("normalizes a missing target to a portable String failure", async () => {
    const service: Dom = {
      query() {
        return serviceFailure({
          tag: "DomTargetNotFound",
          value: "#missing",
        })
      },
      run() {
        throw new Error("run must not be called when query fails")
      },
    }

    const result = await run(
      app<Mode, Msg>({
        target: "#missing",
        initial: "Ready",
        update: (_message) => (mode) => mode,
        view: (_mode) => p({ children: "unused" }),
      }),
      { dom: service }
    )

    expect(result).toEqual({
      kind: "failure",
      error: "DOM target unavailable: #missing",
    })
  })
})
