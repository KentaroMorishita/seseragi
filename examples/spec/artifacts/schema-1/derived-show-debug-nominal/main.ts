import { boundedShow as _ssrg_show_boundedShow, boundedDebug as _ssrg_debug_boundedDebug, intShow as _ssrg_show_intShow, intDebug as _ssrg_debug_intDebug, arrayShow as _ssrg_show_arrayShow, arrayDebug as _ssrg_debug_arrayDebug, stringShow as _ssrg_show_stringShow, rangeShow as _ssrg_show_rangeShow, tupleShow as _ssrg_show_tupleShow, boolShow as _ssrg_show_boolShow, recordShow as _ssrg_show_recordShow, stringDebug as _ssrg_debug_stringDebug, rangeDebug as _ssrg_debug_rangeDebug, tupleDebug as _ssrg_debug_tupleDebug, boolDebug as _ssrg_debug_boolDebug, recordDebug as _ssrg_debug_recordDebug, floatDebug as _ssrg_debug_floatDebug, type Show as _ssrg_show_Show, type Debug as _ssrg_debug_Debug } from "@seseragi/runtime/show"
import { inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type UserId =
  | { readonly tag: "UserId"; readonly value: bigint };
export const UserId = (value: bigint): UserId => ({ tag: "UserId", value } as const);
export type Packet<A> =
  | { readonly tag: "Empty" }
  | { readonly tag: "Packet"; readonly value: ReadonlyArray<A> };
export const Empty = { tag: "Empty" } as const;
export const Packet = <A>(value: ReadonlyArray<A>): Packet<A> => ({ tag: "Packet", value } as const);
declare const __ssrg$brand$Envelope: unique symbol;
export type Envelope<A> = {
  readonly "label": string;
  readonly "value": A;
  readonly "range": Readonly<{ start: bigint; end: bigint; inclusive: boolean }>;
  readonly "summary": readonly [bigint, string];
  readonly "metadata": { readonly "active": boolean };
  readonly [__ssrg$brand$Envelope]: true;
};
declare const __ssrg$brand$Stats: unique symbol;
export type Stats = {
  readonly "total": bigint;
  readonly "ratio": number;
  readonly [__ssrg$brand$Stats]: true;
};
export const __ssrg$instance$Show$0: _ssrg_show_Show<UserId> = _ssrg_show_boundedShow((value: UserId): string => { switch (value.tag) { case "UserId": return "UserId" + " " + _ssrg_show_intShow.show(value.value); } });
export const __ssrg$instance$Debug$1: _ssrg_debug_Debug<UserId> = _ssrg_debug_boundedDebug((value: UserId): string => { switch (value.tag) { case "UserId": return "UserId" + " " + _ssrg_debug_intDebug.debug(value.value); } });
export const __ssrg$instance$Show$2 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_show_Show<Packet<A>> => (_ssrg_show_boundedShow((value: Packet<A>): string => { switch (value.tag) { case "Empty": return "Empty"; case "Packet": return "Packet" + " " + (_ssrg_show_arrayShow<A>(__ssrg$evidence$0)).show(value.value); } }));
export const __ssrg$instance$Debug$3 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_debug_Debug<Packet<A>> => (_ssrg_debug_boundedDebug((value: Packet<A>): string => { switch (value.tag) { case "Empty": return "Empty"; case "Packet": return "Packet" + " " + (_ssrg_debug_arrayDebug<A>(__ssrg$evidence$0)).debug(value.value); } }));
export const __ssrg$instance$Show$4 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_show_Show<Envelope<A>> => (_ssrg_show_boundedShow((value: Envelope<A>): string => "Envelope { " + "label: " + _ssrg_show_stringShow.show(value["label"]) + ", " + "value: " + (__ssrg$evidence$0).show(value["value"]) + ", " + "range: " + (_ssrg_show_rangeShow<bigint>(_ssrg_show_intShow)).show(value["range"]) + ", " + "summary: " + (_ssrg_show_tupleShow<readonly [bigint, string]>(_ssrg_show_intShow, _ssrg_show_stringShow)).show(value["summary"]) + ", " + "metadata: " + (_ssrg_show_recordShow<{ readonly "active": boolean }>(["active"] as const, [false] as const, _ssrg_show_boolShow)).show(value["metadata"]) + " }"));
export const __ssrg$instance$Debug$5 = <A,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>): _ssrg_debug_Debug<Envelope<A>> => (_ssrg_debug_boundedDebug((value: Envelope<A>): string => "Envelope { " + "label: " + _ssrg_debug_stringDebug.debug(value["label"]) + ", " + "value: " + (__ssrg$evidence$0).debug(value["value"]) + ", " + "range: " + (_ssrg_debug_rangeDebug<bigint>(_ssrg_debug_intDebug)).debug(value["range"]) + ", " + "summary: " + (_ssrg_debug_tupleDebug<readonly [bigint, string]>(_ssrg_debug_intDebug, _ssrg_debug_stringDebug)).debug(value["summary"]) + ", " + "metadata: " + (_ssrg_debug_recordDebug<{ readonly "active": boolean }>(["active"] as const, [false] as const, _ssrg_debug_boolDebug)).debug(value["metadata"]) + " }"));
export const __ssrg$instance$Debug$6: _ssrg_debug_Debug<Stats> = _ssrg_debug_boundedDebug((value: Stats): string => "Stats { " + "total: " + _ssrg_debug_intDebug.debug(value["total"]) + ", " + "ratio: " + _ssrg_debug_floatDebug.debug(value["ratio"]) + " }");
const packet: Packet<UserId> = Packet([UserId(7n), UserId(9n)]);
const empty: Packet<bigint> = Empty;
const envelope: Envelope<Packet<UserId>> = (({ "label": "metrics", "value": packet, "range": _ssrg_range_inclusive(1n, 3n), "summary": [42n, "ready"] as const, "metadata": ({ "active": true } as const) } as const) as unknown as Envelope<Packet<UserId>>);
const stats: Stats = (({ "total": _ssrg_int64_add(20n, 22n), "ratio": 1.25 } as const) as unknown as Stats);
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(__ssrg$instance$Show$2<UserId>(__ssrg$instance$Show$0)["show"](packet)), () => _ssrg_effect_flatMap(_ssrg_console_println(__ssrg$instance$Debug$5<Packet<UserId>>(__ssrg$instance$Debug$3<UserId>(__ssrg$instance$Debug$1))["debug"](envelope)), () => _ssrg_effect_flatMap(_ssrg_console_println(__ssrg$instance$Debug$6["debug"](stats)), () => _ssrg_effect_flatMap(_ssrg_console_println(__ssrg$instance$Show$0["show"](UserId(42n))), () => _ssrg_console_println(__ssrg$instance$Show$2<bigint>(_ssrg_show_intShow)["show"](empty))))))
