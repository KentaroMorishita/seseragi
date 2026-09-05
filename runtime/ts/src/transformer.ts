import type { Unit } from "./effect"
import { type Either, Just, type Maybe, Nothing, Right } from "./sum"
import type { RuntimeDictionary } from "./traversable"

// Source HKT applications are checked by Seseragi; TypeScript cannot apply M<A>.
export type MaybeT<_M, _A> = Readonly<{ run: any }>
export type EitherT<_E, _M, _A> = Readonly<{ run: any }>
export type ReaderT<R, _M, _A> = Readonly<{ run: (environment: R) => any }>
export type StateT<S, _M, _A> = Readonly<{ run: (state: S) => any }>
export type WriterT<_W, _M, _A> = Readonly<{ run: any }>

type Monad = Readonly<{
  pure: (value: any) => any
  map: (f: (value: any) => any) => (value: any) => any
  flatMap: (f: (value: any) => any) => (value: any) => any
}>
type Monoid = Readonly<{
  empty: (unit: Unit) => any
  append: (left: any) => (right: any) => any
}>
const wrap = <R>(run: R): Readonly<{ run: R }> => Object.freeze({ run })

// Deriving map/apply from bind keeps the base Monad's sequencing authoritative.
function operations(pure: (value: any) => any, flatMap: Monad["flatMap"]) {
  const map = (f: (value: any) => any) => flatMap((value) => pure(f(value)))
  return Object.freeze({
    pure,
    flatMap,
    map,
    apply: (functions: any) => (values: any) =>
      flatMap((f: (value: any) => any) => map(f)(values))(functions),
  })
}

export function maybeTMonad<_T0>(evidence: RuntimeDictionary) {
  const base = evidence as Monad
  return operations(
    (value) => wrap(base.pure(Just(value))),
    (f) => (value: MaybeT<unknown, unknown>) =>
      wrap(
        base.flatMap((item: Maybe<unknown>) =>
          item.tag === "Nothing" ? base.pure(Nothing) : f(item.value).run
        )(value.run)
      )
  )
}
export const maybeTFunctor = maybeTMonad
export const maybeTApplicative = maybeTMonad
export function maybeTRun<M, A>(value: MaybeT<M, A>): any {
  return value.run
}
export function maybeTFromMaybe<A>(
  evidence: RuntimeDictionary,
  value: Maybe<A>
): MaybeT<unknown, A> {
  return wrap((evidence as Monad).pure(value))
}
export function maybeTLift(
  evidence: RuntimeDictionary,
  value: any
): MaybeT<unknown, unknown> {
  return wrap((evidence as Monad).map(Just)(value))
}

export function eitherTMonad<_T0, _T1>(evidence: RuntimeDictionary) {
  const base = evidence as Monad
  return operations(
    (value) => wrap(base.pure(Right(value))),
    (f) => (value: EitherT<unknown, unknown, unknown>) =>
      wrap(
        base.flatMap((item: Either<unknown, unknown>) =>
          item.tag === "Left" ? base.pure(item) : f(item.value).run
        )(value.run)
      )
  )
}
export const eitherTFunctor = eitherTMonad
export const eitherTApplicative = eitherTMonad
export function eitherTRun<E, M, A>(value: EitherT<E, M, A>): any {
  return value.run
}
export function eitherTFromEither<E, A>(
  evidence: RuntimeDictionary,
  value: Either<E, A>
): EitherT<E, unknown, A> {
  return wrap((evidence as Monad).pure(value))
}
export function eitherTLift(
  evidence: RuntimeDictionary,
  value: any
): EitherT<unknown, unknown, unknown> {
  return wrap((evidence as Monad).map(Right)(value))
}

