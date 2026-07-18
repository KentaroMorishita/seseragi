import { println as _ssrg_console_println } from "@seseragi/runtime/console"

declare const __ssrg$brand$Marker: unique symbol;
export type Marker<A> = {
  readonly "label": string;
  readonly "value": A;
  readonly [__ssrg$brand$Marker]: true;
};
export const marker: Marker<ReadonlyArray<string>> = (({ "label": "ready", "value": [] as ReadonlyArray<string> } as const) as unknown as Marker<ReadonlyArray<string>>);
const render = (value: Marker<ReadonlyArray<string>>) => "Explicit generic Struct: " + (value)["label"]
export const main = (_unit: undefined) => _ssrg_console_println(render(marker))
