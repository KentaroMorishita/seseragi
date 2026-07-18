import type { Either, Left, Right } from "./sum";

export type Unit = undefined;

export type Effect<Environment, Failure, Success> = ((
  environment: Environment
) => Promise<Success> | Success) & {
  readonly __failure?: Failure;
};

export type EffectFailure<Failure> = {
  readonly kind: "failure";
  readonly error: Failure;
};

export type EffectSuccess<Success> = {
  readonly kind: "success";
  readonly value: Success;
};

export type EffectResult<Failure, Success> =
  | EffectFailure<Failure>
  | EffectSuccess<Success>;

class TypedFailureSignal<Failure> {
  readonly error: Failure;

  constructor(error: Failure) {
    this.error = error;
  }
}

export const unit: Unit = undefined;

export function succeed<Success>(
  value: Success
): Effect<unknown, never, Success> {
  return () => value;
}

export function flatMap<Environment, Failure, Success, NextEnvironment, NextFailure, NextSuccess>(
  effect: Effect<Environment, Failure, Success>,
  next: (value: Success) => Effect<NextEnvironment, NextFailure, NextSuccess>
): Effect<Environment & NextEnvironment, Failure | NextFailure, NextSuccess> {
  return async (environment) => {
    const value = await effect(environment);
    return next(value)(environment);
  };
}

export const effectFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    <Environment, Failure>(
      effect: Effect<Environment, Failure, Value>
    ): Effect<Environment, Failure, Result> =>
    async (environment) =>
      f(await effect(environment)),
});

export const effectApplicative = Object.freeze({
  ...effectFunctor,
  pure: succeed,
  apply:
    <Environment, Failure, Value, Result>(
      functions: Effect<Environment, Failure, (value: Value) => Result>
    ) =>
    (
      values: Effect<Environment, Failure, Value>
    ): Effect<Environment, Failure, Result> =>
      flatMap(functions, (f) => effectFunctor.map(f)(values)),
});

export const effectMonad = Object.freeze({
  ...effectApplicative,
  flatMap:
    <Value, NextEnvironment, NextFailure, Result>(
      f: (value: Value) => Effect<NextEnvironment, NextFailure, Result>
    ) =>
    <Environment, Failure>(effect: Effect<Environment, Failure, Value>) =>
      flatMap(effect, f),
});

export function fail<Failure>(error: Failure): Effect<unknown, Failure, never> {
  return () => {
    throw new TypedFailureSignal(error);
  };
}

type EitherFailure<Value> = Value extends Left<infer Failure>
  ? Failure
  : never;

type EitherSuccess<Value> = Value extends Right<infer Success>
  ? Success
  : never;

export function fromEither<Value extends Either<unknown, unknown>>(
  value: Value
): Effect<unknown, EitherFailure<Value>, EitherSuccess<Value>>;
export function fromEither(
  value: Either<unknown, unknown>
): Effect<unknown, unknown, unknown> {
  if (value.tag === "Right") {
    const success = value.value;
    return succeed(success);
  }
  const failure = value.value;
  return fail(failure);
}

export function mapError<Environment, Failure, NextFailure, Success>(
  mapper: (error: Failure) => NextFailure,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment, NextFailure, Success> {
  return async (environment) => {
    try {
      return await effect(environment);
    } catch (error) {
      if (error instanceof TypedFailureSignal) {
        throw new TypedFailureSignal(mapper(error.error as Failure));
      }
      throw error;
    }
  };
}

export async function run<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>,
  environment: Environment
): Promise<EffectResult<Failure, Success>> {
  try {
    return { kind: "success", value: await effect(environment) };
  } catch (error) {
    if (error instanceof TypedFailureSignal) {
      return { kind: "failure", error: error.error as Failure };
    }
    throw error;
  }
}
