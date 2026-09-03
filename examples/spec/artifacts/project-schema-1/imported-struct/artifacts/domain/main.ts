import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Player: unique symbol;
export type Player = {
  readonly "name": string;
  readonly "score": number;
  readonly [__ssrg$brand$Player]: true;
};
