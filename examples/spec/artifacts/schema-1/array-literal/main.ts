import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const choose = (enabled: boolean) => enabled ? [1, 2, 3] : [] as ReadonlyArray<number>
export const matrix = (_unit: undefined) => [[1, 2], [] as ReadonlyArray<number>]
