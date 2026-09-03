import { Active, type Badge, describe as providerDescribe } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const active = (value: undefined) => Active
export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ((providerDescribe(value)(__ssrg$evidence$0)) as string)
