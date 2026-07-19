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
import { renderForDom, type Html } from "../../../../runtime/ts/src/html"
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
    run<Failure, Message>(
      options: DomOptions,
      target: DomTarget,
      dispatch: DomDispatch<Failure, Message>,
      content: Signal<Html<Message>>
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
            element.replaceChildren()
            resolve(result)
          }

          const dispose = () => finish(serviceSuccess(unit))
          disposers.add(dispose)

          const render = (tree: Html<Message>) => {
            if (settled) return
            const snapshot = renderForDom(tree)
            const template = document.createElement("template")
            template.innerHTML = snapshot.html
            element.replaceChildren(template.content.cloneNode(true))

            for (const node of element.querySelectorAll<HTMLElement>(
              "[data-ssrg-click]"
            )) {
              const clickId = node.dataset.ssrgClick
              node.removeAttribute("data-ssrg-click")
              if (clickId === undefined) continue
              if (!snapshot.clickMessages.has(clickId)) continue
              const message = snapshot.clickMessages.get(clickId) as Message
              node.addEventListener("click", () => {
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
                    const result = await dispatch(message)
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
              })
            }
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
