import { report as providerReport } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source121 = { file: "project/transitive-constrained-instance-dispatch/src/facade.ssrg", line: 5, column: 3, uri: "seseragi://fixture/transitive-constrained-instance-dispatch::facade", start: 121, end: 141 };
$ssrg$assertUnicodeVersion("17.0.0")

export const report = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source121, () => ((providerReport<T>(value)(__ssrg$evidence$0)) as string), false)
