import { PassThrough } from "node:stream"
import { toInts } from "../src/bytes"
import {
  createEffectExecution,
  type EffectContext,
  isEffectCancellation,
  run,
} from "../src/effect"
import { MAX_INT } from "../src/int"
import { serviceSuccess } from "../src/service"
import {
  createByteStdin,
  createProcessStdin,
  defaultLineLimit,
  defaultReadSize,
  lineLimit,
  lines,
  MAX_LINE_LIMIT,
  MAX_READ_SIZE,
  readChunk,
  readLine,
  readLineWith,
  readSize,
  type StdinByteSource,
} from "../src/stdin"
import { runCollect, take } from "../src/stream"
import { Just, type Maybe, Nothing } from "../src/sum"

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

function size(bytes: number) {
  const result = readSize(bytes)
  require(result.tag === "Right", `invalid test read size ${bytes}`)
  return result.value
}

function limit(bytes: number) {
  const result = lineLimit(bytes)
  require(result.tag === "Right", `invalid test line limit ${bytes}`)
  return result.value
}

function chunkSource(chunks: readonly Uint8Array[]) {
  const queued = chunks.map((chunk) => new Uint8Array(chunk))
  let reads = 0
  const source: StdinByteSource = {
    read() {
      reads += 1
      const chunk = queued.shift()
      return serviceSuccess(
        chunk === undefined ? Nothing : Just(new Uint8Array(chunk))
      )
    },
  }
  return { source, reads: () => reads }
}

const encoder = new TextEncoder()

const invalidReadSize = readSize(0)
require(invalidReadSize.tag === "Left" &&
  invalidReadSize.value.tag === "NonPositiveReadSize", "readSize accepted zero")
const oversizedRead = readSize(MAX_READ_SIZE + 1)
require(oversizedRead.tag === "Left" &&
  oversizedRead.value.tag ===
    "ReadSizeTooLarge", "readSize accepted an oversized value")
const invalidLineLimit = lineLimit(0)
require(invalidLineLimit.tag === "Left" &&
  invalidLineLimit.value.tag ===
    "NonPositiveLineLimit", "lineLimit accepted zero")
const oversizedLine = lineLimit(MAX_LINE_LIMIT + 1)
require(oversizedLine.tag === "Left" &&
  oversizedLine.value.tag ===
    "LineLimitTooLarge", "lineLimit accepted an oversized value")
require(defaultReadSize().value === 64 * 1024, "wrong default read size")
require(defaultLineLimit().value === 1024 * 1024, "wrong line limit")

const chunkedSource = chunkSource([new Uint8Array([1, 2, 3])])
const chunked = createByteStdin(chunkedSource.source)
const firstChunk = await run(readChunk(size(2)), { stdin: chunked })
const secondChunk = await run(readChunk(size(2)), { stdin: chunked })
const chunkEof = await run(readChunk(size(2)), { stdin: chunked })
const stickyChunkEof = await run(readChunk(size(2)), { stdin: chunked })
require(firstChunk.kind === "success" &&
  firstChunk.value.tag === "Just" &&
  JSON.stringify(toInts(firstChunk.value.value)) ===
    "[1,2]", "readChunk did not respect its requested size")
require(secondChunk.kind === "success" &&
  secondChunk.value.tag === "Just" &&
  JSON.stringify(toInts(secondChunk.value.value)) ===
    "[3]", "readChunk lost its buffered suffix")
require(chunkEof.kind === "success" &&
  chunkEof.value === Nothing &&
  stickyChunkEof.kind === "success" &&
  stickyChunkEof.value === Nothing &&
  chunkedSource.reads() === 2, "readChunk EOF was not sticky")

const lineSource = chunkSource([encoder.encode("first\r\n\nbare\rcut\nlast")])
const lineStdin = createByteStdin(lineSource.source)
const observedLines: Array<Maybe<string>> = []
for (let index = 0; index < 5; index += 1) {
  const result = await run(readLine(), { stdin: lineStdin })
  require(result.kind === "success", "line fixture returned a failure")
  observedLines.push(result.value)
}
require(JSON.stringify(observedLines) ===
  JSON.stringify([
    Just("first"),
    Just(""),
    Just("bare\rcut"),
    Just("last"),
    Nothing,
  ]), "line ending, empty line, or final unterminated line behavior changed")

const limited = createByteStdin(
  chunkSource([encoder.encode("long\nok\n")]).source
)
const tooLong = await run(readLineWith(limit(3)), { stdin: limited })
const afterTooLong = await run(readLineWith(limit(3)), { stdin: limited })
require(tooLong.kind === "failure" &&
  tooLong.error.tag === "StdinLineTooLong" &&
  tooLong.error.value.limitBytes ===
    3, "long line did not produce StdinLineTooLong")
require(afterTooLong.kind === "success" &&
  afterTooLong.value.tag === "Just" &&
  afterTooLong.value.value === "ok", "long line was not discarded through LF")

const invalidUtf8 = createByteStdin(
  chunkSource([
    new Uint8Array([
      0x6f, 0x6b, 0x0a, 0x61, 0xc0, 0x80, 0x0a, 0x6f, 0x6b, 0x0a,
    ]),
  ]).source
)
const beforeInvalid = await run(readLine(), { stdin: invalidUtf8 })
const invalidLine = await run(readLine(), { stdin: invalidUtf8 })
const afterInvalid = await run(readLine(), { stdin: invalidUtf8 })
require(beforeInvalid.kind === "success" &&
  beforeInvalid.value.tag === "Just" &&
  beforeInvalid.value.value === "ok", "invalid UTF-8 setup line failed")
