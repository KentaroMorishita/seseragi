import { report as providerReport } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const report = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ((providerReport(value)(__ssrg$evidence$0)) as string)
