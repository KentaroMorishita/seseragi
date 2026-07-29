import { type Box, keepMaybe, keepEither } from "./domain.js"

export const useMaybe = (value: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, number] }) => keepMaybe(value)
export const useEither = (value: (argument: number) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [number, number] }) => keepEither(value)
export const useBox = (value: (argument: number) => Box<readonly [string, number]>) => value
