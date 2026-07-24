import {
  createDomTarget,
  domTargetValue,
  type Dom,
  type DomDispatch,
  type DomError,
  type DomOptions,
  type DomRuntimeError,
  type DomTarget,
} from "../../../../runtime/ts/src/dom"
import { unit, type Unit } from "../../../../runtime/ts/src/effect"
import {
  domEventPreventsDefault,
  type DomEventHandler,
  type DomRender,
  type Html,
  messageFromDomEvent,
  renderForDom,
} from "../../../../runtime/ts/src/html"
import {
  serviceFailure,
  serviceSuccess,
  type ServiceOperation,
  type ServiceResult,
} from "../../../../runtime/ts/src/service"
import {
  type Signal,
  subscribe,
  type Subscription,
  unsubscribe,
} from "../../../../runtime/ts/src/signal"

export type BrowserDom = Readonly<{
  readonly service: Dom
  readonly dispose: () => Promise<void>
}>

export type DomEventBindings<Action> = Readonly<{
  readonly replace: (render: DomRender<Action>) => void
  readonly handler: (id: string) => DomEventHandler<Action> | undefined
}>

export function createDomEventBindings<Action>(): DomEventBindings<Action> {
  let handlers: ReadonlyMap<string, DomEventHandler<Action>> = new Map()
  return Object.freeze({
    replace(render: DomRender<Action>) {
      handlers = render.eventHandlers
    },
    handler(id: string) {
      return handlers.get(id)
    },
  })
}

export function createBrowserDom(
  document: Document,
  mounted: () => void
): BrowserDom {
  const activeTargets = new Set<Element>()
  const disposers = new Set<() => Promise<void>>()

  const service: Dom = {
    query(selector) {
      let target: Element | null
      try {
        target = document.querySelector(selector)
      } catch {
        return serviceFailure<DomError>({
          tag: "InvalidSelector",
          value: selector,
        })
      }
      if (target === null) {
        return serviceFailure<DomError>({
          tag: "DomTargetNotFound",
          value: selector,
        })
      }
      return serviceSuccess(createDomTarget(target))
    },
    run<Failure, Action>(
      options: DomOptions,
      target: DomTarget,
      dispatch: DomDispatch<Failure, Action>,
      content: Signal<Html<Action>>
    ): ServiceOperation<DomRuntimeError<Failure>, Unit> {
      const element = domTargetValue(target)
      if (!(element instanceof document.defaultView!.Element)) {
        return serviceFailure<DomRuntimeError<never>>({
          tag: "DomFailure",
          value: { tag: "DomOperationFailed", value: "invalid DOM target" },
        })
      }
      if (activeTargets.has(element)) {
        return serviceFailure<DomRuntimeError<never>>({
          tag: "DomFailure",
          value: { tag: "DomTargetAlreadyMounted" },
        })
      }
      activeTargets.add(element)

      return new Promise<ServiceResult<DomRuntimeError<Failure>, Unit>>(
        (resolve, reject) => {
          let subscription: Subscription | undefined
          let settled = false
          let announced = false
          let queuedEvents = 0
          let eventQueue = Promise.resolve()
          const bindings = createDomEventBindings<Action>()

          const finish = async (
            result: ServiceResult<DomRuntimeError<Failure>, undefined>
          ): Promise<void> => {
            if (settled) return
            settled = true
            activeTargets.delete(element)
            disposers.delete(dispose)
            if (subscription !== undefined) {
              await unsubscribe(subscription)({})
            }
            for (const [kind, listener] of listeners) {
              element.removeEventListener(kind, listener)
            }
            element.replaceChildren()
            resolve(result)
          }

          const enqueue = (action: Action): void => {
            if (settled) return
            if (queuedEvents >= options.eventCapacity) {
              void finish(
                serviceFailure({
                  tag: "DomFailure",
                  value: {
                    tag: "DomEventQueueOverflow",
                    value: BigInt(options.eventCapacity),
                  },
                })
              )
              return
            }
            queuedEvents += 1
            eventQueue = eventQueue
              .then(async () => {
                if (settled) return
                const result = await dispatch(action)
                if (result.kind === "failure") {
                  await finish(
                    serviceFailure({
                      tag: "DispatchFailure",
                      value: result.error,
                    })
                  )
                }
              })
              .catch(reject)
              .finally(() => {
                queuedEvents -= 1
              })
          }

          const listeners = (
            ["click", "input", "change", "submit"] as const
          ).map((kind) => {
            const listener = (event: Event): void => {
              if (settled) return
              const eventTarget = event.target
              if (!(eventTarget instanceof document.defaultView!.Element))
                return
              const matched = eventTarget.closest<HTMLElement>(
                `[data-ssrg-event-${kind}]`
              )
              if (matched === null || !element.contains(matched)) return
              const id = matched.getAttribute(`data-ssrg-event-${kind}`)
              if (id === null) return
              const handler = bindings.handler(id)
              if (handler === undefined || handler.kind !== kind) return
              if (domEventPreventsDefault(handler)) event.preventDefault()
              try {
                enqueue(messageFromDomEvent(handler, matched))
              } catch (error) {
                reject(error)
              }
            }
            element.addEventListener(kind, listener)
            return [kind, listener] as const
          })

          const dispose = () => finish(serviceSuccess(unit))
          disposers.add(dispose)

          const render = (tree: Html<Action>) => {
            if (settled) return
            const focus = captureFocusedControl(element, document)
            const snapshot = renderForDom(tree)
            bindings.replace(snapshot)
            const template = document.createElement("template")
            template.innerHTML = snapshot.html
            element.replaceChildren(template.content.cloneNode(true))
            restoreFocusedControl(element, focus)
            if (!announced) {
              announced = true
              mounted()
            }
          }

          void Promise.resolve(
            subscribe(
              (tree) => () => {
                render(tree)
                return unit
              },
              content
            )({})
          )
            .then((activeSubscription) => {
              subscription = activeSubscription
              if (settled) void unsubscribe(activeSubscription)({})
            })
            .catch(reject)
        }
      )
    },
  }

  return Object.freeze({
    service,
    async dispose() {
      await Promise.all([...disposers].map((dispose) => dispose()))
    },
  })
}

