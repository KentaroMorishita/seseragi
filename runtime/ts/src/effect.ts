import type { Either } from "./sum";

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

export function fail<Failure>(error: Failure): Effect<unknown, Failure, never> {
  return () => {
    throw new TypedFailureSignal(error);
  };
}

export function fromEither<Failure, Success>(
  value: Either<Failure, Success>
): Effect<unknown, Failure, Success> {
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
