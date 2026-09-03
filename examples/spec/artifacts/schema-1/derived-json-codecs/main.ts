import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { stringJsonEncode as _ssrg_string_json_encode, derivedStructJsonEncode as _ssrg_json_derivedstruct_encode, stringJsonDecode as _ssrg_string_json_decode, derivedStructJsonDecode as _ssrg_json_derivedstruct_decode, intJsonEncode as _ssrg_int_json_encode, derivedNewtypeJsonEncode as _ssrg_json_derivednewtype_encode, intJsonDecode as _ssrg_int_json_decode, derivedNewtypeJsonDecode as _ssrg_json_derivednewtype_decode, tupleJsonEncode as _ssrg_tuple_json_encode, derivedAdtJsonEncode as _ssrg_json_derivedadt_encode, tupleJsonDecode as _ssrg_tuple_json_decode, derivedAdtJsonDecode as _ssrg_json_derivedadt_decode, recordJsonEncode as _ssrg_record_json_encode, recordJsonDecode as _ssrg_record_json_decode, decodeString as _ssrg_json_decodeString, encodeString as _ssrg_json_encodeString, type JsonReadError as JsonReadError, type JsonEncode as _ssrg_json_JsonEncode, type JsonDecode as _ssrg_json_JsonDecode } from "@seseragi/runtime/json"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

type UserId =
  | { readonly tag: "UserId"; readonly value: number };
const UserId = (value: number): UserId => ({ tag: "UserId", value } as const);
type Tree<A> =
  | { readonly tag: "Leaf"; readonly value: A }
  | { readonly tag: "Branch"; readonly value: readonly [Tree<A>, Tree<A>] };
const Leaf = <A>(value: A): Tree<A> => ({ tag: "Leaf", value } as const);
const Branch = <A>(value: readonly [Tree<A>, Tree<A>]): Tree<A> => ({ tag: "Branch", value } as const);
type Event =
  | { readonly tag: "Idle" }
  | { readonly tag: "Named"; readonly value: { readonly "label": string; readonly "count": number } };
