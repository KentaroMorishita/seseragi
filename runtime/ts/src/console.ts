import { unit, type Effect, type Unit } from "./effect";

export type Console = {
  readonly println: (value: string) => void;
};

export type ConsoleEnvironment = {
  readonly console: Console;
};

export type ConsoleError = {
  readonly kind: "console-error";
  readonly message: string;
};

export const liveConsole: Console = {
  println(value) {
    console.log(value);
  },
};

export function println(
  value: unknown
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return (environment) => {
    environment.console.println(String(value));
    return unit;
  };
}
