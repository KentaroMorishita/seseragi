import { Active, describe as providerDescribe, type Badge } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source183 = { file: "project/transitive-instance-dispatch/src/facade.ssrg", line: 7, column: 3, uri: "seseragi://fixture/transitive-instance-dispatch::facade", start: 183, end: 205 };
$ssrg$assertUnicodeVersion("17.0.0")

export const active = (value: undefined) => Active
export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source183, () => ((providerDescribe<T>(value)(__ssrg$evidence$0)) as string), false)
