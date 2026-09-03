import { type Prefix } from "./contract.js"
import "./contract.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

type Score =
  | { readonly tag: "ScoreValue" };
const ScoreValue: Score = { tag: "ScoreValue" } as const;
export const __ssrg$instance$Render$0 = { "render": (prefix: Prefix) => (value: Score) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => "score" } as const;
export const status = (unit: undefined) => "contract ready"