export function readerTMonad<_T0, _T1>(evidence: RuntimeDictionary) {
  const base = evidence as Monad
  return operations(
    (value) => wrap((_environment: unknown) => base.pure(value)),
    (f) => (value: ReaderT<unknown, unknown, unknown>) =>
      wrap((environment: unknown) =>
        base.flatMap((item) => f(item).run(environment))(value.run(environment))
      )
  )
}
export const readerTFunctor = readerTMonad
export const readerTApplicative = readerTMonad
export function readerTRun<R, M, A>(
  environment: R,
  value: ReaderT<R, M, A>
): any {
  return value.run(environment)
}
export function readerTAsk(
  evidence: RuntimeDictionary,
  _unit: Unit
): ReaderT<unknown, unknown, unknown> {
  return wrap((environment: unknown) => (evidence as Monad).pure(environment))
}
export function readerTAsks<R, A>(
  evidence: RuntimeDictionary,
  f: (environment: R) => A
): ReaderT<R, unknown, A> {
  return wrap((environment: R) => (evidence as Monad).pure(f(environment)))
}
export function readerTLocal<R, M, A>(
  _evidence: RuntimeDictionary,
  f: (environment: R) => R,
  value: ReaderT<R, M, A>
): ReaderT<R, M, A> {
  return wrap((environment: R) => value.run(f(environment)))
}
export function readerTLift(
  _evidence: RuntimeDictionary,
  value: any
): ReaderT<unknown, unknown, unknown> {
  return wrap((_environment: unknown) => value)
}

export function stateTMonad<_T0, _T1>(evidence: RuntimeDictionary) {
  const base = evidence as Monad
  return operations(
    (value) => wrap((state: unknown) => base.pure([value, state])),
    (f) => (value: StateT<unknown, unknown, unknown>) =>
      wrap((state: unknown) =>
        base.flatMap(([item, next]: readonly [unknown, unknown]) =>
          f(item).run(next)
        )(value.run(state))
      )
  )
}
export const stateTFunctor = stateTMonad
export const stateTApplicative = stateTMonad
export function stateTRun<S, M, A>(initial: S, value: StateT<S, M, A>): any {
  return value.run(initial)
}
export function stateTGet(
  evidence: RuntimeDictionary,
  _unit: Unit
): StateT<unknown, unknown, unknown> {
  return wrap((state: unknown) => (evidence as Monad).pure([state, state]))
}
export function stateTPut<S>(
  evidence: RuntimeDictionary,
  next: S
): StateT<S, unknown, Unit> {
  return wrap((_state: S) => (evidence as Monad).pure([undefined, next]))
}
export function stateTModify<S>(
  evidence: RuntimeDictionary,
  f: (state: S) => S
): StateT<S, unknown, Unit> {
  return wrap((state: S) => (evidence as Monad).pure([undefined, f(state)]))
}
export function stateTLift(
  evidence: RuntimeDictionary,
  value: any
): StateT<unknown, unknown, unknown> {
  return wrap((state: unknown) =>
    (evidence as Monad).map((item) => [item, state])(value)
  )
}

export function writerTMonad<_T0, _T1>(
  evidence: RuntimeDictionary,
  outputEvidence: RuntimeDictionary
) {
  const base = evidence as Monad
  const output = outputEvidence as Monoid
  return operations(
    (value) => wrap(base.pure([value, output.empty(undefined)])),
    (f) => (value: WriterT<unknown, unknown, unknown>) =>
      wrap(
        base.flatMap(([item, first]: readonly [unknown, unknown]) =>
          base.map(([result, second]: readonly [unknown, unknown]) => [
            result,
            output.append(first)(second),
          ])(f(item).run)
        )(value.run)
      )
  )
}
export const writerTFunctor = writerTMonad
export const writerTApplicative = writerTMonad
export function writerTRun<W, M, A>(value: WriterT<W, M, A>): any {
  return value.run
}
export function writerTTell<W>(
  evidence: RuntimeDictionary,
  _outputEvidence: RuntimeDictionary,
  output: W
): WriterT<W, unknown, Unit> {
  return wrap((evidence as Monad).pure([undefined, output]))
}
export function writerTListen<W, M, A>(
  evidence: RuntimeDictionary,
  _outputEvidence: RuntimeDictionary,
  value: WriterT<W, M, A>
): WriterT<W, M, readonly [A, W]> {
  return wrap(
    (evidence as Monad).map(([item, output]: readonly [A, W]) => [
      [item, output],
      output,
    ])(value.run)
  )
}
export function writerTLift(
  evidence: RuntimeDictionary,
  outputEvidence: RuntimeDictionary,
  value: any
): WriterT<unknown, unknown, unknown> {
  return wrap(
    (evidence as Monad).map((item) => [
      item,
      (outputEvidence as Monoid).empty(undefined),
    ])(value)
  )
}
