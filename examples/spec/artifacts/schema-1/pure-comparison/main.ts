import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const isZero = (value: number) => ((value) as number) === 0
export const sameBool = (left: boolean) => (right: boolean) => ((left) as boolean) === right
export const sameString = (left: string) => (right: string) => ((left) as string) !== right
