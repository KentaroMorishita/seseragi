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
