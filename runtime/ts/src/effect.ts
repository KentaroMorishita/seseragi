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

export const unit: Unit = undefined;

export function succeed<Success>(
  value: Success
): Effect<unknown, never, Success> {
  return () => value;
}

export function fail<Failure>(error: Failure): Effect<unknown, Failure, never> {
  return () => {
    throw error;
  };
}

export async function run<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>,
  environment: Environment
): Promise<EffectResult<Failure, Success>> {
  try {
    return { kind: "success", value: await effect(environment) };
  } catch (error) {
    return { kind: "failure", error: error as Failure };
  }
}
