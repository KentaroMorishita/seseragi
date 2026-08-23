import {
  createDomMount,
  createDomTarget,
  type Dom,
  type DomDispatch,
  type DomError,
  type DomMount,
  type DomOptions,
  type DomRuntimeError,
  type DomTarget,
  domTargetValue,
} from "../dom"
import { type Unit, unit } from "../effect"
import {
  type DomEventHandler,
  type DomEventResolution,
  type DomRender,
  type Html,
  messageFromDomEvent,
  renderForDom,
  resolveDomEvent,
} from "../html"
import { type ServiceResult, serviceFailure, serviceSuccess } from "../service"
import {
  type Signal,
  type Subscription,
  subscribe,
  unsubscribe,
} from "../signal"
import { createImeInputCoordinator } from "./ime-input"

export type BrowserDom = Readonly<{
  readonly service: Dom
  readonly dispose: () => Promise<void>
}>

export type DomEventBindings<Action> = Readonly<{
  readonly replace: (render: DomRender<Action>) => void
  readonly handler: (id: string) => DomEventHandler<Action> | undefined
}>

export const BROWSER_DOM_EVENT_BINDINGS = Object.freeze([
  Object.freeze({ nativeKind: "click", handlerKind: "click", capture: false }),
  Object.freeze({
    nativeKind: "focusin",
    handlerKind: "focus",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "focusout",
    handlerKind: "blur",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "keydown",
    handlerKind: "keydown",
    capture: false,
  }),
  Object.freeze({ nativeKind: "keyup", handlerKind: "keyup", capture: false }),
  Object.freeze({
    nativeKind: "mousedown",
    handlerKind: "mousedown",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "mouseup",
    handlerKind: "mouseup",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "pointerdown",
    handlerKind: "pointerdown",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "pointerup",
    handlerKind: "pointerup",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "dblclick",
    handlerKind: "dblclick",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "contextmenu",
    handlerKind: "contextmenu",
    capture: false,
  }),
  Object.freeze({ nativeKind: "scroll", handlerKind: "scroll", capture: true }),
  Object.freeze({ nativeKind: "input", handlerKind: "input", capture: false }),
  Object.freeze({
    nativeKind: "change",
    handlerKind: "change",
    capture: false,
  }),
  Object.freeze({
    nativeKind: "submit",
    handlerKind: "submit",
    capture: false,
  }),
] as const)

