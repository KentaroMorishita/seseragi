import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { arrayIterable as _ssrg_array_iterable } from "@seseragi/runtime/array"
import { reduceUntil as _ssrg_collection_reduceUntil, Done as _ssrg_collection_Done, Next as _ssrg_collection_Next, type ReduceStep as ReduceStep } from "@seseragi/runtime/collection"
import { intOrd as _ssrg_int_ord_dictionary, intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { forEachUntil as _ssrg_effect_forEachUntil, succeed as _ssrg_effect_succeed, Break as _ssrg_effect_Break, Continue as _ssrg_effect_Continue, type Effect as Effect, type LoopControl as LoopControl } from "@seseragi/runtime/effect"
$ssrg$assertUnicodeVersion("17.0.0")

const stopAtThree = (value: number) => _ssrg_effect_succeed(_ssrg_int_eq_dictionary["eq"](value)(3) ? _ssrg_effect_Break : _ssrg_effect_Continue)
const totalBeforeNegative: number = _ssrg_collection_reduceUntil(_ssrg_array_iterable, 0, (total: number) => (value: number) => (_ssrg_int_ord_dictionary["compare"](value)(0))["tag"] === "Less" ? _ssrg_collection_Done(total) : _ssrg_collection_Next(_ssrg_int_add(total, value)), [1, 2, _ssrg_int_subtract(0, 1), 100]);
const stopped: Effect<{  }, never, undefined> = _ssrg_effect_forEachUntil(stopAtThree, [1, 2, 3, 4], _ssrg_array_iterable);
