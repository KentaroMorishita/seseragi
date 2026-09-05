import { normalize as ops_normalize, scalarSlice as ops_scalarSlice, graphemeSlice as ops_graphemeSlice, character as ops_character, point as ops_point, inspectError as ops_inspectError } from "./operations.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { textSliceErrorShow as _ssrg_textSliceErrorShow, stringShow as _ssrg_show_stringShow, eitherShow as _ssrg_show_eitherShow, graphemeSliceErrorShow as _ssrg_graphemeSliceErrorShow, charShow as _ssrg_show_charShow, maybeShow as _ssrg_show_maybeShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { maybeFunctor as _ssrg_maybe_functor } from "@seseragi/runtime/sum"
import { InvalidScalarRange as _ssrg_text_InvalidScalarRange, type TextSliceError as TextSliceError } from "@seseragi/runtime/text"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { type GraphemeSliceError as GraphemeSliceError } from "@seseragi/runtime/grapheme"
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => (() => { const normalize: (argument: string) => string = ops_normalize; return (() => { const slice: (argument: string) => { readonly tag: "Left"; readonly value: TextSliceError } | { readonly tag: "Right"; readonly value: string } = ops_scalarSlice; return _ssrg_effect_flatMap(_ssrg_console_println(normalize(source)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_eitherShow<TextSliceError, string>(_ssrg_textSliceErrorShow, _ssrg_show_stringShow)["show"](slice(source))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_eitherShow<GraphemeSliceError, string>(_ssrg_graphemeSliceErrorShow, _ssrg_show_stringShow)["show"](ops_graphemeSlice(source))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_maybeShow<string>(_ssrg_show_charShow)["show"](ops_character(128077)) + " / " + _ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](((_ssrg_maybe_functor["map"](ops_point)(ops_character(128077))) as { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }))), () => _ssrg_console_println(ops_inspectError(_ssrg_text_InvalidScalarRange(({ "start": _ssrg_int_subtract(0, 1), "end": 2, "length": 5 } as const)))))))); })(); })()
const source: string = "A👍🏽e\u{301}";
