import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { boundedShow as _ssrg_show_boundedShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { succeed as _ssrg_effect_succeed, flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"
$ssrg$assertUnicodeVersion("17.0.0")

export type SourceError =
  | { readonly tag: "Source" };
export const Source: SourceError = { tag: "Source" } as const;
export type AppError =
  | { readonly tag: "Wrapped"; readonly value: SourceError };
export const Wrapped = (value: SourceError): AppError => ({ tag: "Wrapped", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<SourceError> = _ssrg_show_boundedShow((value: SourceError): string => { switch (value.tag) { case "Source": return "Source"; } });
export const __ssrg$instance$Show$1: _ssrg_show_Show<AppError> = _ssrg_show_boundedShow((value: AppError): string => { switch (value.tag) { case "Wrapped": return "Wrapped" + " " + __ssrg$instance$Show$0.show(value.value); } });
export const infallible = (_unit: undefined) => _ssrg_effect_succeed(undefined)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_effect_mapError(Wrapped, _ssrg_effect_fail(Source)), () => _ssrg_effect_succeed(undefined))
