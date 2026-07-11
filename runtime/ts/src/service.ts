import { fail, type Effect } from "./effect";

export type Awaitable<Value> = Value | Promise<Value>;

export type ServiceFailure<Failure> = {
  readonly kind: "failure";
  readonly error: Failure;
};

export type ServiceSuccess<Success> = {
  readonly kind: "success";
  readonly value: Success;
};

/**
 * The explicit result of one host-service operation.
 *
 * This is a runtime boundary protocol, not Seseragi's domain-level Either.
 */
export type ServiceResult<Failure, Success> =
  | ServiceFailure<Failure>
  | ServiceSuccess<Success>;

export type ServiceOperation<Failure, Success> = Awaitable<
  ServiceResult<Failure, Success>
>;

export function serviceSuccess<Success>(
  value: Success
): ServiceResult<never, Success> {
  return { kind: "success", value };
}

export function serviceFailure<Failure>(
  error: Failure
): ServiceResult<Failure, never> {
  return { kind: "failure", error };
}

/**
 * Lifts an explicit host-service result into Seseragi's Effect channels.
 *
 * Raw throws and rejected promises intentionally remain runtime defects.
 */
export function serviceEffect<Environment, Failure, Success>(
  operation: (
    environment: Environment
  ) => ServiceOperation<Failure, Success>
): Effect<Environment, Failure, Success> {
  return async (environment) => {
    const result: unknown = await operation(environment);
    if (isRecord(result) && result.kind === "success" && "value" in result) {
      return result.value as Success;
    }
    if (isRecord(result) && result.kind === "failure" && "error" in result) {
      return fail(result.error as Failure)(environment);
    }
    throw new TypeError("host service returned an invalid ServiceResult");
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