type FocusedControl = Readonly<{
  readonly id: string
  readonly tagName: string
  readonly selectionStart: number | null
  readonly selectionEnd: number | null
  readonly selectionDirection: "forward" | "backward" | "none" | null
}>

function captureFocusedControl(
  root: Element,
  document: Document
): FocusedControl | undefined {
  const active = document.activeElement
  if (
    active === null ||
    !root.contains(active) ||
    (active.tagName !== "INPUT" && active.tagName !== "TEXTAREA") ||
    active.id === ""
  ) {
    return undefined
  }
  const control = active as HTMLInputElement | HTMLTextAreaElement
  return Object.freeze({
    id: control.id,
    tagName: control.tagName,
    selectionStart: control.selectionStart,
    selectionEnd: control.selectionEnd,
    selectionDirection: control.selectionDirection,
  })
}

function restoreFocusedControl(
  root: Element,
  focus: FocusedControl | undefined
): void {
  if (focus === undefined) return
  const control = [
    ...root.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(
      "input, textarea"
    ),
  ].find(
    (candidate) =>
      candidate.id === focus.id && candidate.tagName === focus.tagName
  )
  if (control === undefined) return
  control.focus({ preventScroll: true })
  if (focus.selectionStart === null || focus.selectionEnd === null) return
  try {
    control.setSelectionRange(
      focus.selectionStart,
      focus.selectionEnd,
      focus.selectionDirection ?? undefined
    )
  } catch {
    // Checked controls do not expose a text selection.
  }
}
