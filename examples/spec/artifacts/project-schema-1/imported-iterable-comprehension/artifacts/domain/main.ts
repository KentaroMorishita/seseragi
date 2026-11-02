import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { unfold as _ssrg_iterator_unfold, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_range_reduce, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source132 = { file: "project/imported-iterable-comprehension/src/domain.ssrg", line: 6, column: 5, uri: "seseragi://fixture/imported-iterable-comprehension::domain", start: 132, end: 159 };
const $ssrg$source147 = { file: "project/imported-iterable-comprehension/src/domain.ssrg", line: 6, column: 20, uri: "seseragi://fixture/imported-iterable-comprehension::domain", start: 147, end: 158 };
const $ssrg$source310 = { file: "project/imported-iterable-comprehension/src/domain.ssrg", line: 13, column: 26, uri: "seseragi://fixture/imported-iterable-comprehension::domain", start: 310, end: 334 };
const $ssrg$source318 = { file: "project/imported-iterable-comprehension/src/domain.ssrg", line: 13, column: 34, uri: "seseragi://fixture/imported-iterable-comprehension::domain", start: 318, end: 331 };
const $ssrg$source508 = { file: "project/imported-iterable-comprehension/src/domain.ssrg", line: 21, column: 26, uri: "seseragi://fixture/imported-iterable-comprehension::domain", start: 508, end: 539 };
const $ssrg$source529 = { file: "project/imported-iterable-comprehension/src/domain.ssrg", line: 21, column: 47, uri: "seseragi://fixture/imported-iterable-comprehension::domain", start: 529, end: 538 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Countdown =
  | { readonly tag: "Countdown"; readonly value: number };
export const Countdown = (value: number): Countdown => ({ tag: "Countdown", value } as const);
export const __ssrg$instance$Iterable$0 = { "iterate": (values: Countdown) => (($ssrg_match: Countdown): Iterator<number> => $ssrg_match.tag === "Countdown" ? ((limit: number): Iterator<number> => $ssrg$atSource($ssrg$source310, () => _ssrg_iterator_unfold($ssrg$atSource($ssrg$source318, () => advance(limit), false), 1), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(values) } as const;
export const __ssrg$instance$Reducible$1 = (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "reduce": <B,>(initial: B) => (step: (argument: B) => (argument: number) => B) => (values: Countdown) => (($ssrg_match: Countdown): B => $ssrg_match.tag === "Countdown" ? ((limit: number): B => $ssrg$atSource($ssrg$source508, () => _ssrg_range_reduce(initial, step, $ssrg$atSource($ssrg$source529, () => _ssrg_range_inclusive(1, limit), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(values) }) as const;
const advance = (limit: number) => (current: number) => (_ssrg_int_ord_dictionary["compare"](current)(limit))["tag"] !== "Greater" ? $ssrg$atSource($ssrg$source132, () => _ssrg_maybe_Just([current, _ssrg_int_add(current, 1, $ssrg$source147)] as const), false) : _ssrg_maybe_Nothing
