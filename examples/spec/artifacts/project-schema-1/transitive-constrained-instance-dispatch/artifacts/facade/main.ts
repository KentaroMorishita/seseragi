import { report as providerReport } from "./provider.js"

export const report = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => providerReport(value)(__ssrg$evidence$0)
