import { PassThrough, type Transform } from "node:stream"
import {
  createBrotliDecompress,
  createGunzip,
  createInflate,
  createInflateRaw,
} from "node:zlib"
import type { HttpClientEvent } from "@seseragi/runtime/http-client"
import type {
  ProviderSubscriptionObserver,
  ProviderSubscriptionRegistration,
} from "@seseragi/runtime/provider"

type RawExchange = (
  value: unknown,
  observer: unknown,
  attachment: unknown
) => ProviderSubscriptionRegistration

const chunkLimit = 64 * 1024
const encodingDepthLimit = 8

/** Native transports expose transfer-decoded bytes; Fetch already decodes them. */
export function createDecodedHttpExchange(
  createRawExchange: RawExchange,
  value: unknown,
  observerValue: unknown,
  attachment: unknown
): ProviderSubscriptionRegistration {
  const observer = observerValue as ProviderSubscriptionObserver
  if (
    typeof observer?.next !== "function" ||
    typeof observer.complete !== "function" ||
    typeof observer.failure !== "function" ||
    typeof observer.defect !== "function"
  ) {
    throw new TypeError("HTTP exchange observer is invalid")
  }
  const pending: HttpClientEvent[] = []
  const decoders: Array<Readonly<{ input: Transform; output: Transform }>> = []
  const decodingStreams: Transform[] = []
  let raw: ProviderSubscriptionRegistration | undefined
  let demand = 0
  let rawPending = false
  let rawEnded = false
  let writePending = false
  let receivedBody = false
  let decoderEnded = false
  let trailers: HttpClientEvent | undefined
  let terminal = false
  let stopped = false
  let stopping: Promise<void> | undefined
  let draining = false
  let again = false

  const destroyDecoders = (): void => {
    for (const decoder of decodingStreams) decoder.destroy()
  }
  const stop = (): Promise<void> => {
    if (stopping !== undefined) return stopping
    stopped = true
    pending.length = 0
    trailers = undefined
    const closed = decodingStreams.map(
      (decoder) =>
        new Promise<void>((resolve) => {
          if (decoder.closed) resolve()
          else decoder.once("close", resolve)
          decoder.destroy()
        })
    )
    // Assign the shared Promise before invoking transport cleanup, including a
    // pending upload finalizer. Every unsubscribe joins the same completion.
    stopping = Promise.resolve()
      .then(() => raw?.unsubscribe())
      .then(() => undefined)
      .finally(async () => {
        destroyDecoders()
        await Promise.all(closed)
      })
    return stopping
  }
  const terminate = (kind: "failure" | "defect", cause: unknown): void => {
    if (terminal || stopped) return
    terminal = true
    void stop().catch(() => undefined)
    observer[kind](cause)
  }
  const protocolFailure = (cause: unknown): void => {
    terminate("failure", {
      tag: "HttpProtocolFailure",
      value:
        cause instanceof Error
          ? `HTTP content decoding failed: ${cause.message}`
          : "HTTP content decoding failed",
    })
  }

  const drain = (): void => {
    if (draining) {
      again = true
      return
    }
    if (terminal || stopped) return
    draining = true
    try {
      do {
        again = false
        while (demand > 0 && !terminal && !stopped) {
          const next = pending.shift()
          if (next !== undefined) {
            demand -= 1
            observer.next(next)
            continue
          }
          const decoded = decoders.at(-1)?.output
          if (decoded !== undefined) {
            const available = Math.min(decoded.readableLength, chunkLimit)
            if (available > 0) {
              const chunk = decoded.read(available) as Buffer | null
              if (chunk !== null && chunk.length > 0) {
                demand -= 1
                observer.next({
                  kind: "ResponseBodyChunk",
                  bytes: new Uint8Array(chunk),
                })
                continue
              }
            } else {
              decoded.read(0)
            }
            if (decoderEnded && trailers !== undefined) {
              demand -= 1
              const nextTrailers = trailers
              trailers = undefined
              observer.next(nextTrailers)
              continue
            }
          }
          if (rawEnded || rawPending || writePending || raw === undefined) break
          rawPending = true
          try {
            const requested = raw.demand(1)
            if (requested !== undefined) {
              void Promise.resolve(requested).catch((cause) => {
                terminate("defect", cause)
              })
            }
          } catch (cause) {
            terminate("defect", cause)
          }
          // A raw provider may synchronously satisfy demand. Re-enter only when
          // it did so; asynchronous readable/write callbacks will wake us later.
          if (rawPending) break
        }
        if (
          rawEnded &&
          pending.length === 0 &&
          (decoders.length === 0 || decoderEnded) &&
          trailers === undefined &&
          !terminal &&
          !stopped
        ) {
          terminal = true
          observer.complete()
        }
      } while (again && !terminal && !stopped)
    } catch (cause) {
      protocolFailure(cause)
    } finally {
      draining = false
    }
  }

  const configure = (
    event: Extract<HttpClientEvent, { kind: "ResponseStarted" }>
  ) => {
    const method = (value as { method: string }).method
    if (
      method === "HEAD" ||
      event.head.status === 204 ||
      event.head.status === 205 ||
      event.head.status === 304
    )
      return
    const codings = event.head.headers
      .filter(({ name }) => name.toLowerCase() === "content-encoding")
      .flatMap(({ value }) => value.toLowerCase().split(","))
      .map((value) => value.trim())
    // Fetch leaves the entire representation unchanged when a coding is not
    // supported. Never decode only a recognized suffix of an unknown stack.
    if (
      codings.length === 0 ||
      codings.some(
        (coding) =>
          !["gzip", "x-gzip", "deflate", "x-deflate", "br"].includes(coding)
      )
    )
      return
    if (codings.length > encodingDepthLimit)
      throw new TypeError("HTTP content encoding stack exceeds 8")
    const options = {
      chunkSize: 16 * 1024,
      readableHighWaterMark: chunkLimit,
      writableHighWaterMark: chunkLimit,
    }
    const register = <Stream extends Transform>(stream: Stream): Stream => {
      stream.on("error", protocolFailure)
      decodingStreams.push(stream)
      return stream
    }
    for (const coding of codings.reverse()) {
      let input: Transform
      let output: Transform
      if (coding === "deflate" || coding === "x-deflate") {
        input = register(new PassThrough(options))
        output = register(new PassThrough(options))
        let initialized = false
        input.once("readable", () => {
          if (stopped) return
          try {
            const prefix = input.read(1) as Buffer | null
            if (prefix === null) return
            initialized = true
            // Fetch tolerates HTTP servers sending raw deflate. Sniff before
            // decoding so no full-body replay buffer is needed for that fallback.
            const decoder = register(
              (prefix[0]! & 0x0f) === 8
                ? createInflate(options)
                : createInflateRaw(options)
            )
            input.unshift(prefix)
            input.pipe(decoder).pipe(output)
          } catch (cause) {
            protocolFailure(cause)
          }
        })
        input.once("end", () => {
          if (!initialized && !stopped) output.end()
        })
      } else {
        input = output = register(
          coding === "br"
            ? createBrotliDecompress(options)
            : createGunzip(options)
        )
      }
      decoders.at(-1)?.output.pipe(input)
      decoders.push({ input, output })
    }
    const decoded = decoders.at(-1)!.output
    decoded.on("readable", drain)
    decoded.once("end", () => {
      decoderEnded = true
      drain()
    })
  }

  try {
    raw = createRawExchange(
      value,
      {
        next(next: unknown) {
          if (terminal || stopped) return
          rawPending = false
          try {
            const event = next as HttpClientEvent
            if (event.kind === "ResponseStarted") {
              configure(event)
              pending.push(event)
            } else if (
              event.kind === "ResponseBodyChunk" &&
              decoders.length > 0
            ) {
              receivedBody = true
              writePending = true
              // Read decoded output concurrently with this write. Waiting for its
              // callback before reading would deadlock on a high expansion ratio.
              decoders[0]!.input.write(event.bytes, (cause) => {
                writePending = false
                if (cause !== null && cause !== undefined)
                  protocolFailure(cause)
                else drain()
              })
            } else if (
              event.kind === "ResponseTrailers" &&
              decoders.length > 0
            ) {
              trailers = event
            } else {
              pending.push(event)
            }
            drain()
          } catch (cause) {
            protocolFailure(cause)
          }
        },
        complete() {
          if (terminal || stopped) return
          rawPending = false
          rawEnded = true
          if (decoders.length > 0) {
            if (receivedBody) decoders[0]!.input.end()
            else {
              // Fetch accepts an empty HTTP body even when its representation
              // headers describe a compressed resource (e.g. an empty 200).
              decoderEnded = true
              destroyDecoders()
            }
          }
          drain()
        },
        failure(cause: unknown) {
          terminate("failure", cause)
        },
        defect(cause: unknown) {
          terminate("defect", cause)
        },
      },
      attachment
    )
  } catch (cause) {
    destroyDecoders()
    throw cause
  }
  if (stopped) void stop().catch(() => undefined)

  return Object.freeze({
    demand(count: number) {
      if (terminal || stopped) return
      demand += count
      drain()
    },
    unsubscribe: stop,
  })
}