const Idle: Event = { tag: "Idle" } as const;
const Named = (value: { readonly "label": string; readonly "count": number }): Event => ({ tag: "Named", value } as const);
declare const __ssrg$brand$Profile: unique symbol;
type Profile<A> = {
  readonly "name": string;
  readonly "value": A;
  readonly [__ssrg$brand$Profile]: true;
};
export const __ssrg$instance$JsonEncode$0 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_json_JsonEncode<Profile<A>> => (_ssrg_json_derivedstruct_encode<Profile<A>>(["name", "value"], [() => (_ssrg_string_json_encode), () => ((__ssrg$evidence$0))]));
export const __ssrg$instance$JsonDecode$1 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_json_JsonDecode<Profile<A>> => (_ssrg_json_derivedstruct_decode<Profile<A>>(["name", "value"], [() => (_ssrg_string_json_decode), () => ((__ssrg$evidence$0))]));
export const __ssrg$instance$JsonEncode$2: _ssrg_json_JsonEncode<UserId> = _ssrg_json_derivednewtype_encode<UserId>(() => (_ssrg_int_json_encode));
export const __ssrg$instance$JsonDecode$3: _ssrg_json_JsonDecode<UserId> = _ssrg_json_derivednewtype_decode<UserId>("UserId", () => (_ssrg_int_json_decode));
export const __ssrg$instance$JsonEncode$4 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_json_JsonEncode<Tree<A>> => (_ssrg_json_derivedadt_encode<Tree<A>>([["Leaf", () => ((__ssrg$evidence$0))], ["Branch", () => ((_ssrg_tuple_json_encode<readonly [Tree<A>, Tree<A>]>(__ssrg$instance$JsonEncode$4<A>(__ssrg$evidence$0), __ssrg$instance$JsonEncode$4<A>(__ssrg$evidence$0))))]]));
export const __ssrg$instance$JsonDecode$5 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_json_JsonDecode<Tree<A>> => (_ssrg_json_derivedadt_decode<Tree<A>>([["Leaf", () => ((__ssrg$evidence$0))], ["Branch", () => ((_ssrg_tuple_json_decode<readonly [Tree<A>, Tree<A>]>(__ssrg$instance$JsonDecode$5<A>(__ssrg$evidence$0), __ssrg$instance$JsonDecode$5<A>(__ssrg$evidence$0))))]]));
export const __ssrg$instance$JsonEncode$6: _ssrg_json_JsonEncode<Event> = _ssrg_json_derivedadt_encode<Event>([["Idle", undefined], ["Named", () => ((_ssrg_record_json_encode<{ readonly "label": string; readonly "count": number }>(["count", "label"] as const, [false, false] as const, _ssrg_int_json_encode, _ssrg_string_json_encode)))]]);
export const __ssrg$instance$JsonDecode$7: _ssrg_json_JsonDecode<Event> = _ssrg_json_derivedadt_decode<Event>([["Idle", undefined], ["Named", () => ((_ssrg_record_json_decode<{ readonly "label": string; readonly "count": number }>(["count", "label"] as const, [false, false] as const, _ssrg_int_json_decode, _ssrg_string_json_decode)))]]);
const decodeProfile = (text: string) => _ssrg_json_decodeString(text, __ssrg$instance$JsonDecode$1<number>(_ssrg_int_json_decode))
const decodeTree = (text: string) => _ssrg_json_decodeString(text, __ssrg$instance$JsonDecode$5<number>(_ssrg_int_json_decode))
const decodeEvent = (text: string) => _ssrg_json_decodeString(text, __ssrg$instance$JsonDecode$7)
const normalizeProfile = (text: string) => (($ssrg_match: { readonly tag: "Left"; readonly value: JsonReadError } | { readonly tag: "Right"; readonly value: Profile<number> }): string => $ssrg_match.tag === "Left" ? "error" : $ssrg_match.tag === "Right" ? ((value: Profile<number>): string => _ssrg_json_encodeString(value, __ssrg$instance$JsonEncode$0<number>(_ssrg_int_json_encode)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(decodeProfile(text))
const normalizeTree = (text: string) => (($ssrg_match: { readonly tag: "Left"; readonly value: JsonReadError } | { readonly tag: "Right"; readonly value: Tree<number> }): string => $ssrg_match.tag === "Left" ? "error" : $ssrg_match.tag === "Right" ? ((value: Tree<number>): string => _ssrg_json_encodeString(value, __ssrg$instance$JsonEncode$4<number>(_ssrg_int_json_encode)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(decodeTree(text))
const normalizeEvent = (text: string) => (($ssrg_match: { readonly tag: "Left"; readonly value: JsonReadError } | { readonly tag: "Right"; readonly value: Event }): string => $ssrg_match.tag === "Left" ? "error" : $ssrg_match.tag === "Right" ? ((value: Event): string => _ssrg_json_encodeString(value, __ssrg$instance$JsonEncode$6))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(decodeEvent(text))
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_json_encodeString((({ "name": "count", "value": 3 } as const) as unknown as Profile<number>), __ssrg$instance$JsonEncode$0<number>(_ssrg_int_json_encode))), () => _ssrg_effect_flatMap(_ssrg_console_println(normalizeProfile("{\"name\":\"count\",\"value\":3}")), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_json_encodeString(UserId(7), __ssrg$instance$JsonEncode$2)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_json_encodeString(Branch([Leaf(1), Leaf(2)] as const), __ssrg$instance$JsonEncode$4<number>(_ssrg_int_json_encode))), () => _ssrg_effect_flatMap(_ssrg_console_println(normalizeTree("{\"tag\":\"Branch\",\"value\":[{\"tag\":\"Leaf\",\"value\":1},{\"tag\":\"Leaf\",\"value\":2}]}")), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_json_encodeString(Named(({ "label": "ready", "count": 2 } as const)), __ssrg$instance$JsonEncode$6)), () => _ssrg_console_println(normalizeEvent("{\"tag\":\"Named\",\"value\":{\"count\":2,\"label\":\"ready\"}}"))))))))
