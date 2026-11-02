import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { normalize as _ssrg_unicode_normalize, NFC as _ssrg_unicode_NFC, type NormalizationForm as NormalizationForm } from "@seseragi/runtime/unicode"
import { sliceScalars as _ssrg_text_sliceScalars, type TextSliceError as TextSliceError } from "@seseragi/runtime/text"
import { slice as _ssrg_grapheme_slice, type GraphemeSliceError as GraphemeSliceError } from "@seseragi/runtime/grapheme"
import { fromCodePoint as _ssrg_char_fromCodePoint, codePoint as _ssrg_char_codePoint } from "@seseragi/runtime/char"
import { textSliceErrorDebug as _ssrg_textSliceErrorDebug } from "@seseragi/runtime/show"
const $ssrg$source195 = { file: "project/imported-unicode/src/operations.ssrg", line: 6, column: 38, uri: "seseragi://fixture/imported-unicode::operations", start: 195, end: 224 };
const $ssrg$source290 = { file: "project/imported-unicode/src/operations.ssrg", line: 7, column: 66, uri: "seseragi://fixture/imported-unicode::operations", start: 290, end: 311 };
const $ssrg$source390 = { file: "project/imported-unicode/src/operations.ssrg", line: 9, column: 3, uri: "seseragi://fixture/imported-unicode::operations", start: 390, end: 409 };
const $ssrg$source454 = { file: "project/imported-unicode/src/operations.ssrg", line: 11, column: 44, uri: "seseragi://fixture/imported-unicode::operations", start: 454, end: 472 };
const $ssrg$source549 = { file: "project/imported-unicode/src/operations.ssrg", line: 13, column: 3, uri: "seseragi://fixture/imported-unicode::operations", start: 549, end: 566 };
const $ssrg$source654 = { file: "project/imported-unicode/src/operations.ssrg", line: 15, column: 3, uri: "seseragi://fixture/imported-unicode::operations", start: 654, end: 673 };
const $ssrg$source719 = { file: "project/imported-unicode/src/operations.ssrg", line: 16, column: 46, uri: "seseragi://fixture/imported-unicode::operations", start: 719, end: 743 };
const $ssrg$source778 = { file: "project/imported-unicode/src/operations.ssrg", line: 17, column: 35, uri: "seseragi://fixture/imported-unicode::operations", start: 778, end: 798 };
const $ssrg$source858 = { file: "project/imported-unicode/src/operations.ssrg", line: 18, column: 60, uri: "seseragi://fixture/imported-unicode::operations", start: 858, end: 869 };
$ssrg$assertUnicodeVersion("17.0.0")

export const normalize = (value: string) => $ssrg$atSource($ssrg$source454, () => canonicalize(value), false)
export const scalarSlice = (value: string) => $ssrg$atSource($ssrg$source549, () => takeScalars(value), false)
export const graphemeSlice = (value: string) => $ssrg$atSource($ssrg$source654, () => takeGraphemes(value), false)
export const character = (value: number) => $ssrg$atSource($ssrg$source719, () => _ssrg_char_fromCodePoint(value), false)
export const point = (value: string) => $ssrg$atSource($ssrg$source778, () => _ssrg_char_codePoint(value), false)
export const inspectError = (value: TextSliceError) => $ssrg$atSource($ssrg$source858, () => ((_ssrg_textSliceErrorDebug["debug"](value)) as string), false)
const canonicalize: (argument: string) => string = $ssrg$atSource($ssrg$source195, () => (() => { const __ssrg$text_bytes$saved0: NormalizationForm = _ssrg_unicode_NFC; return (__ssrg$text_bytes$partial$0: string) => _ssrg_unicode_normalize(__ssrg$text_bytes$saved0, __ssrg$text_bytes$partial$0); })(), false);
const takeScalars: (argument: string) => { readonly tag: "Left"; readonly value: TextSliceError } | { readonly tag: "Right"; readonly value: string } = $ssrg$atSource($ssrg$source290, () => (() => { const __ssrg$text_bytes$saved0: number = 1; return (() => { const __ssrg$text_bytes$saved1: number = 3; return (__ssrg$text_bytes$partial$0: string) => _ssrg_text_sliceScalars(__ssrg$text_bytes$saved0, __ssrg$text_bytes$saved1, __ssrg$text_bytes$partial$0); })(); })(), false);
const takeGraphemes: (argument: string) => { readonly tag: "Left"; readonly value: GraphemeSliceError } | { readonly tag: "Right"; readonly value: string } = $ssrg$atSource($ssrg$source390, () => (() => { const __ssrg$text_bytes$saved0: number = 1; return (() => { const __ssrg$text_bytes$saved1: number = 3; return (__ssrg$text_bytes$partial$0: string) => _ssrg_grapheme_slice(__ssrg$text_bytes$saved0, __ssrg$text_bytes$saved1, __ssrg$text_bytes$partial$0); })(); })(), false);
