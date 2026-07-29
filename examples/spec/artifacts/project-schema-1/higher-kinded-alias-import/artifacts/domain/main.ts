export type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
export const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const keepMaybe = (value: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, number] }) => value
export const keepEither = (value: (argument: number) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [number, number] }) => value