require(invalidLine.kind === "failure" &&
  invalidLine.error.tag === "InvalidStdinUtf8" &&
  invalidLine.error.value.offset ===
    4, "invalid UTF-8 did not report its first absolute byte offset")
require(afterInvalid.kind === "success" &&
  afterInvalid.value.tag === "Just" &&
  afterInvalid.value.value ===
    "ok", "invalid UTF-8 line was not discarded through LF")

let resolveLease:
  | ((value: ReturnType<typeof serviceSuccess<Maybe<Uint8Array>>>) => void)
  | undefined
const leased = createByteStdin({
  read() {
    return new Promise((resolve) => {
      resolveLease = resolve
    })
  },
})
const leasePending = run(readLine(), { stdin: leased })
const concurrent = await run(readLine(), { stdin: leased })
require(concurrent.kind === "failure" &&
  concurrent.error.tag ===
    "ConcurrentStdinRead", "concurrent stdin read was not rejected")
require(resolveLease !== undefined, "stdin lease did not reach its source")
resolveLease(serviceSuccess(Just(encoder.encode("leased\n"))))
const leaseResult = await leasePending
require(leaseResult.kind === "success" &&
  leaseResult.value.tag === "Just" &&
  leaseResult.value.value ===
    "leased", "the active stdin lease did not retain the cursor")

let resolveCancelled:
  | ((value: ReturnType<typeof serviceSuccess<Maybe<Uint8Array>>>) => void)
  | undefined
const cancellationStdin = createByteStdin({
  read(_size: number, _context: EffectContext) {
    return new Promise((resolve) => {
      resolveCancelled = resolve
    })
  },
})
const cancelledExecution = createEffectExecution()
const cancelledRead = run(
  readChunk(size(4)),
  { stdin: cancellationStdin },
  cancelledExecution.context
).then(
  () => false,
  (error) => isEffectCancellation(error)
)
await cancelledExecution.cancel()
require(await cancelledRead, "cancelled stdin read did not exit by cancellation")
require(resolveCancelled !==
  undefined, "cancelled read did not reach its source")
resolveCancelled(serviceSuccess(Just(new Uint8Array([7, 8]))))
await new Promise<void>((resolve) => setTimeout(resolve, 0))
const returnedChunk = await run(readChunk(size(4)), {
  stdin: cancellationStdin,
})
require(returnedChunk.kind === "success" &&
  returnedChunk.value.tag === "Just" &&
  JSON.stringify(toInts(returnedChunk.value.value)) ===
    "[7,8]", "bytes that lost the cancellation race were not returned to the cursor")

const streamSource = chunkSource([encoder.encode("one\ntwo\n")])
const streamStdin = createByteStdin(streamSource.source)
const lineStream = lines(defaultLineLimit())
require(streamSource.reads() === 0, "lines stream was not cold")
const firstRun = await run(runCollect(take(1, lineStream)), {
  stdin: streamStdin,
})
const secondRun = await run(runCollect(lineStream), { stdin: streamStdin })
require(firstRun.kind === "success" &&
  JSON.stringify(firstRun.value) === '["one"]' &&
  secondRun.kind === "success" &&
  JSON.stringify(secondRun.value) ===
    '["two"]', "lines stream replayed or lost the shared cursor")

const overflow = createByteStdin(chunkSource([new Uint8Array([1])]).source, {
  initialOffset: MAX_INT,
})
const overflowResult = await run(readChunk(size(1)), { stdin: overflow })
const overflowAgain = await run(readChunk(size(1)), { stdin: overflow })
require(overflowResult.kind === "failure" &&
  overflowResult.error.tag === "StdinPositionOverflow" &&
  overflowAgain.kind === "failure" &&
  overflowAgain.error.tag ===
    "StdinPositionOverflow", "stdin position overflow was not sticky")

const processInput = new PassThrough()
const processStdin = createProcessStdin(processInput)
require(["data", "error", "end"].every(
  (event) => processInput.listenerCount(event) === 0
), "process stdin adapter was not lazy")
const processPending = run(readLine(), { stdin: processStdin })
const processConcurrent = await run(readLine(), { stdin: processStdin })
processInput.end("process\n")
const processLine = await processPending
const processEof = await run(readLine(), { stdin: processStdin })
const processEofAgain = await run(readLine(), { stdin: processStdin })
require(processConcurrent.kind === "failure" &&
  processConcurrent.error.tag === "ConcurrentStdinRead" &&
  processLine.kind === "success" &&
  processLine.value.tag === "Just" &&
  processLine.value.value === "process" &&
  processEof.kind === "success" &&
  processEof.value === Nothing &&
  processEofAgain.kind === "success" &&
  processEofAgain.value ===
    Nothing, "process stdin adapter violated the shared cursor contract")
processStdin.close()
processStdin.close()

const failureInput = new PassThrough()
const failureStdin = createProcessStdin(failureInput)
const failurePending = run(readLine(), { stdin: failureStdin })
failureInput.destroy(new Error("injected read failure"))
const readFailure = await failurePending
const failureAgain = await run(readLine(), { stdin: failureStdin })
require(readFailure.kind === "failure" &&
  readFailure.error.tag === "StdinReadFailure" &&
  failureAgain.kind === "failure" &&
  failureAgain.error.tag ===
    "StdinReadFailure", "process stdin read failure was not typed and sticky")
failureStdin.close()

const closeInput = new PassThrough()
const closedStdin = createProcessStdin(closeInput)
closedStdin.close()
require(!closeInput.destroyed, "closing Stdin destroyed process input")

console.log("stdin runtime probe passed")
