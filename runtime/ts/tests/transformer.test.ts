import { expect, test } from "bun:test"
import { effectMonad, run } from "../src/effect"
import { Just, Left, maybeMonad, Nothing, Right } from "../src/sum"
import * as t from "../src/transformer"

const output = { empty: () => "", append: (a: string) => (b: string) => a + b }

test("MaybeT and EitherT preserve base results and short circuit", () => {
  const m = t.maybeTMonad(maybeMonad)
  const e = t.eitherTMonad(maybeMonad)
  let calls = 0
  const next = () => {
    calls++
    return m.pure(9)
  }
  expect(
    t.maybeTRun(m.flatMap(next)(t.maybeTFromMaybe(maybeMonad, Nothing)))
  ).toEqual(Just(Nothing))
  expect(
    t.eitherTRun(e.flatMap(next)(t.eitherTFromEither(maybeMonad, Left("bad"))))
  ).toEqual(Just(Left("bad")))
  expect(calls).toBe(0)
  expect(t.maybeTRun(m.apply(m.pure((n: number) => n + 1))(m.pure(3)))).toEqual(
    Just(Just(4))
  )
  expect(t.eitherTRun(e.map((n: number) => n + 1)(e.pure(3)))).toEqual(
    Just(Right(4))
  )
})

test("ReaderT local affects only its subcomputation", () => {
  const m = t.readerTMonad(maybeMonad)
  const ask = t.readerTAsk(maybeMonad, undefined)
  const local = t.readerTLocal(maybeMonad, (n: number) => n + 10, ask)
  const work = m.flatMap((a: number) => m.map((b: number) => [a, b])(ask))(
    local
  )
  expect(t.readerTRun(2, work)).toEqual(Just([12, 2]))
  expect(t.readerTRun(5, work)).toEqual(Just([15, 5]))
})

test("StateT threads state and WriterT preserves output order and listen", () => {
  const s = t.stateTMonad(maybeMonad)
  const work = s.flatMap(() => t.stateTGet(maybeMonad, undefined))(
    t.stateTModify(maybeMonad, (n: number) => n + 1)
  )
  expect(t.stateTRun(4, work)).toEqual(Just([5, 5]))
  const w = t.writerTMonad(maybeMonad, output)
  const written = w.flatMap(() => t.writerTTell(maybeMonad, output, "second"))(
    t.writerTTell(maybeMonad, output, "first-")
  )
  expect(t.writerTRun(t.writerTListen(maybeMonad, output, written))).toEqual(
    Just([[undefined, "first-second"], "first-second"])
  )
  expect(t.writerTRun(w.pure(8))).toEqual(Just([8, ""]))
})

test("lift, composition and run retain Effect coldness and repeated execution", async () => {
  const events: string[] = []
  const source = () => {
    events.push("base")
    return 2
  }
  const m = t.maybeTMonad(effectMonad)
  const lifted = t.maybeTLift(effectMonad, source)
  const work = m.flatMap((n: number) => {
    events.push("callback")
    return m.pure(n + 1)
  })(lifted)
  const effect = t.maybeTRun(work)
  expect(events).toEqual([])
  expect(await run(effect, {})).toEqual({ kind: "success", value: Just(3) })
  expect(events).toEqual(["base", "callback"])
  await run(effect, {})
  expect(events).toEqual(["base", "callback", "base", "callback"])
  for (const value of [
    t.eitherTLift(effectMonad, source),
    t.writerTLift(effectMonad, output, source),
  ]) {
    const before = events.length
    expect(typeof value.run).toBe("function")
    expect(events.length).toBe(before)
  }
  const reader = t.readerTLift(effectMonad, source)
  const state = t.stateTLift(effectMonad, source)
  expect(events).toHaveLength(4)
  t.readerTRun({}, reader)
  t.stateTRun(0, state)
  expect(events).toHaveLength(4)
})

test("nested stack ordering controls whether failed computations retain state", () => {
  const state = t.stateTMonad(maybeMonad)
  // MaybeT over StateT keeps the state produced before Nothing.
  const outerMaybe = t.maybeTMonad(state)
  const set = t.maybeTLift(state, t.stateTPut(maybeMonad, 7))
  const fail = t.maybeTFromMaybe(state, Nothing)
  const maybeOverState = outerMaybe.flatMap(() => fail)(set)
  expect(t.stateTRun(0, t.maybeTRun(maybeOverState))).toEqual(
    Just([Nothing, 7])
  )
  // StateT over MaybeT loses the (value,state) payload on Nothing.
  const maybe = t.maybeTMonad(maybeMonad)
  const outerState = t.stateTMonad(maybe)
  const change = t.stateTPut(maybe, 7)
  const stop = t.stateTLift(maybe, t.maybeTFromMaybe(maybeMonad, Nothing))
  const stateOverMaybe = outerState.flatMap(() => stop)(change)
  expect(t.maybeTRun(t.stateTRun(0, stateOverMaybe))).toEqual(Just(Nothing))
})
