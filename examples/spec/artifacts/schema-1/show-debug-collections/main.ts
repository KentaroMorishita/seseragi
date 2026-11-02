import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow, listShow as _ssrg_show_listShow, eitherShow as _ssrg_show_eitherShow, maybeShow as _ssrg_show_maybeShow, arrayShow as _ssrg_show_arrayShow, stringDebug as _ssrg_debug_stringDebug, boolDebug as _ssrg_debug_boolDebug, listDebug as _ssrg_debug_listDebug, eitherDebug as _ssrg_debug_eitherDebug, maybeDebug as _ssrg_debug_maybeDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just, Right as _ssrg_either_Right, Nothing as _ssrg_maybe_Nothing, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"
import { fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
const $ssrg$source403 = { file: "main.ssrg", line: 24, column: 3, uri: "seseragi://artifact/show-debug-collections", start: 403, end: 413 };
const $ssrg$source500 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/show-debug-collections", start: 500, end: 511 };
const $ssrg$source577 = { file: "main.ssrg", line: 33, column: 3, uri: "seseragi://artifact/show-debug-collections", start: 577, end: 588 };
const $ssrg$source659 = { file: "main.ssrg", line: 37, column: 3, uri: "seseragi://artifact/show-debug-collections", start: 659, end: 670 };
const $ssrg$source704 = { file: "main.ssrg", line: 41, column: 5, uri: "seseragi://artifact/show-debug-collections", start: 704, end: 776 };
const $ssrg$source713 = { file: "main.ssrg", line: 41, column: 14, uri: "seseragi://artifact/show-debug-collections", start: 713, end: 775 };
const $ssrg$source725 = { file: "main.ssrg", line: 41, column: 26, uri: "seseragi://artifact/show-debug-collections", start: 725, end: 745 };
const $ssrg$source731 = { file: "main.ssrg", line: 41, column: 32, uri: "seseragi://artifact/show-debug-collections", start: 731, end: 744 };
const $ssrg$source756 = { file: "main.ssrg", line: 41, column: 57, uri: "seseragi://artifact/show-debug-collections", start: 756, end: 774 };
const $ssrg$source762 = { file: "main.ssrg", line: 41, column: 63, uri: "seseragi://artifact/show-debug-collections", start: 762, end: 773 };
const $ssrg$source781 = { file: "main.ssrg", line: 42, column: 5, uri: "seseragi://artifact/show-debug-collections", start: 781, end: 861 };
const $ssrg$source790 = { file: "main.ssrg", line: 42, column: 14, uri: "seseragi://artifact/show-debug-collections", start: 790, end: 860 };
const $ssrg$source803 = { file: "main.ssrg", line: 42, column: 27, uri: "seseragi://artifact/show-debug-collections", start: 803, end: 830 };
const $ssrg$source809 = { file: "main.ssrg", line: 42, column: 33, uri: "seseragi://artifact/show-debug-collections", start: 809, end: 829 };
const $ssrg$source841 = { file: "main.ssrg", line: 42, column: 65, uri: "seseragi://artifact/show-debug-collections", start: 841, end: 859 };
const $ssrg$source847 = { file: "main.ssrg", line: 42, column: 71, uri: "seseragi://artifact/show-debug-collections", start: 847, end: 858 };
const $ssrg$source866 = { file: "main.ssrg", line: 43, column: 5, uri: "seseragi://artifact/show-debug-collections", start: 866, end: 897 };
const $ssrg$source875 = { file: "main.ssrg", line: 43, column: 14, uri: "seseragi://artifact/show-debug-collections", start: 875, end: 896 };
const $ssrg$source902 = { file: "main.ssrg", line: 44, column: 5, uri: "seseragi://artifact/show-debug-collections", start: 902, end: 931 };
const $ssrg$source911 = { file: "main.ssrg", line: 44, column: 14, uri: "seseragi://artifact/show-debug-collections", start: 911, end: 930 };
const $ssrg$source918 = { file: "main.ssrg", line: 44, column: 21, uri: "seseragi://artifact/show-debug-collections", start: 918, end: 929 };
$ssrg$assertUnicodeVersion("17.0.0")

type Badge =
  | { readonly tag: "Active" }
  | { readonly tag: "Paused" };
const Active: Badge = { tag: "Active" } as const;
const Paused: Badge = { tag: "Paused" } as const;
export const __ssrg$instance$Show$0 = { "show": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "active" : "paused")(value) } as const;
export const __ssrg$instance$Debug$1 = { "debug": (value: Badge) => (($ssrg_match: Badge): string => $ssrg_match.tag === "Active" ? "Badge.Active" : "Badge.Paused")(value) } as const;
export const showNested = (value: ReadonlyArray<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: List<number> } }>) => $ssrg$atSource($ssrg$source403, () => ((_ssrg_show_arrayShow<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: List<number> } }>(_ssrg_show_maybeShow<{ readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: List<number> }>(_ssrg_show_eitherShow<string, List<number>>(_ssrg_show_stringShow, _ssrg_show_listShow<number>(_ssrg_show_intShow))))["show"](value)) as string), false)
export const debugNested = (value: ReadonlyArray<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: List<boolean> } }>) => $ssrg$atSource($ssrg$source500, () => ((_ssrg_debug_arrayDebug<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: List<boolean> } }>(_ssrg_debug_maybeDebug<{ readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: List<boolean> }>(_ssrg_debug_eitherDebug<string, List<boolean>>(_ssrg_debug_stringDebug, _ssrg_debug_listDebug<boolean>(_ssrg_debug_boolDebug))))["debug"](value)) as string), false)
export const showMany = <A,>(values: ReadonlyArray<A>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source577, () => ((_ssrg_show_arrayShow<A>(__ssrg$evidence$0)["show"](values)) as string), false)
export const debugOptional = <A,>(value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source659, () => ((_ssrg_debug_maybeDebug<A>(__ssrg$evidence$0)["debug"](value)) as string), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source704, () => _ssrg_console_println($ssrg$atSource($ssrg$source713, () => showNested([$ssrg$atSource($ssrg$source725, () => _ssrg_maybe_Just($ssrg$atSource($ssrg$source731, () => _ssrg_either_Right(_ssrg_list_from_array([1, 2])), false)), false), _ssrg_maybe_Nothing, $ssrg$atSource($ssrg$source756, () => _ssrg_maybe_Just($ssrg$atSource($ssrg$source762, () => _ssrg_either_Left("oops"), false)), false)]), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source781, () => _ssrg_console_println($ssrg$atSource($ssrg$source790, () => debugNested([$ssrg$atSource($ssrg$source803, () => _ssrg_maybe_Just($ssrg$atSource($ssrg$source809, () => _ssrg_either_Right(_ssrg_list_from_array([true, false])), false)), false), _ssrg_maybe_Nothing, $ssrg$atSource($ssrg$source841, () => _ssrg_maybe_Just($ssrg$atSource($ssrg$source847, () => _ssrg_either_Left("oops"), false)), false)]), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source866, () => _ssrg_console_println($ssrg$atSource($ssrg$source875, () => ((_ssrg_show_arrayShow<Badge>(__ssrg$instance$Show$0)["show"]([Active, Paused])) as string), false)), true), () => $ssrg$atSource($ssrg$source902, () => _ssrg_console_println($ssrg$atSource($ssrg$source911, () => ((_ssrg_debug_maybeDebug<Badge>(__ssrg$instance$Debug$1)["debug"]($ssrg$atSource($ssrg$source918, () => _ssrg_maybe_Just(Active), false))) as string), false)), true))))
