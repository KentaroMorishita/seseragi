import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { Cons as _ssrg_list_cons, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow, maybeShow as _ssrg_show_maybeShow, listShow as _ssrg_show_listShow, boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { intEq as _ssrg_int_eq_dictionary, listEq as _ssrg_list_eq_dictionary } from "@seseragi/runtime/equality"
$ssrg$assertUnicodeVersion("17.0.0")

const prepend = <A,>(head: A) => (tail: List<A>) => _ssrg_list_cons<A>(head, tail)
const first = (values: List<number>) => (($ssrg_match: List<number>): number => $ssrg_match.tag === "Cons" ? ((head: number, tail: List<number>): number => head)($ssrg_match.head, $ssrg_match.tail) : 0)(values)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_listShow<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }>(_ssrg_show_maybeShow<number>(_ssrg_show_intShow))["show"](_ssrg_list_cons<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }>(_ssrg_maybe_Nothing, _ssrg_list_from_array([_ssrg_maybe_Just(1)])))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_listShow<number>(_ssrg_show_intShow)["show"](values)) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_listShow<number>(_ssrg_show_intShow)["show"](prepend(0)(values))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"](first(values))) as string)), () => _ssrg_console_println(((_ssrg_show_boolShow["show"](_ssrg_list_eq_dictionary<number>(_ssrg_int_eq_dictionary)["eq"](withFour(values))(_ssrg_list_from_array([4, 1, 2, 3])))) as string))))))
const values: List<number> = _ssrg_list_cons<number>(1, _ssrg_list_cons<number>(2, _ssrg_list_cons<number>(3, _ssrg_list_from_array([] as ReadonlyArray<number>))));
const withFour: (argument: List<number>) => List<number> = (__ssrg$collection$partial$0: List<number>) => _ssrg_list_cons<number>(4, __ssrg$collection$partial$0);
