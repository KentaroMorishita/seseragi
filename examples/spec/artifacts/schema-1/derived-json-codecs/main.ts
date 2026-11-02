import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringJsonEncode as _ssrg_string_json_encode, derivedStructJsonEncode as _ssrg_json_derivedstruct_encode, stringJsonDecode as _ssrg_string_json_decode, derivedStructJsonDecode as _ssrg_json_derivedstruct_decode, intJsonEncode as _ssrg_int_json_encode, derivedNewtypeJsonEncode as _ssrg_json_derivednewtype_encode, intJsonDecode as _ssrg_int_json_decode, derivedNewtypeJsonDecode as _ssrg_json_derivednewtype_decode, tupleJsonEncode as _ssrg_tuple_json_encode, derivedAdtJsonEncode as _ssrg_json_derivedadt_encode, tupleJsonDecode as _ssrg_tuple_json_decode, derivedAdtJsonDecode as _ssrg_json_derivedadt_decode, recordJsonEncode as _ssrg_record_json_encode, recordJsonDecode as _ssrg_record_json_decode, decodeString as _ssrg_json_decodeString, encodeString as _ssrg_json_encodeString, type JsonReadError as JsonReadError, type JsonEncode as _ssrg_json_JsonEncode, type JsonDecode as _ssrg_json_JsonDecode } from "@seseragi/runtime/json"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source433 = { file: "main.ssrg", line: 19, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 433, end: 455 };
const $ssrg$source529 = { file: "main.ssrg", line: 22, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 529, end: 551 };
const $ssrg$source622 = { file: "main.ssrg", line: 25, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 622, end: 644 };
const $ssrg$source699 = { file: "main.ssrg", line: 28, column: 9, uri: "seseragi://artifact/derived-json-codecs", start: 699, end: 717 };
const $ssrg$source761 = { file: "main.ssrg", line: 30, column: 20, uri: "seseragi://artifact/derived-json-codecs", start: 761, end: 784 };
const $ssrg$source840 = { file: "main.ssrg", line: 34, column: 9, uri: "seseragi://artifact/derived-json-codecs", start: 840, end: 855 };
const $ssrg$source899 = { file: "main.ssrg", line: 36, column: 20, uri: "seseragi://artifact/derived-json-codecs", start: 899, end: 922 };
const $ssrg$source979 = { file: "main.ssrg", line: 40, column: 9, uri: "seseragi://artifact/derived-json-codecs", start: 979, end: 995 };
const $ssrg$source1039 = { file: "main.ssrg", line: 42, column: 20, uri: "seseragi://artifact/derived-json-codecs", start: 1039, end: 1062 };
const $ssrg$source1096 = { file: "main.ssrg", line: 46, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 1096, end: 1161 };
const $ssrg$source1105 = { file: "main.ssrg", line: 46, column: 12, uri: "seseragi://artifact/derived-json-codecs", start: 1105, end: 1160 };
const $ssrg$source1164 = { file: "main.ssrg", line: 47, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 1164, end: 1225 };
const $ssrg$source1173 = { file: "main.ssrg", line: 47, column: 12, uri: "seseragi://artifact/derived-json-codecs", start: 1173, end: 1224 };
const $ssrg$source1228 = { file: "main.ssrg", line: 48, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 1228, end: 1266 };
const $ssrg$source1237 = { file: "main.ssrg", line: 48, column: 12, uri: "seseragi://artifact/derived-json-codecs", start: 1237, end: 1265 };
const $ssrg$source1256 = { file: "main.ssrg", line: 48, column: 31, uri: "seseragi://artifact/derived-json-codecs", start: 1256, end: 1264 };
const $ssrg$source1269 = { file: "main.ssrg", line: 49, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 1269, end: 1322 };
const $ssrg$source1278 = { file: "main.ssrg", line: 49, column: 12, uri: "seseragi://artifact/derived-json-codecs", start: 1278, end: 1321 };
const $ssrg$source1297 = { file: "main.ssrg", line: 49, column: 31, uri: "seseragi://artifact/derived-json-codecs", start: 1297, end: 1320 };
const $ssrg$source1305 = { file: "main.ssrg", line: 49, column: 39, uri: "seseragi://artifact/derived-json-codecs", start: 1305, end: 1311 };
const $ssrg$source1313 = { file: "main.ssrg", line: 49, column: 47, uri: "seseragi://artifact/derived-json-codecs", start: 1313, end: 1319 };
const $ssrg$source1325 = { file: "main.ssrg", line: 50, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 1325, end: 1445 };
const $ssrg$source1334 = { file: "main.ssrg", line: 50, column: 12, uri: "seseragi://artifact/derived-json-codecs", start: 1334, end: 1444 };
const $ssrg$source1448 = { file: "main.ssrg", line: 51, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 1448, end: 1514 };
const $ssrg$source1457 = { file: "main.ssrg", line: 51, column: 12, uri: "seseragi://artifact/derived-json-codecs", start: 1457, end: 1513 };
const $ssrg$source1476 = { file: "main.ssrg", line: 51, column: 31, uri: "seseragi://artifact/derived-json-codecs", start: 1476, end: 1512 };
const $ssrg$source1517 = { file: "main.ssrg", line: 52, column: 3, uri: "seseragi://artifact/derived-json-codecs", start: 1517, end: 1607 };
const $ssrg$source1526 = { file: "main.ssrg", line: 52, column: 12, uri: "seseragi://artifact/derived-json-codecs", start: 1526, end: 1606 };
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
export const __ssrg$instance$JsonEncode$0 = <A,>(__ssrg$evidence$0: import("@seseragi/runtime/json").JsonEncode<A>): _ssrg_json_JsonEncode<Profile<A>> => (_ssrg_json_derivedstruct_encode<Profile<A>>(["name", "value"], [() => (_ssrg_string_json_encode), () => ((__ssrg$evidence$0))]));
export const __ssrg$instance$JsonDecode$1 = <A,>(__ssrg$evidence$0: import("@seseragi/runtime/json").JsonDecode<A>): _ssrg_json_JsonDecode<Profile<A>> => (_ssrg_json_derivedstruct_decode<Profile<A>>(["name", "value"], [() => (_ssrg_string_json_decode), () => ((__ssrg$evidence$0))]));
export const __ssrg$instance$JsonEncode$2: _ssrg_json_JsonEncode<UserId> = _ssrg_json_derivednewtype_encode<UserId>(() => (_ssrg_int_json_encode));
export const __ssrg$instance$JsonDecode$3: _ssrg_json_JsonDecode<UserId> = _ssrg_json_derivednewtype_decode<UserId>("UserId", () => (_ssrg_int_json_decode));
export const __ssrg$instance$JsonEncode$4 = <A,>(__ssrg$evidence$0: import("@seseragi/runtime/json").JsonEncode<A>): _ssrg_json_JsonEncode<Tree<A>> => (_ssrg_json_derivedadt_encode<Tree<A>>([["Leaf", () => ((__ssrg$evidence$0))], ["Branch", () => ((_ssrg_tuple_json_encode<readonly [Tree<A>, Tree<A>]>(__ssrg$instance$JsonEncode$4<A>(__ssrg$evidence$0), __ssrg$instance$JsonEncode$4<A>(__ssrg$evidence$0))))]]));
export const __ssrg$instance$JsonDecode$5 = <A,>(__ssrg$evidence$0: import("@seseragi/runtime/json").JsonDecode<A>): _ssrg_json_JsonDecode<Tree<A>> => (_ssrg_json_derivedadt_decode<Tree<A>>([["Leaf", () => ((__ssrg$evidence$0))], ["Branch", () => ((_ssrg_tuple_json_decode<readonly [Tree<A>, Tree<A>]>(__ssrg$instance$JsonDecode$5<A>(__ssrg$evidence$0), __ssrg$instance$JsonDecode$5<A>(__ssrg$evidence$0))))]]));
export const __ssrg$instance$JsonEncode$6: _ssrg_json_JsonEncode<Event> = _ssrg_json_derivedadt_encode<Event>([["Idle", undefined], ["Named", () => ((_ssrg_record_json_encode<{ readonly "label": string; readonly "count": number }>(["count", "label"] as const, [false, false] as const, _ssrg_int_json_encode, _ssrg_string_json_encode)))]]);
export const __ssrg$instance$JsonDecode$7: _ssrg_json_JsonDecode<Event> = _ssrg_json_derivedadt_decode<Event>([["Idle", undefined], ["Named", () => ((_ssrg_record_json_decode<{ readonly "label": string; readonly "count": number }>(["count", "label"] as const, [false, false] as const, _ssrg_int_json_decode, _ssrg_string_json_decode)))]]);
const decodeProfile = (text: string) => $ssrg$atSource($ssrg$source433, () => _ssrg_json_decodeString(text, __ssrg$instance$JsonDecode$1<number>(_ssrg_int_json_decode)), false)
const decodeTree = (text: string) => $ssrg$atSource($ssrg$source529, () => _ssrg_json_decodeString(text, __ssrg$instance$JsonDecode$5<number>(_ssrg_int_json_decode)), false)
const decodeEvent = (text: string) => $ssrg$atSource($ssrg$source622, () => _ssrg_json_decodeString(text, __ssrg$instance$JsonDecode$7), false)
const normalizeProfile = (text: string) => (($ssrg_match: { readonly tag: "Left"; readonly value: JsonReadError } | { readonly tag: "Right"; readonly value: Profile<number> }): string => $ssrg_match.tag === "Left" ? "error" : $ssrg_match.tag === "Right" ? ((value: Profile<number>): string => $ssrg$atSource($ssrg$source761, () => _ssrg_json_encodeString(value, __ssrg$instance$JsonEncode$0<number>(_ssrg_int_json_encode)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())($ssrg$atSource($ssrg$source699, () => decodeProfile(text), false))
const normalizeTree = (text: string) => (($ssrg_match: { readonly tag: "Left"; readonly value: JsonReadError } | { readonly tag: "Right"; readonly value: Tree<number> }): string => $ssrg_match.tag === "Left" ? "error" : $ssrg_match.tag === "Right" ? ((value: Tree<number>): string => $ssrg$atSource($ssrg$source899, () => _ssrg_json_encodeString(value, __ssrg$instance$JsonEncode$4<number>(_ssrg_int_json_encode)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())($ssrg$atSource($ssrg$source840, () => decodeTree(text), false))
const normalizeEvent = (text: string) => (($ssrg_match: { readonly tag: "Left"; readonly value: JsonReadError } | { readonly tag: "Right"; readonly value: Event }): string => $ssrg_match.tag === "Left" ? "error" : $ssrg_match.tag === "Right" ? ((value: Event): string => $ssrg$atSource($ssrg$source1039, () => _ssrg_json_encodeString(value, __ssrg$instance$JsonEncode$6), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())($ssrg$atSource($ssrg$source979, () => decodeEvent(text), false))
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1096, () => _ssrg_console_println($ssrg$atSource($ssrg$source1105, () => _ssrg_json_encodeString((({ "name": "count", "value": 3 } as const) as unknown as Profile<number>), __ssrg$instance$JsonEncode$0<number>(_ssrg_int_json_encode)), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1164, () => _ssrg_console_println($ssrg$atSource($ssrg$source1173, () => normalizeProfile("{\"name\":\"count\",\"value\":3}"), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1228, () => _ssrg_console_println($ssrg$atSource($ssrg$source1237, () => _ssrg_json_encodeString($ssrg$atSource($ssrg$source1256, () => UserId(7), false), __ssrg$instance$JsonEncode$2), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1269, () => _ssrg_console_println($ssrg$atSource($ssrg$source1278, () => _ssrg_json_encodeString($ssrg$atSource($ssrg$source1297, () => Branch<number>([$ssrg$atSource($ssrg$source1305, () => Leaf<number>(1), false), $ssrg$atSource($ssrg$source1313, () => Leaf<number>(2), false)] as const), false), __ssrg$instance$JsonEncode$4<number>(_ssrg_int_json_encode)), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1325, () => _ssrg_console_println($ssrg$atSource($ssrg$source1334, () => normalizeTree("{\"tag\":\"Branch\",\"value\":[{\"tag\":\"Leaf\",\"value\":1},{\"tag\":\"Leaf\",\"value\":2}]}"), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1448, () => _ssrg_console_println($ssrg$atSource($ssrg$source1457, () => _ssrg_json_encodeString($ssrg$atSource($ssrg$source1476, () => Named((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "label": "ready", "count": 2 } as const))), false), __ssrg$instance$JsonEncode$6), false)), true), () => $ssrg$atSource($ssrg$source1517, () => _ssrg_console_println($ssrg$atSource($ssrg$source1526, () => normalizeEvent("{\"tag\":\"Named\",\"value\":{\"count\":2,\"label\":\"ready\"}}"), false)), true)))))))
