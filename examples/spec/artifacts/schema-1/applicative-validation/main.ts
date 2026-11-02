import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringEq as _ssrg_string_eq_dictionary, intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source354 = { file: "main.ssrg", line: 16, column: 21, uri: "seseragi://artifact/applicative-validation", start: 354, end: 368 };
const $ssrg$source362 = { file: "main.ssrg", line: 16, column: 29, uri: "seseragi://artifact/applicative-validation", start: 362, end: 368 };
const $ssrg$source393 = { file: "main.ssrg", line: 17, column: 25, uri: "seseragi://artifact/applicative-validation", start: 393, end: 407 };
const $ssrg$source658 = { file: "main.ssrg", line: 29, column: 18, uri: "seseragi://artifact/applicative-validation", start: 658, end: 677 };
const $ssrg$source704 = { file: "main.ssrg", line: 30, column: 27, uri: "seseragi://artifact/applicative-validation", start: 704, end: 741 };
const $ssrg$source717 = { file: "main.ssrg", line: 30, column: 40, uri: "seseragi://artifact/applicative-validation", start: 717, end: 740 };
const $ssrg$source835 = { file: "main.ssrg", line: 34, column: 45, uri: "seseragi://artifact/applicative-validation", start: 835, end: 846 };
const $ssrg$source1024 = { file: "main.ssrg", line: 41, column: 39, uri: "seseragi://artifact/applicative-validation", start: 1024, end: 1045 };
const $ssrg$source1032 = { file: "main.ssrg", line: 41, column: 47, uri: "seseragi://artifact/applicative-validation", start: 1032, end: 1045 };
const $ssrg$source1085 = { file: "main.ssrg", line: 42, column: 40, uri: "seseragi://artifact/applicative-validation", start: 1085, end: 1118 };
const $ssrg$source1095 = { file: "main.ssrg", line: 42, column: 50, uri: "seseragi://artifact/applicative-validation", start: 1095, end: 1118 };
const $ssrg$source1148 = { file: "main.ssrg", line: 43, column: 30, uri: "seseragi://artifact/applicative-validation", start: 1148, end: 1162 };
const $ssrg$source1192 = { file: "main.ssrg", line: 44, column: 30, uri: "seseragi://artifact/applicative-validation", start: 1192, end: 1206 };
const $ssrg$source1366 = { file: "main.ssrg", line: 56, column: 3, uri: "seseragi://artifact/applicative-validation", start: 1366, end: 1382 };
const $ssrg$source1453 = { file: "main.ssrg", line: 59, column: 6, uri: "seseragi://artifact/applicative-validation", start: 1453, end: 1463 };
const $ssrg$source1469 = { file: "main.ssrg", line: 59, column: 22, uri: "seseragi://artifact/applicative-validation", start: 1469, end: 1495 };
const $ssrg$source1479 = { file: "main.ssrg", line: 59, column: 32, uri: "seseragi://artifact/applicative-validation", start: 1479, end: 1495 };
const $ssrg$source1501 = { file: "main.ssrg", line: 59, column: 54, uri: "seseragi://artifact/applicative-validation", start: 1501, end: 1511 };
const $ssrg$source1587 = { file: "main.ssrg", line: 62, column: 19, uri: "seseragi://artifact/applicative-validation", start: 1587, end: 1596 };
const $ssrg$source1602 = { file: "main.ssrg", line: 62, column: 34, uri: "seseragi://artifact/applicative-validation", start: 1602, end: 1633 };
const $ssrg$source1612 = { file: "main.ssrg", line: 62, column: 44, uri: "seseragi://artifact/applicative-validation", start: 1612, end: 1633 };
const $ssrg$source1711 = { file: "main.ssrg", line: 65, column: 3, uri: "seseragi://artifact/applicative-validation", start: 1711, end: 1724 };
const $ssrg$source1731 = { file: "main.ssrg", line: 66, column: 7, uri: "seseragi://artifact/applicative-validation", start: 1731, end: 1748 };
const $ssrg$source1755 = { file: "main.ssrg", line: 67, column: 7, uri: "seseragi://artifact/applicative-validation", start: 1755, end: 1770 };
const $ssrg$source2067 = { file: "main.ssrg", line: 77, column: 39, uri: "seseragi://artifact/applicative-validation", start: 2067, end: 2098 };
const $ssrg$source2261 = { file: "main.ssrg", line: 84, column: 23, uri: "seseragi://artifact/applicative-validation", start: 2261, end: 2280 };
const $ssrg$source2318 = { file: "main.ssrg", line: 89, column: 5, uri: "seseragi://artifact/applicative-validation", start: 2318, end: 2335 };
const $ssrg$source2370 = { file: "main.ssrg", line: 93, column: 5, uri: "seseragi://artifact/applicative-validation", start: 2370, end: 2391 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Errors extends $ssrg$Kind<readonly [unknown]> { readonly type: Errors<this["args"][0]> }
export interface $ssrg$kind$Validation extends $ssrg$Kind<readonly [unknown, unknown]> { readonly type: Validation<this["args"][0], this["args"][1]> }
export type Errors<E> =
  | { readonly tag: "One"; readonly value: E }
  | { readonly tag: "More"; readonly value: readonly [E, Errors<E>] };
export const One = <E>(value: E): Errors<E> => ({ tag: "One", value } as const);
export const More = <E>(value: readonly [E, Errors<E>]): Errors<E> => ({ tag: "More", value } as const);
export type Validation<E, A> =
  | { readonly tag: "Valid"; readonly value: A }
  | { readonly tag: "Invalid"; readonly value: Errors<E> };
export const Valid = <E, A>(value: A): Validation<E, A> => ({ tag: "Valid", value } as const);
export const Invalid = <E, A>(value: Errors<E>): Validation<E, A> => ({ tag: "Invalid", value } as const);
export type FormError =
  | { readonly tag: "NameRequired" }
  | { readonly tag: "AgeMustBePositive" };
export const NameRequired: FormError = { tag: "NameRequired" } as const;
export const AgeMustBePositive: FormError = { tag: "AgeMustBePositive" } as const;
export type User =
  | { readonly tag: "User"; readonly value: readonly [string, number] };
export const User = (value: readonly [string, number]): User => ({ tag: "User", value } as const);
export const __ssrg$instance$Functor$0 = <E,>() => ({ "map": <A, B,>(f: (argument: A) => B) => (value: Validation<E, A>) => (($ssrg_match: Validation<E, A>): Validation<E, B> => $ssrg_match.tag === "Valid" ? ((item: A): Validation<E, B> => $ssrg$atSource($ssrg$source354, () => Valid<E, B>($ssrg$atSource($ssrg$source362, () => f(item), false)), false))($ssrg_match.value) : $ssrg_match.tag === "Invalid" ? ((errors: Errors<E>): Validation<E, B> => $ssrg$atSource($ssrg$source393, () => Invalid<E, B>(errors), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) }) as const;
export const __ssrg$instance$Applicative$1 = <E,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "pure": <A,>(value: A) => $ssrg$atSource($ssrg$source835, () => Valid<E, A>(value), false), "apply": <A, B,>(wrapped: Validation<E, (argument: A) => B>) => (value: Validation<E, A>) => (($ssrg_match: readonly [Validation<E, (argument: A) => B>, Validation<E, A>]): Validation<E, B> => $ssrg_match[0].tag === "Valid" && $ssrg_match[1].tag === "Valid" ? ((_function: (argument: A) => B, item: A): Validation<E, B> => $ssrg$atSource($ssrg$source1024, () => Valid<E, B>($ssrg$atSource($ssrg$source1032, () => _function(item), false)), false))($ssrg_match[0].value, $ssrg_match[1].value) : $ssrg_match[0].tag === "Invalid" && $ssrg_match[1].tag === "Invalid" ? ((left: Errors<E>, right: Errors<E>): Validation<E, B> => $ssrg$atSource($ssrg$source1085, () => Invalid<E, B>($ssrg$atSource($ssrg$source1095, () => appendErrors<E>(left)(right), false)), false))($ssrg_match[0].value, $ssrg_match[1].value) : $ssrg_match[0].tag === "Invalid" ? ((errors: Errors<E>): Validation<E, B> => $ssrg$atSource($ssrg$source1148, () => Invalid<E, B>(errors), false))($ssrg_match[0].value) : $ssrg_match[1].tag === "Invalid" ? ((errors: Errors<E>): Validation<E, B> => $ssrg$atSource($ssrg$source1192, () => Invalid<E, B>(errors), false))($ssrg_match[1].value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())([wrapped, value] as const) }) as const;
const appendErrors = <E,>(left: Errors<E>) => (right: Errors<E>) => (($ssrg_match: Errors<E>): Errors<E> => $ssrg_match.tag === "One" ? ((error: E): Errors<E> => $ssrg$atSource($ssrg$source658, () => More<E>([error, right] as const), false))($ssrg_match.value) : $ssrg_match.tag === "More" ? ((error: E, rest: Errors<E>): Errors<E> => $ssrg$atSource($ssrg$source704, () => More<E>([error, $ssrg$atSource($ssrg$source717, () => appendErrors(rest)(right), false)] as const), false))($ssrg_match.value[0], $ssrg_match.value[1]) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(left)
const makeUser = (name: string) => (age: number) => $ssrg$atSource($ssrg$source1366, () => User([name, age] as const), false)
const validateName = (name: string) => $ssrg$atSource($ssrg$source1453, () => _ssrg_string_eq_dictionary["eq"](name)(""), false) ? $ssrg$atSource($ssrg$source1469, () => Invalid<FormError, string>($ssrg$atSource($ssrg$source1479, () => One<FormError>(NameRequired), false)), false) : $ssrg$atSource($ssrg$source1501, () => Valid<FormError, string>(name), false)
const validateAge = (age: number) => (_ssrg_int_ord_dictionary["compare"](age)(0))["tag"] === "Greater" ? $ssrg$atSource($ssrg$source1587, () => Valid<FormError, number>(age), false) : $ssrg$atSource($ssrg$source1602, () => Invalid<FormError, number>($ssrg$atSource($ssrg$source1612, () => One<FormError>(AgeMustBePositive), false)), false)
const validateUser = (name: string) => (age: number) => $ssrg$atSource($ssrg$source1711, () => ((__ssrg$instance$Applicative$1<FormError>(__ssrg$instance$Functor$0<FormError>())["apply"]($ssrg$atSource($ssrg$source1711, () => ((__ssrg$instance$Applicative$1<FormError>(__ssrg$instance$Functor$0<FormError>())["apply"]($ssrg$atSource($ssrg$source1711, () => ((__ssrg$instance$Applicative$1<FormError>(__ssrg$instance$Functor$0<FormError>())["pure"](makeUser)) as Validation<FormError, (argument: string) => (argument: number) => User>), false))($ssrg$atSource($ssrg$source1731, () => validateName(name), false))) as Validation<FormError, (argument: number) => User>), false))($ssrg$atSource($ssrg$source1755, () => validateAge(age), false))) as Validation<FormError, User>), false)
const renderRemainingErrors = (errors: Errors<FormError>) => (($ssrg_match: Errors<FormError>): string => $ssrg_match.tag === "One" && $ssrg_match.value.tag === "AgeMustBePositive" ? "NameRequired, AgeMustBePositive" : "Multiple validation errors")(errors)
const renderErrors = (errors: Errors<FormError>) => (($ssrg_match: Errors<FormError>): string => $ssrg_match.tag === "More" && $ssrg_match.value[0].tag === "NameRequired" ? ((remaining: Errors<FormError>): string => $ssrg$atSource($ssrg$source2067, () => renderRemainingErrors(remaining), false))($ssrg_match.value[1]) : "One validation error")(errors)
const render = (result: Validation<FormError, User>) => (($ssrg_match: Validation<FormError, User>): string => $ssrg_match.tag === "Valid" ? "Valid user" : $ssrg_match.tag === "Invalid" ? ((errors: Errors<FormError>): string => $ssrg$atSource($ssrg$source2261, () => renderErrors(errors), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(result)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source2318, () => _ssrg_console_println($ssrg$atSource($ssrg$source2318, () => render($ssrg$atSource($ssrg$source2318, () => validateUser("")(0), false)), false)), true), () => $ssrg$atSource($ssrg$source2370, () => _ssrg_console_println($ssrg$atSource($ssrg$source2370, () => render($ssrg$atSource($ssrg$source2370, () => validateUser("Mio")(20), false)), false)), true))
