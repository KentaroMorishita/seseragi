import { type Box as Foreign, type $ssrg$kind$Box as $ssrg$kind$Foreign } from "./domain.js"
import { boxed } from "./facade.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow, boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

type Box<A> =
  | { readonly tag: "Local"; readonly value: A };
const Local = <A>(value: A): Box<A> => ({ tag: "Local", value } as const);
const read = <A,>(value: Foreign<A>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (($ssrg_match: Foreign<A>): string => $ssrg_match.tag === "Box" ? ((inner: A): string => ((__ssrg$evidence$0["show"](inner)) as string))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
const hold = <A,>(value: Foreign<A>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (prefix: string) => _ssrg_string_add_dictionary["add"](prefix)(((read<A>(value)(__ssrg$evidence$0)) as string))
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(((read<number>(boxed<number>(42))(_ssrg_show_intShow)) as string)), () => (() => { const saved: (argument: string) => string = ((hold<number>(boxed<number>(7))(_ssrg_show_intShow)) as (argument: string) => string); return _ssrg_effect_flatMap(_ssrg_console_println(saved("saved=")), () => ((forward) => { return _ssrg_effect_flatMap(_ssrg_console_println(((forward<number>(42)(_ssrg_show_intShow)) as string)), () => _ssrg_console_println(((forward<boolean>(true)(_ssrg_show_boolShow)) as string))); })(<Let426T0,>(value: Let426T0) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ((read<Let426T0>(boxed<Let426T0>(value))(__ssrg$evidence$0)) as string))); })())
