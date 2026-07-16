export type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
export const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: Box<A>) => (($ssrg_match: Box<A>): Box<B> => $ssrg_match.tag === "Boxed" ? ((item: A): Box<B> => Boxed(f(item)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value) } as const;
export const transform = <F, A, B,>(f: (argument: A) => B) => (value: unknown) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["map"](f)(value)
