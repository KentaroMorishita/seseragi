import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { boundedDebug as _ssrg_debug_boundedDebug, intDebug as _ssrg_debug_intDebug, maybeDebug as _ssrg_debug_maybeDebug, stringDebug as _ssrg_debug_stringDebug, eitherDebug as _ssrg_debug_eitherDebug, arrayDebug as _ssrg_debug_arrayDebug, listDebug as _ssrg_debug_listDebug, type Debug as _ssrg_debug_Debug } from "@seseragi/runtime/show"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing, maybeMonad as _ssrg_maybe_monad, maybeApplicative as _ssrg_maybe_applicative, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export type Packet<A> =
  | { readonly tag: "Packet"; readonly value: A };
export const Packet = <A>(value: A): Packet<A> => ({ tag: "Packet", value } as const);
declare const __ssrg$brand$Box: unique symbol;
export type Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: true;
};
export const __ssrg$instance$Debug$0 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_debug_Debug<Packet<A>> => (_ssrg_debug_boundedDebug((value: Packet<A>): string => { switch (value.tag) { case "Packet": return "Packet" + " " + (__ssrg$evidence$0).debug(value.value); } }));
export const __ssrg$instance$Debug$1 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_debug_Debug<Box<A>> => (_ssrg_debug_boundedDebug((value: Box<A>): string => "Box { " + "value: " + (__ssrg$evidence$0).debug(value["value"]) + " }"));
const addMaybe = (left: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (right: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => _ssrg_maybe_monad["flatMap"]((a: number) => _ssrg_maybe_monad["flatMap"]((b: number) => _ssrg_maybe_applicative["pure"](_ssrg_int_add(a, b)))(right))(left)
const wrapMaybe = <A,>(value: A) => _ssrg_maybe_Just(value)
const wrapEither = <A,>(value: A) => _ssrg_either_Right(value)
const wrapArray = <A,>(value: A) => [value]
const wrapList = <A,>(value: A) => _ssrg_list_from_array([value])
const wrapPacket = <A,>(value: A) => Packet(value)
const wrapBox = <A,>(value: A) => (({ "value": value } as const) as unknown as Box<A>)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_maybeDebug<number>(_ssrg_debug_intDebug)["debug"](success)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_maybeDebug<number>(_ssrg_debug_intDebug)["debug"](stopped)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_maybeDebug<number>(_ssrg_debug_intDebug)["debug"](wrapped)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_eitherDebug<string, number>(_ssrg_debug_stringDebug, _ssrg_debug_intDebug)["debug"](eitherValue)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_intDebug)["debug"](arrayValue)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_listDebug<number>(_ssrg_debug_intDebug)["debug"](listValue)), () => _ssrg_effect_flatMap(_ssrg_console_println(__ssrg$instance$Debug$0<number>(_ssrg_debug_intDebug)["debug"](packetValue)), () => _ssrg_console_println(__ssrg$instance$Debug$1<number>(_ssrg_debug_intDebug)["debug"](boxValue)))))))))
const success: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = addMaybe(_ssrg_maybe_Just(20))(_ssrg_maybe_Just(22));
const stopped: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = addMaybe(_ssrg_maybe_Just(20))(_ssrg_maybe_Nothing);
const wrapped: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = wrapMaybe(42);
const eitherValue: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: number } = wrapEither(42);
const arrayValue: ReadonlyArray<number> = wrapArray(42);
const listValue: List<number> = wrapList(42);
const packetValue: Packet<number> = wrapPacket(42);
const boxValue: Box<number> = wrapBox(42);