export function applyDomEventResolution<Action>(
  event: Pick<Event, "preventDefault" | "stopPropagation">,
  resolution: DomEventResolution<Action>,
  enqueue: (action: Action) => void
): void {
  if (resolution.preventDefault) event.preventDefault()
  if (resolution.stopPropagation) event.stopPropagation()
  if (resolution.kind === "dispatch") enqueue(resolution.action)
}

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
    mount<Failure, Action>(
      options: DomOptions,
      target: DomTarget,
      dispatch: DomDispatch<Failure, Action>,
      content: Signal<Html<Action>>
    ): Promise<ServiceResult<DomError, DomMount<Failure>>> {
      const element = domTargetValue(target)
      if (!(element instanceof document.defaultView!.Element)) {
        return Promise.resolve(
          serviceFailure<DomError>({
            tag: "DomOperationFailed",
            value: "invalid DOM target",
          })
        )
      }
      if (activeTargets.has(element)) {
        return Promise.resolve(
          serviceFailure<DomError>({ tag: "DomTargetAlreadyMounted" })
        )
      }
      if (!element.isConnected) {
        return Promise.resolve(
          serviceFailure<DomError>({ tag: "DomTargetRemoved" })
        )
      }
      activeTargets.add(element)

      return new Promise<ServiceResult<DomError, DomMount<Failure>>>(
        (resolveMount, rejectMount) => {
          let resolveCompletion!: (
            result: ServiceResult<DomRuntimeError<Failure>, Unit>
          ) => void
          let rejectCompletion!: (error: unknown) => void
          const completion = new Promise<
            ServiceResult<DomRuntimeError<Failure>, Unit>
          >((resolve, reject) => {
            resolveCompletion = resolve
            rejectCompletion = reject
          })
          let subscription: Subscription | undefined
          let settled = false
          let interactive = false
          let releaseCancellation: (() => void) | undefined
          let queuedEvents = 0
          let eventQueue = Promise.resolve()
          let deferredTree: Html<Action> | undefined
          let restoringFocus = false
          let initialRender = true
          let initialFailure: DomError | undefined
          const bindings = createDomEventBindings<Action>()
          const ime = createImeInputCoordinator<HTMLElement>()
          const imeTimers = new Map<HTMLElement, number>()
          const targetObserver = new document.defaultView!.MutationObserver(
            () => {
              if (!element.isConnected) {
                void finish(
                  serviceFailure({
                    tag: "DomFailure",
                    value: { tag: "DomTargetRemoved" },
                  })
                )
              }
            }
          )

          const cleanup = async (applyCleanup = true): Promise<boolean> => {
            if (settled) return false
            settled = true
            interactive = false
            releaseCancellation?.()
            releaseCancellation = undefined
            activeTargets.delete(element)
            disposers.delete(dispose)
            targetObserver.disconnect()
            if (subscription !== undefined) {
              await unsubscribe(subscription)({})
            }
            for (const timer of imeTimers.values()) {
              document.defaultView!.clearTimeout(timer)
            }
            imeTimers.clear()
            ime.reset()
            for (const [kind, listener, capture] of listeners) {
              element.removeEventListener(kind, listener, capture)
            }
            if (applyCleanup && options.cleanup.tag === "ClearRenderedDom") {
              element.replaceChildren()
            }
            return true
          }

          const finish = async (
            result: ServiceResult<DomRuntimeError<Failure>, Unit>,
            applyCleanup = true
          ): Promise<void> => {
            if (await cleanup(applyCleanup)) resolveCompletion(result)
          }

          const finishDefect = async (error: unknown): Promise<void> => {
            if (await cleanup()) rejectCompletion(error)
          }

          const enqueue = (action: Action, after?: () => void): void => {
            if (settled) return
            if (queuedEvents >= options.eventCapacity) {
              void finish(
                serviceFailure({
                  tag: "DomFailure",
                  value: {
                    tag: "DomEventQueueOverflow",
                    value: options.eventCapacity,
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
              .catch(finishDefect)
              .finally(() => {
                queuedEvents -= 1
              })
          }

          const listeners: Array<readonly [string, EventListener, boolean]> = []
          const listen = (
            kind: string,
            listener: EventListener,
            capture = false
          ): void => {
            element.addEventListener(kind, listener, capture)
            listeners.push([kind, listener, capture])
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
              void finishDefect(error)
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

          for (const {
            nativeKind,
            handlerKind,
            capture,
          } of BROWSER_DOM_EVENT_BINDINGS) {
            const listener: EventListener = (event: Event): void => {
              if (settled || !interactive) return
              const eventTarget = event.target
              if (!(eventTarget instanceof document.defaultView!.Element))
                return
              const matched = eventTarget.closest<HTMLElement>(
                `[data-ssrg-event-${handlerKind}]`
              )
              if (matched === null || !element.contains(matched)) return
              const id = matched.getAttribute(`data-ssrg-event-${handlerKind}`)
              if (id === null) return
              const handler = bindings.handler(id)
              if (handler === undefined || handler.kind !== handlerKind) return
              if (handlerKind === "focus" && restoringFocus) return
              if (
                handlerKind === "input" &&
                !ime.input(matched, nativeInputIsComposing(event))
              ) {
                return
              }
              if (handlerKind === "submit" && ime.busy()) commitCompositions()
              try {
                const resolution = resolveDomEvent(handler, matched, event)
                applyDomEventResolution(event, resolution, (action) =>
                  enqueue(
                    action,
                    handlerKind === "submit" ? flushDeferredRender : undefined
                  )
                )
              } catch (error) {
                void finishDefect(error)
              }
            }
            listen(nativeKind, listener, capture)
          }

          for (const kind of [
            "compositionstart",
            "compositionupdate",
            "compositionend",
          ] as const) {
            const listener: EventListener = (event: Event): void => {
              if (settled || !interactive) return
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
            if (!element.isConnected) {
              void finish(
                serviceFailure({
                  tag: "DomFailure",
                  value: { tag: "DomTargetRemoved" },
                })
              )
              return
            }
            if (ime.busy()) {
              deferredTree = tree
              return
            }
            deferredTree = undefined
            const focus = captureFocusedControl(element, document)
            const snapshot = renderForDom(tree)
            bindings.replace(snapshot)
            const expected = domFragment(document, snapshot)
            if (initialRender) {
              initialRender = false
              if (options.hydration.tag === "FreshMount") {
                element.replaceChildren(expected.cloneNode(true))
              } else {
                const mismatch = firstDomMismatch(
                  element.childNodes,
                  expected.childNodes,
                  []
                )
                if (
                  mismatch !== undefined &&
                  options.hydration.tag === "HydrateStrict"
                ) {
                  initialFailure = {
                    tag: "HydrationMismatch",
                    value: mismatch,
                  }
                  return
                }
                attachHydratedChildren(element, expected)
              }
            } else {
              element.replaceChildren(expected.cloneNode(true))
            }
            restoringFocus = true
            try {
              restoreFocusedControl(element, focus)
            } finally {
              restoringFocus = false
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
            .then(async (activeSubscription) => {
              subscription = activeSubscription
              if (settled) {
                void unsubscribe(activeSubscription)({})
                return
              }
              if (initialFailure !== undefined) {
                await finish(serviceSuccess(unit), false)
                resolveMount(serviceFailure(initialFailure))
                return
              }
              interactive = true
              targetObserver.observe(document, {
                childList: true,
                subtree: true,
              })
              mounted()
              resolveMount(
                serviceSuccess(
                  createDomMount<Failure>(
                    Object.freeze({
                      awaitResult: () => completion,
                      unmount: async () => {
                        await finish(serviceSuccess(unit))
                      },
                      bindCancellation(release) {
                        if (settled) {
                          release()
                          return
                        }
                        releaseCancellation?.()
                        releaseCancellation = release
                      },
                    })
                  )
                )
              )
            })
            .catch(async (error) => {
              await cleanup()
              rejectMount(error)
            })
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

type DomMismatch = Readonly<{
  readonly path: readonly number[]
  readonly expected: string
  readonly actual: string
}>

function domFragment<Action>(
  document: Document,
  render: DomRender<Action>
): DocumentFragment {
  const template = document.createElement("template")
  template.innerHTML = render.html
  return template.content
}

function firstDomMismatch(
  actual: NodeListOf<ChildNode> | NodeList,
  expected: NodeListOf<ChildNode> | NodeList,
  path: readonly number[]
): DomMismatch | undefined {
  const length = Math.max(actual.length, expected.length)
  for (let index = 0; index < length; index += 1) {
    const mismatch = nodeMismatch(actual.item(index), expected.item(index), [
      ...path,
      index,
    ])
    if (mismatch !== undefined) return mismatch
  }
  return undefined
}

function nodeMismatch(
  actual: Node | null,
  expected: Node | null,
  path: readonly number[]
): DomMismatch | undefined {
  if (actual === null || expected === null) {
    return Object.freeze({
      path,
      expected: describeNode(expected),
      actual: describeNode(actual),
    })
  }
  if (actual.nodeType !== expected.nodeType) {
    return Object.freeze({
      path,
      expected: describeNode(expected),
      actual: describeNode(actual),
    })
  }
  if (actual.nodeType === 3) {
    return actual.nodeValue === expected.nodeValue
      ? undefined
      : Object.freeze({
          path,
          expected: describeNode(expected),
          actual: describeNode(actual),
        })
  }
  if (actual.nodeType !== 1 || expected.nodeType !== 1) {
    return actual.nodeValue === expected.nodeValue
      ? undefined
      : Object.freeze({
          path,
          expected: describeNode(expected),
          actual: describeNode(actual),
        })
  }
  const actualElement = actual as Element
  const expectedElement = expected as Element
  if (
    actualElement.localName !== expectedElement.localName ||
    actualElement.namespaceURI !== expectedElement.namespaceURI ||
    !sameDomAttributes(actualElement, expectedElement)
  ) {
    return Object.freeze({
      path,
      expected: describeNode(expected),
      actual: describeNode(actual),
    })
  }
  return firstDomMismatch(
    actualElement.childNodes,
    expectedElement.childNodes,
    path
  )
}

function sameDomAttributes(actual: Element, expected: Element): boolean {
  const actualAttributes = comparableAttributes(actual)
  const expectedAttributes = comparableAttributes(expected)
  if (actualAttributes.size !== expectedAttributes.size) return false
  for (const [name, value] of expectedAttributes) {
    if (actualAttributes.get(name) !== value) return false
  }
  return true
}

function comparableAttributes(element: Element): ReadonlyMap<string, string> {
  return new Map(
    [...element.attributes]
      .filter(({ name }) => !name.startsWith("data-ssrg-event-"))
      .map(({ name, value }) => [name, value])
  )
}

function describeNode(node: Node | null): string {
  if (node === null) return "missing"
  if (node.nodeType === 3) return JSON.stringify(node.nodeValue ?? "")
  if (node.nodeType === 1) return (node as Element).outerHTML
  return node.nodeName
}

/** Initial hydration attachment only; mount-time updates do not use this. */
function attachHydratedChildren(actual: Node, expected: Node): void {
  let index = 0
  while (index < expected.childNodes.length) {
    const expectedChild = expected.childNodes.item(index)
    const actualChild = actual.childNodes.item(index)
    if (actualChild === null) {
      actual.appendChild(expectedChild.cloneNode(true))
      index += 1
      continue
    }
    if (!sameDomNodeKind(actualChild, expectedChild)) {
      actual.replaceChild(expectedChild.cloneNode(true), actualChild)
      index += 1
      continue
    }
    if (actualChild.nodeType === 3) {
      if (actualChild.nodeValue !== expectedChild.nodeValue) {
        actualChild.nodeValue = expectedChild.nodeValue
      }
      index += 1
      continue
    }
    if (actualChild.nodeType === 1 && expectedChild.nodeType === 1) {
      reconcileAttributes(actualChild as Element, expectedChild as Element)
      attachHydratedChildren(actualChild, expectedChild)
    }
    index += 1
  }
  while (actual.childNodes.length > expected.childNodes.length) {
    actual.lastChild?.remove()
  }
}

function sameDomNodeKind(actual: Node, expected: Node): boolean {
  if (actual.nodeType !== expected.nodeType) return false
  if (actual.nodeType !== 1 || expected.nodeType !== 1) return true
  const actualElement = actual as Element
  const expectedElement = expected as Element
  return (
    actualElement.localName === expectedElement.localName &&
    actualElement.namespaceURI === expectedElement.namespaceURI
  )
}

function reconcileAttributes(actual: Element, expected: Element): void {
  const expectedNames = new Set<string>()
  for (const { name, value } of [...expected.attributes]) {
    expectedNames.add(name)
    if (actual.getAttribute(name) !== value) actual.setAttribute(name, value)
  }
  for (const { name } of [...actual.attributes]) {
    if (!expectedNames.has(name)) actual.removeAttribute(name)
  }
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
