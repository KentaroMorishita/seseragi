import type {
  CompilerMethod,
  CompilerReply,
  CompilerRequest,
  CompilerResult,
  CompilerService,
} from "./compiler-worker-protocol"

type WorkerTransport = Pick<
  Worker,
  "postMessage" | "addEventListener" | "terminate"
>

export function createCompilerWorkerClient(
  createWorker: () => WorkerTransport
) {
  let worker: WorkerTransport | undefined
  let nextId = 0
  const pending = new Map<
    number,
    { resolve: (result: unknown) => void; reject: (error: Error) => void }
  >()

  const fail = (error: Error): void => {
    worker?.terminate()
    worker = undefined
    for (const request of pending.values()) request.reject(error)
    pending.clear()
  }

  const getWorker = (): WorkerTransport => {
    if (worker !== undefined) return worker
    const transport = createWorker()
    worker = transport
    const failWorker = (error: Error): void => {
      if (worker === transport) fail(error)
    }
    transport.addEventListener(
      "message",
      (event: MessageEvent<CompilerReply>) => {
        const reply = event.data
        const request = pending.get(reply.id)
        if (request === undefined) return
        pending.delete(reply.id)
        if (reply.ok) request.resolve(reply.result)
        else request.reject(new Error(reply.error))
      }
    )
    transport.addEventListener("error", (event: ErrorEvent) =>
      failWorker(new Error(event.message || "Compiler worker failed"))
    )
    transport.addEventListener("messageerror", () =>
      failWorker(new Error("Compiler worker response could not be read"))
    )
    return transport
  }

  return {
    call: <Method extends CompilerMethod>(
      method: Method,
      ...args: Parameters<CompilerService[Method]>
    ): Promise<CompilerResult<Method>> =>
      new Promise((resolve, reject) => {
        try {
          const transport = getWorker()
          const id = ++nextId
          pending.set(id, {
            resolve: (result) => resolve(result as CompilerResult<Method>),
            reject,
          })
          transport.postMessage({ id, method, args } as CompilerRequest)
        } catch (error) {
          fail(error instanceof Error ? error : new Error(String(error)))
          reject(error)
        }
      }),
  }
}
