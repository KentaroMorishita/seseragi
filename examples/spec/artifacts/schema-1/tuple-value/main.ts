import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const pair = (left: number) => (right: boolean) => [left, right] as const
export const sample: readonly [number, boolean] = [1, true] as const;
