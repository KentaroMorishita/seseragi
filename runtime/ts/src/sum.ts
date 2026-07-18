export type Nothing = {
  readonly tag: "Nothing";
};

export type Just<Value> = {
  readonly tag: "Just";
  readonly value: Value;
};

export type Maybe<Value> = Nothing | Just<Value>;

export type Left<Error> = {
  readonly tag: "Left";
  readonly value: Error;
};

export type Right<Value> = {
  readonly tag: "Right";
  readonly value: Value;
};

export type Either<Error, Value> = Left<Error> | Right<Value>;

export const Nothing: Nothing = Object.freeze({ tag: "Nothing" });

export function Just<Value>(value: Value): Just<Value> {
  return { tag: "Just", value };
}

export function Left<Error>(error: Error): Left<Error> {
  return { tag: "Left", value: error };
}

export function Right<Value>(value: Value): Right<Value> {
  return { tag: "Right", value };
}

export const maybeFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    (value: Maybe<Value>): Maybe<Result> =>
      value.tag === "Nothing" ? Nothing : Just(f(value.value)),
});

export const maybeApplicative = Object.freeze({
  ...maybeFunctor,
  pure: <Value>(value: Value): Maybe<Value> => Just(value),
  apply:
    <Value, Result>(wrappedFunction: Maybe<(value: Value) => Result>) =>
    (wrappedValue: Maybe<Value>): Maybe<Result> => {
      if (wrappedFunction.tag === "Nothing") return Nothing;
      if (wrappedValue.tag === "Nothing") return Nothing;
      return Just(wrappedFunction.value(wrappedValue.value));
    },
});

export const maybeMonad = Object.freeze({
  ...maybeApplicative,
  flatMap:
    <Value, Result>(f: (value: Value) => Maybe<Result>) =>
    (value: Maybe<Value>): Maybe<Result> =>
      value.tag === "Nothing" ? Nothing : f(value.value),
});

export const eitherFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    <Error>(value: Either<Error, Value>): Either<Error, Result> =>
      value.tag === "Left" ? value : Right(f(value.value)),
});

export const eitherApplicative = Object.freeze({
  ...eitherFunctor,
  pure: <Error, Value>(value: Value): Either<Error, Value> => Right(value),
  apply:
    <Error, Value, Result>(
      wrappedFunction: Either<Error, (value: Value) => Result>,
    ) =>
    (wrappedValue: Either<Error, Value>): Either<Error, Result> => {
      if (wrappedFunction.tag === "Left") return wrappedFunction;
      if (wrappedValue.tag === "Left") return wrappedValue;
      return Right(wrappedFunction.value(wrappedValue.value));
    },
});

export const eitherMonad = Object.freeze({
  ...eitherApplicative,
  flatMap:
    <Error, Value, Result>(f: (value: Value) => Either<Error, Result>) =>
    (value: Either<Error, Value>): Either<Error, Result> =>
      value.tag === "Left" ? value : f(value.value),
});
