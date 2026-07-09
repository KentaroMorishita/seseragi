import { unit, type Unit } from "./effect";

export type Console = {
  readonly print: (value: string) => void;
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
  print(value) {
    process.stdout.write(value);
  },
  println(value) {
    console.log(value);
  },
};

export function print(value: unknown): Unit {
  liveConsole.print(String(value));
  return unit;
}

export function println(value: unknown): Unit {
  liveConsole.println(String(value));
  return unit;
}
