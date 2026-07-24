import { describe, expect, test } from "bun:test"
import {
  app,
  createDomTarget,
  type Dom,
  type DomDispatch,
  type DomOptions,
  type DomRuntimeError,
  type DomTarget,
  defaultOptions,
  run as runDom,
} from "../../../runtime/ts/src/dom"
import {
  type Effect,
  run,
  type Unit,
  unit,
} from "../../../runtime/ts/src/effect"
import {
  button,
  type Html,
  p,
  renderForDom,
  renderToString,
} from "../../../runtime/ts/src/html"
import {
  type ServiceOperation,
  serviceFailure,
  serviceSuccess,
} from "../../../runtime/ts/src/service"
import { constant, type Signal } from "../../../runtime/ts/src/signal"

type Mode = "Ready" | "Active"
type Action = "Activate"

describe("high-level DOM app runtime", () => {
  test("executes a child-owned Effect value without a root Action union", async () => {
    let childUpdates = 0
    const childAction: Effect<Record<string, never>, never, Unit> = () => {
      childUpdates += 1
      return unit
    }
    const content = constant(
      button({ onClick: childAction, children: "Run child action" })
    )
    const target = createDomTarget({ selector: "#app" })
    const service: Dom = {
      query() {
        return serviceSuccess(target)
      },
      async run<Failure, Action>(
        _options: DomOptions,
        _target: DomTarget,
        dispatch: DomDispatch<Failure, Action>,
        mounted: Signal<Html<Action>>
      ): Promise<Awaited<ServiceOperation<DomRuntimeError<Failure>, Unit>>> {
        const handler = renderForDom(mounted.current()).eventHandlers.get("0")
        if (handler?.kind !== "click") {
          throw new Error("expected child click action")
        }
        const result = await dispatch(handler.message)
        return result.kind === "failure"
          ? serviceFailure({ tag: "DispatchFailure", value: result.error })
          : serviceSuccess(unit)
      },
    }

    const result = await run(
      runDom(
        defaultOptions(unit),
        target,
        (action: Effect<Record<string, never>, never, Unit>) => action,
        content
      ),
      { dom: service }
    )

    expect(result).toEqual({ kind: "success", value: unit })
    expect(childUpdates).toBe(1)
  })

  test("owns Signal setup and applies actions through a pure reducer", async () => {
    const snapshots: string[] = []
    const service: Dom = {
      query(selector) {
        expect(selector).toBe("#app")
        return serviceSuccess(createDomTarget({ selector }))
      },
      async run<Failure, Action>(
        _options: DomOptions,
        _target: DomTarget,
        dispatch: DomDispatch<Failure, Action>,
        content: Signal<Html<Action>>
      ): Promise<Awaited<ServiceOperation<DomRuntimeError<Failure>, Unit>>> {
        snapshots.push(renderToString(content.current()))
        const result = await dispatch("Activate" as Action)
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
      app<Mode, Action>({
        target: "#app",
        initial: "Ready",
        update: (action) => (_mode) =>
          action === "Activate" ? "Active" : "Ready",
        view: (mode) =>
          mode === "Ready"
            ? button({ onClick: "Activate" as Action, children: "Start" })
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
      app<Mode, Action>({
        target: "#missing",
        initial: "Ready",
        update: (_action) => (mode) => mode,
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
