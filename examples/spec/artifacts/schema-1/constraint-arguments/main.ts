import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const chooseFallback = <C, A,>(values: C) => (fallback: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => fallback
