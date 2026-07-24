import {
  createDomTarget,
  type Dom,
  type DomDispatch,
  type DomError,
  type DomOptions,
  type DomRuntimeError,
  type DomTarget,
  domTargetValue,
} from "../../../../runtime/ts/src/dom"
import { type Unit, unit } from "../../../../runtime/ts/src/effect"
import {
  type DomEventHandler,
  type DomRender,
  domEventPreventsDefault,
  type Html,
  messageFromDomEvent,
  renderForDom,
} from "../../../../runtime/ts/src/html"
import {
  type ServiceOperation,
  type ServiceResult,
  serviceFailure,
  serviceSuccess,
} from "../../../../runtime/ts/src/service"
import {
  type Signal,
  type Subscription,
  subscribe,
  unsubscribe,
} from "../../../../runtime/ts/src/signal"
import { createImeInputCoordinator } from "./ime-input"

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
          let deferredTree: Html<Action> | undefined
          const bindings = createDomEventBindings<Action>()
          const ime = createImeInputCoordinator<HTMLElement>()
          const imeTimers = new Map<HTMLElement, number>()

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
            for (const timer of imeTimers.values()) {
              document.defaultView!.clearTimeout(timer)
            }
            imeTimers.clear()
            ime.reset()
            for (const [kind, listener] of listeners) {
              element.removeEventListener(kind, listener)
            }
            element.replaceChildren()
            resolve(result)
          }

          const enqueue = (action: Action, after?: () => void): void => {
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
                  return
                }
                after?.()
              })
              .catch(reject)
              .finally(() => {
                queuedEvents -= 1
              })
          }

          const listeners: Array<readonly [string, EventListener]> = []
          const listen = (kind: string, listener: EventListener): void => {
            element.addEventListener(kind, listener)
            listeners.push([kind, listener])
          }

          const inputHandler = (
            control: HTMLElement
          ): DomEventHandler<Action> | undefined => {
            const id = control.getAttribute("data-ssrg-event-input")
            if (id === null) return undefined
            const handler = bindings.handler(id)
            return handler?.kind === "input" ? handler : undefined
          }

          const flushDeferredRender = (): void => {
            if (ime.busy() || deferredTree === undefined) return
            const tree = deferredTree
            deferredTree = undefined
            render(tree)
          }

          const dispatchInput = (
            control: HTMLElement,
            after?: () => void
          ): void => {
            const handler = inputHandler(control)
            if (handler === undefined) {
              after?.()
              return
            }
            try {
              enqueue(messageFromDomEvent(handler, control), after)
            } catch (error) {
              reject(error)
            }
          }

          const scheduleCompositionCommit = (control: HTMLElement): void => {
            if (imeTimers.has(control)) return
            const timer = document.defaultView!.setTimeout(() => {
              imeTimers.delete(control)
              if (!ime.finalize(control)) return
              dispatchInput(control, flushDeferredRender)
            }, 0)
            imeTimers.set(control, timer)
          }

          const commitCompositions = (): void => {
            const controls = [...ime.targets()].sort((left, right) => {
              if (left === right) return 0
              return left.compareDocumentPosition(right) &
                document.defaultView!.Node.DOCUMENT_POSITION_FOLLOWING
                ? -1
                : 1
            })
            for (const control of controls) {
              const timer = imeTimers.get(control)
              if (timer !== undefined) {
                document.defaultView!.clearTimeout(timer)
                imeTimers.delete(control)
              }
              if (!ime.commit(control)) continue
              if (!ime.finalize(control)) continue
              dispatchInput(control)
            }
          }

          for (const kind of ["click", "input", "change", "submit"] as const) {
            const listener: EventListener = (event: Event): void => {
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
              if (
                kind === "input" &&
                !ime.input(matched, nativeInputIsComposing(event))
              ) {
                return
              }
              if (kind === "submit" && ime.busy()) commitCompositions()
              if (domEventPreventsDefault(handler)) event.preventDefault()
              try {
                enqueue(
                  messageFromDomEvent(handler, matched),
                  kind === "submit" ? flushDeferredRender : undefined
                )
              } catch (error) {
                reject(error)
              }
            }
            listen(kind, listener)
          }

          for (const kind of [
            "compositionstart",
            "compositionupdate",
            "compositionend",
          ] as const) {
            const listener: EventListener = (event: Event): void => {
              if (settled) return
              const eventTarget = event.target
              if (!(eventTarget instanceof document.defaultView!.Element))
                return
              const matched = eventTarget.closest<HTMLElement>(
                "[data-ssrg-event-input]"
              )
              if (
                matched === null ||
                !element.contains(matched) ||
                inputHandler(matched) === undefined
              ) {
                return
              }
              if (kind === "compositionstart") ime.start(matched)
              if (kind === "compositionupdate") ime.update(matched)
              if (kind === "compositionend" && ime.end(matched)) {
                scheduleCompositionCommit(matched)
              }
            }
            listen(kind, listener)
          }

          const dispose = () => finish(serviceSuccess(unit))
          disposers.add(dispose)

          const render = (tree: Html<Action>) => {
            if (settled) return
            if (ime.busy()) {
              deferredTree = tree
              return
            }
            deferredTree = undefined
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

function nativeInputIsComposing(event: Event): boolean {
  return (
    (event as Event & { readonly isComposing?: unknown }).isComposing === true
  )
}
