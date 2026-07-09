import { stdin } from "node:process";
import { createInterface } from "node:readline";

let lines: AsyncIterator<string> | undefined;

function lineIterator(): AsyncIterator<string> {
  if (lines === undefined) {
    lines = createInterface({
      input: stdin,
      crlfDelay: Infinity,
    })[Symbol.asyncIterator]();
  }

  return lines;
}

/**
 * Reads one line from the process-wide standard-input cursor.
 *
 * This is intentionally the narrow first runtime slice: calls must be made
 * sequentially, and EOF is represented by `undefined`.
 */
export async function readLine(): Promise<string | undefined> {
  const next = await lineIterator().next();
  return next.done ? undefined : next.value;
}
