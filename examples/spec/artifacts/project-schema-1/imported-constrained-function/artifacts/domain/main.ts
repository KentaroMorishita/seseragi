import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source111 = { file: "project/imported-constrained-function/src/domain.ssrg", line: 7, column: 3, uri: "seseragi://fixture/imported-constrained-function::domain", start: 111, end: 122 };
$ssrg$assertUnicodeVersion("17.0.0")

export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source111, () => ((__ssrg$evidence$0["ready"](value)) as string), false)
