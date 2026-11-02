import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedDebug as _ssrg_debug_boundedDebug, intDebug as _ssrg_debug_intDebug, maybeDebug as _ssrg_debug_maybeDebug, stringDebug as _ssrg_debug_stringDebug, eitherDebug as _ssrg_debug_eitherDebug, arrayDebug as _ssrg_debug_arrayDebug, listDebug as _ssrg_debug_listDebug, type Debug as _ssrg_debug_Debug } from "@seseragi/runtime/show"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing, maybeMonad as _ssrg_maybe_monad, maybeApplicative as _ssrg_maybe_applicative, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source306 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 306, end: 318 };
const $ssrg$source312 = { file: "main.ssrg", line: 16, column: 11, uri: "seseragi://artifact/top-level-call-inference", start: 312, end: 317 };
const $ssrg$source365 = { file: "main.ssrg", line: 20, column: 3, uri: "seseragi://artifact/top-level-call-inference", start: 365, end: 375 };
const $ssrg$source428 = { file: "main.ssrg", line: 23, column: 3, uri: "seseragi://artifact/top-level-call-inference", start: 428, end: 439 };
const $ssrg$source583 = { file: "main.ssrg", line: 32, column: 3, uri: "seseragi://artifact/top-level-call-inference", start: 583, end: 595 };
const $ssrg$source663 = { file: "main.ssrg", line: 37, column: 15, uri: "seseragi://artifact/top-level-call-inference", start: 663, end: 691 };
const $ssrg$source673 = { file: "main.ssrg", line: 37, column: 25, uri: "seseragi://artifact/top-level-call-inference", start: 673, end: 680 };
const $ssrg$source683 = { file: "main.ssrg", line: 37, column: 35, uri: "seseragi://artifact/top-level-call-inference", start: 683, end: 690 };
const $ssrg$source706 = { file: "main.ssrg", line: 38, column: 15, uri: "seseragi://artifact/top-level-call-inference", start: 706, end: 732 };
const $ssrg$source716 = { file: "main.ssrg", line: 38, column: 25, uri: "seseragi://artifact/top-level-call-inference", start: 716, end: 723 };
const $ssrg$source747 = { file: "main.ssrg", line: 39, column: 15, uri: "seseragi://artifact/top-level-call-inference", start: 747, end: 759 };
const $ssrg$source778 = { file: "main.ssrg", line: 40, column: 19, uri: "seseragi://artifact/top-level-call-inference", start: 778, end: 791 };
const $ssrg$source809 = { file: "main.ssrg", line: 41, column: 18, uri: "seseragi://artifact/top-level-call-inference", start: 809, end: 821 };
const $ssrg$source838 = { file: "main.ssrg", line: 42, column: 17, uri: "seseragi://artifact/top-level-call-inference", start: 838, end: 849 };
const $ssrg$source868 = { file: "main.ssrg", line: 43, column: 19, uri: "seseragi://artifact/top-level-call-inference", start: 868, end: 881 };
const $ssrg$source897 = { file: "main.ssrg", line: 44, column: 16, uri: "seseragi://artifact/top-level-call-inference", start: 897, end: 907 };
const $ssrg$source941 = { file: "main.ssrg", line: 48, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 941, end: 954 };
const $ssrg$source970 = { file: "main.ssrg", line: 49, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 970, end: 983 };
const $ssrg$source999 = { file: "main.ssrg", line: 50, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 999, end: 1012 };
const $ssrg$source1028 = { file: "main.ssrg", line: 51, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 1028, end: 1045 };
const $ssrg$source1061 = { file: "main.ssrg", line: 52, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 1061, end: 1077 };
const $ssrg$source1093 = { file: "main.ssrg", line: 53, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 1093, end: 1108 };
const $ssrg$source1124 = { file: "main.ssrg", line: 54, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 1124, end: 1141 };
const $ssrg$source1157 = { file: "main.ssrg", line: 55, column: 5, uri: "seseragi://artifact/top-level-call-inference", start: 1157, end: 1171 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
export interface $ssrg$kind$Packet extends $ssrg$Kind<readonly [unknown]> { readonly type: Packet<this["args"][0]> }
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
const addMaybe = (left: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (right: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => _ssrg_maybe_monad["flatMap"]((a: number) => _ssrg_maybe_monad["flatMap"]((b: number) => $ssrg$atSource($ssrg$source306, () => ((_ssrg_maybe_applicative["pure"](_ssrg_int_add(a, b, $ssrg$source312))) as { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }), false))(right))(left)
const wrapMaybe = <A,>(value: A) => $ssrg$atSource($ssrg$source365, () => _ssrg_maybe_Just(value), false)
const wrapEither = <A,>(value: A) => $ssrg$atSource($ssrg$source428, () => _ssrg_either_Right(value), false)
const wrapArray = <A,>(value: A) => [value]
const wrapList = <A,>(value: A) => _ssrg_list_from_array([value])
const wrapPacket = <A,>(value: A) => $ssrg$atSource($ssrg$source583, () => Packet<A>(value), false)
const wrapBox = <A,>(value: A) => (({ "value": value } as const) as unknown as Box<A>)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source941, () => _ssrg_console_println($ssrg$atSource($ssrg$source941, () => ((_ssrg_debug_maybeDebug<number>(_ssrg_debug_intDebug)["debug"](success)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source970, () => _ssrg_console_println($ssrg$atSource($ssrg$source970, () => ((_ssrg_debug_maybeDebug<number>(_ssrg_debug_intDebug)["debug"](stopped)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source999, () => _ssrg_console_println($ssrg$atSource($ssrg$source999, () => ((_ssrg_debug_maybeDebug<number>(_ssrg_debug_intDebug)["debug"](wrapped)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1028, () => _ssrg_console_println($ssrg$atSource($ssrg$source1028, () => ((_ssrg_debug_eitherDebug<string, number>(_ssrg_debug_stringDebug, _ssrg_debug_intDebug)["debug"](eitherValue)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1061, () => _ssrg_console_println($ssrg$atSource($ssrg$source1061, () => ((_ssrg_debug_arrayDebug<number>(_ssrg_debug_intDebug)["debug"](arrayValue)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1093, () => _ssrg_console_println($ssrg$atSource($ssrg$source1093, () => ((_ssrg_debug_listDebug<number>(_ssrg_debug_intDebug)["debug"](listValue)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1124, () => _ssrg_console_println($ssrg$atSource($ssrg$source1124, () => ((__ssrg$instance$Debug$0<number>(_ssrg_debug_intDebug)["debug"](packetValue)) as string), false)), true), () => $ssrg$atSource($ssrg$source1157, () => _ssrg_console_println($ssrg$atSource($ssrg$source1157, () => ((__ssrg$instance$Debug$1<number>(_ssrg_debug_intDebug)["debug"](boxValue)) as string), false)), true))))))))
const success: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = $ssrg$atSource($ssrg$source663, () => addMaybe($ssrg$atSource($ssrg$source673, () => _ssrg_maybe_Just(20), false))($ssrg$atSource($ssrg$source683, () => _ssrg_maybe_Just(22), false)), false);
const stopped: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = $ssrg$atSource($ssrg$source706, () => addMaybe($ssrg$atSource($ssrg$source716, () => _ssrg_maybe_Just(20), false))(_ssrg_maybe_Nothing), false);
const wrapped: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = $ssrg$atSource($ssrg$source747, () => wrapMaybe<number>(42), false);
const eitherValue: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: number } = $ssrg$atSource($ssrg$source778, () => wrapEither<number>(42), false);
const arrayValue: ReadonlyArray<number> = $ssrg$atSource($ssrg$source809, () => wrapArray<number>(42), false);
const listValue: List<number> = $ssrg$atSource($ssrg$source838, () => wrapList<number>(42), false);
const packetValue: Packet<number> = $ssrg$atSource($ssrg$source868, () => wrapPacket<number>(42), false);
const boxValue: Box<number> = $ssrg$atSource($ssrg$source897, () => wrapBox<number>(42), false);
