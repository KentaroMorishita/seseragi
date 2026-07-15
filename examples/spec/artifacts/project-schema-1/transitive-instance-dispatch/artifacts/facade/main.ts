import { Active, type Badge, describe as providerDescribe } from "./provider.js"

export const active = (value: undefined) => Active
export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => providerDescribe(value)(__ssrg$evidence$0)
