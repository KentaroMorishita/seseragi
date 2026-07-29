import { type Box, keepMaybe, keepEither } from "./domain.js"

export const useMaybe = (value: (argument: bigint) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, bigint] }) => keepMaybe(value)
export const useEither = (value: (argument: bigint) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [bigint, bigint] }) => keepEither(value)
export const useBox = (value: (argument: bigint) => Box<readonly [string, bigint]>) => value
