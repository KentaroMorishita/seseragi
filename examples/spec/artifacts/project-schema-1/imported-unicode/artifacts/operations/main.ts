import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { normalize as _ssrg_unicode_normalize, NFC as _ssrg_unicode_NFC, type NormalizationForm as NormalizationForm } from "@seseragi/runtime/unicode"
import { sliceScalars as _ssrg_text_sliceScalars, type TextSliceError as TextSliceError } from "@seseragi/runtime/text"
import { slice as _ssrg_grapheme_slice, type GraphemeSliceError as GraphemeSliceError } from "@seseragi/runtime/grapheme"
import { fromCodePoint as _ssrg_char_fromCodePoint, codePoint as _ssrg_char_codePoint } from "@seseragi/runtime/char"
import { textSliceErrorDebug as _ssrg_textSliceErrorDebug } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

export const normalize = (value: string) => canonicalize(value)
export const scalarSlice = (value: string) => takeScalars(value)
export const graphemeSlice = (value: string) => takeGraphemes(value)
export const character = (value: number) => _ssrg_char_fromCodePoint(value)
export const point = (value: string) => _ssrg_char_codePoint(value)
export const inspectError = (value: TextSliceError) => ((_ssrg_textSliceErrorDebug["debug"](value)) as string)
const canonicalize: (argument: string) => string = (__ssrg$text_bytes$partial$0: string) => _ssrg_unicode_normalize(_ssrg_unicode_NFC, __ssrg$text_bytes$partial$0);
const takeScalars: (argument: string) => { readonly tag: "Left"; readonly value: TextSliceError } | { readonly tag: "Right"; readonly value: string } = (__ssrg$text_bytes$partial$0: string) => _ssrg_text_sliceScalars(1, 3, __ssrg$text_bytes$partial$0);
const takeGraphemes: (argument: string) => { readonly tag: "Left"; readonly value: GraphemeSliceError } | { readonly tag: "Right"; readonly value: string } = (__ssrg$text_bytes$partial$0: string) => _ssrg_grapheme_slice(1, 3, __ssrg$text_bytes$partial$0);
