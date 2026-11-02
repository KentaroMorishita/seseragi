import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { succeed as _ssrg_effect_succeed, flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"
const $ssrg$source160 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/effect-explicit-failure-contract", start: 160, end: 170 };
const $ssrg$source227 = { file: "main.ssrg", line: 14, column: 5, uri: "seseragi://artifact/effect-explicit-failure-contract", start: 227, end: 257 };
const $ssrg$source245 = { file: "main.ssrg", line: 14, column: 23, uri: "seseragi://artifact/effect-explicit-failure-contract", start: 245, end: 256 };
const $ssrg$source262 = { file: "main.ssrg", line: 15, column: 5, uri: "seseragi://artifact/effect-explicit-failure-contract", start: 262, end: 272 };
$ssrg$assertUnicodeVersion("17.0.0")

export type SourceError =
  | { readonly tag: "Source" };
export const Source: SourceError = { tag: "Source" } as const;
export type AppError =
  | { readonly tag: "Wrapped"; readonly value: SourceError };
export const Wrapped = (value: SourceError): AppError => ({ tag: "Wrapped", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<SourceError> = _ssrg_show_boundedShow((value: SourceError): string => { switch (value.tag) { case "Source": return "Source"; } });
export const __ssrg$instance$Show$1: _ssrg_show_Show<AppError> = _ssrg_show_boundedShow((value: AppError): string => { switch (value.tag) { case "Wrapped": return "Wrapped" + " " + __ssrg$instance$Show$0.show(value.value); } });
export const infallible = (_unit: undefined) => $ssrg$atSource($ssrg$source160, () => _ssrg_effect_succeed(undefined), true)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source227, () => _ssrg_effect_mapError(Wrapped, $ssrg$atSource($ssrg$source245, () => _ssrg_effect_fail(Source), true)), true), () => $ssrg$atSource($ssrg$source262, () => _ssrg_effect_succeed(undefined), true))
