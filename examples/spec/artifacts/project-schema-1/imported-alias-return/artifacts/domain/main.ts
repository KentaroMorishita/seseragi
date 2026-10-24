import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export type Box<A> =
  | { readonly tag: "Box"; readonly value: A };
export const Box = <A>(value: A): Box<A> => ({ tag: "Box", value } as const);
export const boxed = <A,>(value: A) => Box<A>(value)
