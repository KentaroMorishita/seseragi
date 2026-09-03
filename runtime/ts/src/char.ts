import { Just, type Maybe, Nothing } from "./sum"

export const codePoint = (value: string): number => value.codePointAt(0)!

export function fromCodePoint(value: number): Maybe<string> {
  return !Number.isInteger(value) ||
    value < 0 ||
    value > 0x10ffff ||
    (value >= 0xd800 && value <= 0xdfff)
    ? Nothing
    : Just(String.fromCodePoint(value))
}

const charToString = (value: string): string =>
  String.fromCodePoint(codePoint(value))
export { charToString as toString }
