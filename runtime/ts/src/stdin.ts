import { stdin as processStdin } from "node:process";
import { createInterface } from "node:readline";
import type { Effect } from "./effect";
import { Just, type Maybe, Nothing } from "./sum";

export type Stdin = {
  readonly readLine: () => Promise<Maybe<string>> | Maybe<string>;
};

export type ProcessStdin = Stdin & {
  readonly close: () => void;
};

export type StdinEnvironment = {
  readonly stdin: Stdin;
};

export type StdinError = {
  readonly kind: "stdin-error";
  readonly message: string;
};

/**
 * Creates a root-run-local adapter over process standard input.
 *
 * The readline interface is allocated lazily on the first read. `close` is
 * idempotent, and EOF closes the interface before returning the shared
 * `Nothing` value.
 */
export function createProcessStdin(
  input: NodeJS.ReadableStream = processStdin
): ProcessStdin {
  let interface_: ReturnType<typeof createInterface> | undefined;
  let lines: AsyncIterator<string> | undefined;
  let closed = false;

  const close = () => {
    if (closed) return;
    closed = true;
    interface_?.close();
    interface_ = undefined;
    lines = undefined;
  };

  return {
    async readLine() {
      if (closed) return Nothing;

      if (lines === undefined) {
        interface_ = createInterface({
          input,
          crlfDelay: Infinity,
        });
        lines = interface_[Symbol.asyncIterator]();
      }

      const next = await lines.next();
      if (next.done) {
        close();
        return Nothing;
      }
      return Just(next.value);
    },
    close,
  };
}

/** Reads one line from the Stdin service supplied at the runner boundary. */
export function readLine(): Effect<StdinEnvironment, StdinError, Maybe<string>> {
  return (environment) => environment.stdin.readLine();
}
