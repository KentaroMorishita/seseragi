import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, boundedDebug as _ssrg_debug_boundedDebug, intShow as _ssrg_show_intShow, intDebug as _ssrg_debug_intDebug, arrayShow as _ssrg_show_arrayShow, arrayDebug as _ssrg_debug_arrayDebug, stringShow as _ssrg_show_stringShow, rangeShow as _ssrg_show_rangeShow, tupleShow as _ssrg_show_tupleShow, boolShow as _ssrg_show_boolShow, recordShow as _ssrg_show_recordShow, stringDebug as _ssrg_debug_stringDebug, rangeDebug as _ssrg_debug_rangeDebug, tupleDebug as _ssrg_debug_tupleDebug, boolDebug as _ssrg_debug_boolDebug, recordDebug as _ssrg_debug_recordDebug, floatDebug as _ssrg_debug_floatDebug, type Show as _ssrg_show_Show, type Debug as _ssrg_debug_Debug } from "@seseragi/runtime/show"
import { inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source488 = { file: "main.ssrg", line: 24, column: 3, uri: "seseragi://artifact/derived-show-debug-nominal", start: 488, end: 515 };
const $ssrg$source496 = { file: "main.ssrg", line: 24, column: 11, uri: "seseragi://artifact/derived-show-debug-nominal", start: 496, end: 504 };
const $ssrg$source506 = { file: "main.ssrg", line: 24, column: 21, uri: "seseragi://artifact/derived-show-debug-nominal", start: 506, end: 514 };
const $ssrg$source655 = { file: "main.ssrg", line: 32, column: 12, uri: "seseragi://artifact/derived-show-debug-nominal", start: 655, end: 660 };
const $ssrg$source767 = { file: "main.ssrg", line: 39, column: 12, uri: "seseragi://artifact/derived-show-debug-nominal", start: 767, end: 774 };
const $ssrg$source830 = { file: "main.ssrg", line: 45, column: 5, uri: "seseragi://artifact/derived-show-debug-nominal", start: 830, end: 851 };
const $ssrg$source839 = { file: "main.ssrg", line: 45, column: 14, uri: "seseragi://artifact/derived-show-debug-nominal", start: 839, end: 850 };
const $ssrg$source856 = { file: "main.ssrg", line: 46, column: 5, uri: "seseragi://artifact/derived-show-debug-nominal", start: 856, end: 880 };
const $ssrg$source865 = { file: "main.ssrg", line: 46, column: 14, uri: "seseragi://artifact/derived-show-debug-nominal", start: 865, end: 879 };
const $ssrg$source885 = { file: "main.ssrg", line: 47, column: 5, uri: "seseragi://artifact/derived-show-debug-nominal", start: 885, end: 906 };
const $ssrg$source894 = { file: "main.ssrg", line: 47, column: 14, uri: "seseragi://artifact/derived-show-debug-nominal", start: 894, end: 905 };
const $ssrg$source911 = { file: "main.ssrg", line: 48, column: 5, uri: "seseragi://artifact/derived-show-debug-nominal", start: 911, end: 937 };
const $ssrg$source920 = { file: "main.ssrg", line: 48, column: 14, uri: "seseragi://artifact/derived-show-debug-nominal", start: 920, end: 936 };
const $ssrg$source926 = { file: "main.ssrg", line: 48, column: 20, uri: "seseragi://artifact/derived-show-debug-nominal", start: 926, end: 935 };
const $ssrg$source942 = { file: "main.ssrg", line: 49, column: 5, uri: "seseragi://artifact/derived-show-debug-nominal", start: 942, end: 962 };
const $ssrg$source951 = { file: "main.ssrg", line: 49, column: 14, uri: "seseragi://artifact/derived-show-debug-nominal", start: 951, end: 961 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Envelope extends $ssrg$Kind<readonly [unknown]> { readonly type: Envelope<this["args"][0]> }
export interface $ssrg$kind$Packet extends $ssrg$Kind<readonly [unknown]> { readonly type: Packet<this["args"][0]> }
export type UserId =
  | { readonly tag: "UserId"; readonly value: number };
export const UserId = (value: number): UserId => ({ tag: "UserId", value } as const);
export type Packet<A> =
  | { readonly tag: "Empty" }
  | { readonly tag: "Packet"; readonly value: ReadonlyArray<A> };
export const Empty = { tag: "Empty" } as const;
export const Packet = <A>(value: ReadonlyArray<A>): Packet<A> => ({ tag: "Packet", value } as const);
declare const __ssrg$brand$Envelope: unique symbol;
export type Envelope<A> = {
  readonly "label": string;
  readonly "value": A;
  readonly "range": Readonly<{ start: number; end: number; inclusive: boolean }>;
  readonly "summary": readonly [number, string];
  readonly "metadata": { readonly "active": boolean };
  readonly [__ssrg$brand$Envelope]: true;
};
declare const __ssrg$brand$Stats: unique symbol;
export type Stats = {
  readonly "total": number;
  readonly "ratio": number;
  readonly [__ssrg$brand$Stats]: true;
};
export const __ssrg$instance$Show$0: _ssrg_show_Show<UserId> = _ssrg_show_boundedShow((value: UserId): string => { switch (value.tag) { case "UserId": return "UserId" + " " + _ssrg_show_intShow.show(value.value); } });
export const __ssrg$instance$Debug$1: _ssrg_debug_Debug<UserId> = _ssrg_debug_boundedDebug((value: UserId): string => { switch (value.tag) { case "UserId": return "UserId" + " " + _ssrg_debug_intDebug.debug(value.value); } });
export const __ssrg$instance$Show$2 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_show_Show<Packet<A>> => (_ssrg_show_boundedShow((value: Packet<A>): string => { switch (value.tag) { case "Empty": return "Empty"; case "Packet": return "Packet" + " " + (_ssrg_show_arrayShow<A>(__ssrg$evidence$0)).show(value.value); } }));
export const __ssrg$instance$Debug$3 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_debug_Debug<Packet<A>> => (_ssrg_debug_boundedDebug((value: Packet<A>): string => { switch (value.tag) { case "Empty": return "Empty"; case "Packet": return "Packet" + " " + (_ssrg_debug_arrayDebug<A>(__ssrg$evidence$0)).debug(value.value); } }));
export const __ssrg$instance$Show$4 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_show_Show<Envelope<A>> => (_ssrg_show_boundedShow((value: Envelope<A>): string => "Envelope { " + "label: " + _ssrg_show_stringShow.show(value["label"]) + ", " + "value: " + (__ssrg$evidence$0).show(value["value"]) + ", " + "range: " + (_ssrg_show_rangeShow<number>(_ssrg_show_intShow)).show(value["range"]) + ", " + "summary: " + (_ssrg_show_tupleShow<readonly [number, string]>(_ssrg_show_intShow, _ssrg_show_stringShow)).show(value["summary"]) + ", " + "metadata: " + (_ssrg_show_recordShow<{ readonly "active": boolean }>(["active"] as const, [false] as const, _ssrg_show_boolShow)).show(value["metadata"]) + " }"));
export const __ssrg$instance$Debug$5 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_debug_Debug<Envelope<A>> => (_ssrg_debug_boundedDebug((value: Envelope<A>): string => "Envelope { " + "label: " + _ssrg_debug_stringDebug.debug(value["label"]) + ", " + "value: " + (__ssrg$evidence$0).debug(value["value"]) + ", " + "range: " + (_ssrg_debug_rangeDebug<number>(_ssrg_debug_intDebug)).debug(value["range"]) + ", " + "summary: " + (_ssrg_debug_tupleDebug<readonly [number, string]>(_ssrg_debug_intDebug, _ssrg_debug_stringDebug)).debug(value["summary"]) + ", " + "metadata: " + (_ssrg_debug_recordDebug<{ readonly "active": boolean }>(["active"] as const, [false] as const, _ssrg_debug_boolDebug)).debug(value["metadata"]) + " }"));
export const __ssrg$instance$Debug$6: _ssrg_debug_Debug<Stats> = _ssrg_debug_boundedDebug((value: Stats): string => "Stats { " + "total: " + _ssrg_debug_intDebug.debug(value["total"]) + ", " + "ratio: " + _ssrg_debug_floatDebug.debug(value["ratio"]) + " }");
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source830, () => _ssrg_console_println($ssrg$atSource($ssrg$source839, () => ((__ssrg$instance$Show$2<UserId>(__ssrg$instance$Show$0)["show"](packet)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source856, () => _ssrg_console_println($ssrg$atSource($ssrg$source865, () => ((__ssrg$instance$Debug$5<Packet<UserId>>(__ssrg$instance$Debug$3<UserId>(__ssrg$instance$Debug$1))["debug"](envelope)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source885, () => _ssrg_console_println($ssrg$atSource($ssrg$source894, () => ((__ssrg$instance$Debug$6["debug"](stats)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source911, () => _ssrg_console_println($ssrg$atSource($ssrg$source920, () => ((__ssrg$instance$Show$0["show"]($ssrg$atSource($ssrg$source926, () => UserId(42), false))) as string), false)), true), () => $ssrg$atSource($ssrg$source942, () => _ssrg_console_println($ssrg$atSource($ssrg$source951, () => ((__ssrg$instance$Show$2<number>(_ssrg_show_intShow)["show"](empty)) as string), false)), true)))))
const packet: Packet<UserId> = $ssrg$atSource($ssrg$source488, () => Packet<UserId>([$ssrg$atSource($ssrg$source496, () => UserId(7), false), $ssrg$atSource($ssrg$source506, () => UserId(9), false)]), false);
const empty: Packet<number> = Empty;
const envelope: Envelope<Packet<UserId>> = (({ "label": "metrics", "value": packet, "range": $ssrg$atSource($ssrg$source655, () => _ssrg_range_inclusive(1, 3), false), "summary": [42, "ready"] as const, "metadata": (<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "active": true } as const)) } as const) as unknown as Envelope<Packet<UserId>>);
const stats: Stats = (({ "total": _ssrg_int_add(20, 22, $ssrg$source767), "ratio": 1.25 } as const) as unknown as Stats);
