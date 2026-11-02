import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source220 = { file: "project/transitive-instance-dispatch/src/provider.ssrg", line: 14, column: 3, uri: "seseragi://fixture/transitive-instance-dispatch::provider", start: 220, end: 231 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "provider ready" } as const;
export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source220, () => ((__ssrg$evidence$0["ready"](value)) as string), false)
