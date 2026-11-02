import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source426 = { file: "main.ssrg", line: 20, column: 3, uri: "seseragi://artifact/equality-operator-section", start: 426, end: 444 };
const $ssrg$source545 = { file: "main.ssrg", line: 27, column: 3, uri: "seseragi://artifact/equality-operator-section", start: 545, end: 563 };
const $ssrg$source654 = { file: "main.ssrg", line: 34, column: 3, uri: "seseragi://artifact/equality-operator-section", start: 654, end: 672 };
const $ssrg$source736 = { file: "main.ssrg", line: 38, column: 3, uri: "seseragi://artifact/equality-operator-section", start: 736, end: 765 };
const $ssrg$source1035 = { file: "main.ssrg", line: 49, column: 3, uri: "seseragi://artifact/equality-operator-section", start: 1035, end: 1237 };
const $ssrg$source1045 = { file: "main.ssrg", line: 49, column: 13, uri: "seseragi://artifact/equality-operator-section", start: 1045, end: 1237 };
const $ssrg$source1058 = { file: "main.ssrg", line: 50, column: 5, uri: "seseragi://artifact/equality-operator-section", start: 1058, end: 1090 };
const $ssrg$source1096 = { file: "main.ssrg", line: 51, column: 5, uri: "seseragi://artifact/equality-operator-section", start: 1096, end: 1130 };
const $ssrg$source1136 = { file: "main.ssrg", line: 52, column: 5, uri: "seseragi://artifact/equality-operator-section", start: 1136, end: 1165 };
const $ssrg$source1171 = { file: "main.ssrg", line: 53, column: 5, uri: "seseragi://artifact/equality-operator-section", start: 1171, end: 1200 };
const $ssrg$source1206 = { file: "main.ssrg", line: 54, column: 5, uri: "seseragi://artifact/equality-operator-section", start: 1206, end: 1233 };
$ssrg$assertUnicodeVersion("17.0.0")

type Status =
  | { readonly tag: "Ready" }
  | { readonly tag: "Waiting" };
const Ready: Status = { tag: "Ready" } as const;
const Waiting: Status = { tag: "Waiting" } as const;
export const __ssrg$instance$Eq$0 = { "eq": (left: Status) => (right: Status) => (($ssrg_match: readonly [Status, Status]): boolean => $ssrg_match[0].tag === "Ready" && $ssrg_match[1].tag === "Ready" ? true : $ssrg_match[0].tag === "Waiting" && $ssrg_match[1].tag === "Waiting" ? true : false)([left, right] as const) } as const;
const applyComparison = (compare: (argument: Status) => (argument: Status) => boolean) => (left: Status) => (right: Status) => $ssrg$atSource($ssrg$source426, () => compare(left)(right), false)
const applyIntComparison = (compare: (argument: number) => (argument: number) => boolean) => (left: number) => (right: number) => $ssrg$atSource($ssrg$source545, () => compare(left)(right), false)
const applyEquality = <T,>(compare: (argument: T) => (argument: T) => boolean) => (left: T) => (right: T) => $ssrg$atSource($ssrg$source654, () => compare(left)(right), false)
const genericSame = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: import("@seseragi/runtime/equality").Eq<T>) => $ssrg$atSource($ssrg$source736, () => applyEquality<T>(((__ssrg$evidence$0["eq"]) as (argument: T) => (argument: T) => boolean))(left)(right), false)
const render = (checks: readonly [boolean, boolean, boolean, boolean, boolean]) => (($ssrg_match: readonly [boolean, boolean, boolean, boolean, boolean]): string => $ssrg_match[0] === true && $ssrg_match[1] === true && $ssrg_match[2] === true && $ssrg_match[3] === true && $ssrg_match[4] === true ? "Equality sections: same / different" : "unexpected equality section result")(checks)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source1035, () => _ssrg_console_println($ssrg$atSource($ssrg$source1045, () => render([$ssrg$atSource($ssrg$source1058, () => applyComparison(((__ssrg$instance$Eq$0["eq"]) as (argument: Status) => (argument: Status) => boolean))(Ready)(Ready), false), $ssrg$atSource($ssrg$source1096, () => applyComparison((((() => { const __ssrg$equality$method: (argument: Status) => (argument: Status) => boolean = __ssrg$instance$Eq$0["eq"]; return (__ssrg$equality$left: Status) => (() => { const __ssrg$equality$stage: (argument: Status) => boolean = __ssrg$equality$method(__ssrg$equality$left); return (__ssrg$equality$right: Status) => __ssrg$equality$stage(__ssrg$equality$right) === false; })(); })()) as (argument: Status) => (argument: Status) => boolean))(Ready)(Waiting), false), $ssrg$atSource($ssrg$source1136, () => applyIntComparison(((_ssrg_int_eq_dictionary["eq"]) as (argument: number) => (argument: number) => boolean))(42)(42), false), $ssrg$atSource($ssrg$source1171, () => applyIntComparison((((() => { const __ssrg$equality$method: (argument: number) => (argument: number) => boolean = _ssrg_int_eq_dictionary["eq"]; return (__ssrg$equality$left: number) => (() => { const __ssrg$equality$stage: (argument: number) => boolean = __ssrg$equality$method(__ssrg$equality$left); return (__ssrg$equality$right: number) => __ssrg$equality$stage(__ssrg$equality$right) === false; })(); })()) as (argument: number) => (argument: number) => boolean))(42)(41), false), $ssrg$atSource($ssrg$source1206, () => ((genericSame<Status>(Waiting)(Waiting)(__ssrg$instance$Eq$0)) as boolean), false)] as const), false)), true)
