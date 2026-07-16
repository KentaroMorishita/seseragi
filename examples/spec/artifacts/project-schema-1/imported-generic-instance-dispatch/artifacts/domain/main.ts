export const __ssrg$instance$Inspect$0 = <T,>() => ({ "inspect": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => "imported generic dictionary" }) as const;
export const report = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["inspect"](value)
