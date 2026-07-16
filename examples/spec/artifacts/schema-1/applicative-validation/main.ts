import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

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
  | { readonly tag: "User"; readonly value: readonly [string, bigint] };
export const User = (value: readonly [string, bigint]): User => ({ tag: "User", value } as const);
export const __ssrg$instance$Functor$0 = <E,>() => ({ "map": <A, B,>(f: (argument: A) => B) => (value: Validation<E, A>) => (($ssrg_match: Validation<E, A>): Validation<E, B> => $ssrg_match.tag === "Valid" ? ((item: A): Validation<E, B> => Valid(f(item)))($ssrg_match.value) : $ssrg_match.tag === "Invalid" ? ((errors: Errors<E>): Validation<E, B> => Invalid(errors))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value) }) as const;
export const __ssrg$instance$Applicative$1 = <E,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "pure": <A,>(value: A) => Valid(value), "apply": <A, B,>(wrapped: Validation<E, (argument: A) => B>) => (value: Validation<E, A>) => (($ssrg_match: readonly [Validation<E, (argument: A) => B>, Validation<E, A>]): Validation<E, B> => $ssrg_match[0].tag === "Valid" && $ssrg_match[1].tag === "Valid" ? ((_function: (argument: A) => B, item: A): Validation<E, B> => Valid(_function(item)))($ssrg_match[0].value, $ssrg_match[1].value) : $ssrg_match[0].tag === "Invalid" && $ssrg_match[1].tag === "Invalid" ? ((left: Errors<E>, right: Errors<E>): Validation<E, B> => Invalid(appendErrors(left)(right)))($ssrg_match[0].value, $ssrg_match[1].value) : $ssrg_match[0].tag === "Invalid" ? ((errors: Errors<E>): Validation<E, B> => Invalid(errors))($ssrg_match[0].value) : $ssrg_match[1].tag === "Invalid" ? ((errors: Errors<E>): Validation<E, B> => Invalid(errors))($ssrg_match[1].value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())([wrapped, value] as const) }) as const;
const appendErrors = <E,>(left: Errors<E>) => (right: Errors<E>) => (($ssrg_match: Errors<E>): Errors<E> => $ssrg_match.tag === "One" ? ((error: E): Errors<E> => More([error, right] as const))($ssrg_match.value) : $ssrg_match.tag === "More" ? ((error: E, rest: Errors<E>): Errors<E> => More([error, appendErrors(rest)(right)] as const))($ssrg_match.value[0], $ssrg_match.value[1]) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(left)
const makeUser = (name: string) => (age: bigint) => User([name, age] as const)
const validateName = (name: string) => name === "" ? Invalid(One(NameRequired)) : Valid(name)
const validateAge = (age: bigint) => age > 0n ? Valid(age) : Invalid(One(AgeMustBePositive))
const validateUser = (name: string) => (age: bigint) => __ssrg$instance$Applicative$1<FormError>(__ssrg$instance$Functor$0<FormError>())["apply"](__ssrg$instance$Applicative$1<FormError>(__ssrg$instance$Functor$0<FormError>())["apply"](__ssrg$instance$Applicative$1<FormError>(__ssrg$instance$Functor$0<FormError>())["pure"](makeUser))(validateName(name)))(validateAge(age))
const renderRemainingErrors = (errors: Errors<FormError>) => (($ssrg_match: Errors<FormError>): string => $ssrg_match.tag === "One" && $ssrg_match.value.tag === "AgeMustBePositive" ? "NameRequired, AgeMustBePositive" : "Multiple validation errors")(errors)
const renderErrors = (errors: Errors<FormError>) => (($ssrg_match: Errors<FormError>): string => $ssrg_match.tag === "More" && $ssrg_match.value[0].tag === "NameRequired" ? ((remaining: Errors<FormError>): string => renderRemainingErrors(remaining))($ssrg_match.value[1]) : "One validation error")(errors)
const render = (result: Validation<FormError, User>) => (($ssrg_match: Validation<FormError, User>): string => $ssrg_match.tag === "Valid" ? "Valid user" : $ssrg_match.tag === "Invalid" ? ((errors: Errors<FormError>): string => renderErrors(errors))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(result)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(render(validateUser("")(0n))), () => _ssrg_console_println(render(validateUser("Mio")(20n))))
