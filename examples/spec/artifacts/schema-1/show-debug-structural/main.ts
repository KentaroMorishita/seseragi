import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intShow as _ssrg_show_intShow, stringShow as _ssrg_show_stringShow, tupleShow as _ssrg_show_tupleShow, intDebug as _ssrg_debug_intDebug, stringDebug as _ssrg_debug_stringDebug, recordDebug as _ssrg_debug_recordDebug, rangeDebug as _ssrg_debug_rangeDebug, boolDebug as _ssrg_debug_boolDebug, tupleDebug as _ssrg_debug_tupleDebug, arrayDebug as _ssrg_debug_arrayDebug, recordShow as _ssrg_show_recordShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { exclusive as _ssrg_range_exclusive, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
$ssrg$assertUnicodeVersion("17.0.0")

type Badge =
  | { readonly tag: "Active" }
  | { readonly tag: "Paused" };
const Active: Badge = { tag: "Active" } as const;
const Paused: Badge = { tag: "Paused" } as const;
export const __ssrg$instance$Show$0 = { "show": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "active" : "paused")(value) } as const;
export const __ssrg$instance$Debug$1 = { "debug": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "Badge.Active" : "Badge.Paused")(value) } as const;
export const showTuple = (value: readonly [number, string]) => _ssrg_show_tupleShow<readonly [number, string]>(_ssrg_show_intShow, _ssrg_show_stringShow)["show"](value)
export const debugRecord = (value: { readonly "zeta"?: string; readonly "alpha": number }) => _ssrg_debug_recordDebug<{ readonly "zeta"?: string; readonly "alpha": number }>(["alpha", "zeta"] as const, [false, true] as const, _ssrg_debug_intDebug, _ssrg_debug_stringDebug)["debug"](value)
export const debugNested = (value: { readonly "ranges": ReadonlyArray<readonly [Readonly<{ start: number; end: number; inclusive: boolean }>, boolean]>; readonly "label"?: string }) => _ssrg_debug_recordDebug<{ readonly "ranges": ReadonlyArray<readonly [Readonly<{ start: number; end: number; inclusive: boolean }>, boolean]>; readonly "label"?: string }>(["label", "ranges"] as const, [true, false] as const, _ssrg_debug_stringDebug, _ssrg_debug_arrayDebug<readonly [Readonly<{ start: number; end: number; inclusive: boolean }>, boolean]>(_ssrg_debug_tupleDebug<readonly [Readonly<{ start: number; end: number; inclusive: boolean }>, boolean]>(_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug), _ssrg_debug_boolDebug)))["debug"](value)
export const showGeneric = <A,>(value: readonly [A, { readonly "item": A }]) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_show_tupleShow<readonly [A, { readonly "item": A }]>(__ssrg$evidence$0, _ssrg_show_recordShow<{ readonly "item": A }>(["item"] as const, [false] as const, __ssrg$evidence$0))["show"](value)
export const debugBadgeRecord = (value: { readonly "badge": Badge }) => _ssrg_debug_recordDebug<{ readonly "badge": Badge }>(["badge"] as const, [false] as const, __ssrg$instance$Debug$1)["debug"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(showTuple([42, "ready"] as const)), () => _ssrg_effect_flatMap(_ssrg_console_println(debugRecord(({ "alpha": 1 } as const))), () => _ssrg_effect_flatMap(_ssrg_console_println(debugRecord(({ "zeta": "last", "alpha": 1 } as const))), () => _ssrg_effect_flatMap(_ssrg_console_println(debugNested(({ "ranges": [[_ssrg_range_exclusive(1, 3), true] as const, [_ssrg_range_inclusive(3, 3), false] as const] } as const))), () => _ssrg_console_println(debugBadgeRecord(({ "badge": Active } as const)))))))
