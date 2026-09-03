import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { boundedShow as _ssrg_show_boundedShow, boundedDebug as _ssrg_debug_boundedDebug, stringShow as _ssrg_show_stringShow, stringDebug as _ssrg_debug_stringDebug, maybeShow as _ssrg_show_maybeShow, maybeDebug as _ssrg_debug_maybeDebug, type Show as _ssrg_show_Show, type Debug as _ssrg_debug_Debug } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

export type Code =
  | { readonly tag: "Code"; readonly value: string };
export const Code = (value: string): Code => ({ tag: "Code", value } as const);
export type Remote<A> =
  | { readonly tag: "Missing" }
  | { readonly tag: "Remote"; readonly value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A } };
export const Missing = { tag: "Missing" } as const;
export const Remote = <A>(value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }): Remote<A> => ({ tag: "Remote", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<Code> = _ssrg_show_boundedShow((value: Code): string => { switch (value.tag) { case "Code": return "Code" + " " + _ssrg_show_stringShow.show(value.value); } });
export const __ssrg$instance$Debug$1: _ssrg_debug_Debug<Code> = _ssrg_debug_boundedDebug((value: Code): string => { switch (value.tag) { case "Code": return "Code" + " " + _ssrg_debug_stringDebug.debug(value.value); } });
export const __ssrg$instance$Show$2 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_show_Show<Remote<A>> => (_ssrg_show_boundedShow((value: Remote<A>): string => { switch (value.tag) { case "Missing": return "Missing"; case "Remote": return "Remote" + " " + (_ssrg_show_maybeShow<A>(__ssrg$evidence$0)).show(value.value); } }));
export const __ssrg$instance$Debug$3 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_debug_Debug<Remote<A>> => (_ssrg_debug_boundedDebug((value: Remote<A>): string => { switch (value.tag) { case "Missing": return "Missing"; case "Remote": return "Remote" + " " + (_ssrg_debug_maybeDebug<A>(__ssrg$evidence$0)).debug(value.value); } }));
