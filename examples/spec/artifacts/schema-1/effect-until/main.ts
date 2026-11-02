import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { arrayIterable as _ssrg_array_iterable } from "@seseragi/runtime/array"
import { reduceUntil as _ssrg_collection_reduceUntil, Done as _ssrg_collection_Done, Next as _ssrg_collection_Next, type ReduceStep as ReduceStep } from "@seseragi/runtime/collection"
import { intOrd as _ssrg_int_ord_dictionary, intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { forEachUntil as _ssrg_effect_forEachUntil, succeed as _ssrg_effect_succeed, Break as _ssrg_effect_Break, Continue as _ssrg_effect_Continue, type Effect as Effect, type LoopControl as LoopControl } from "@seseragi/runtime/effect"
const $ssrg$source111 = { file: "main.ssrg", line: 4, column: 27, uri: "seseragi://artifact/effect-until", start: 111, end: 288 };
const $ssrg$source125 = { file: "main.ssrg", line: 7, column: 3, uri: "seseragi://artifact/effect-until", start: 125, end: 127 };
const $ssrg$source214 = { file: "main.ssrg", line: 12, column: 7, uri: "seseragi://artifact/effect-until", start: 214, end: 236 };
const $ssrg$source252 = { file: "main.ssrg", line: 14, column: 7, uri: "seseragi://artifact/effect-until", start: 252, end: 284 };
const $ssrg$source270 = { file: "main.ssrg", line: 14, column: 25, uri: "seseragi://artifact/effect-until", start: 270, end: 283 };
const $ssrg$source362 = { file: "main.ssrg", line: 18, column: 3, uri: "seseragi://artifact/effect-until", start: 362, end: 446 };
const $ssrg$source385 = { file: "main.ssrg", line: 19, column: 6, uri: "seseragi://artifact/effect-until", start: 385, end: 395 };
const $ssrg$source462 = { file: "main.ssrg", line: 24, column: 15, uri: "seseragi://artifact/effect-until", start: 462, end: 507 };
$ssrg$assertUnicodeVersion("17.0.0")

const stopAtThree = (value: number) => $ssrg$atSource($ssrg$source362, () => _ssrg_effect_succeed($ssrg$atSource($ssrg$source385, () => _ssrg_int_eq_dictionary["eq"](value)(3), false) ? _ssrg_effect_Break : _ssrg_effect_Continue), true)
const totalBeforeNegative: number = $ssrg$atSource($ssrg$source111, () => _ssrg_collection_reduceUntil(_ssrg_array_iterable, 0, (total: number) => (value: number) => (_ssrg_int_ord_dictionary["compare"](value)(0))["tag"] === "Less" ? $ssrg$atSource($ssrg$source214, () => _ssrg_collection_Done(total), false) : $ssrg$atSource($ssrg$source252, () => _ssrg_collection_Next(_ssrg_int_add(total, value, $ssrg$source270)), false), [1, 2, _ssrg_int_subtract(0, 1, $ssrg$source125), 100]), false);
const stopped: Effect<{  }, never, undefined> = $ssrg$atSource($ssrg$source462, () => _ssrg_effect_forEachUntil(stopAtThree, [1, 2, 3, 4], _ssrg_array_iterable), true);
