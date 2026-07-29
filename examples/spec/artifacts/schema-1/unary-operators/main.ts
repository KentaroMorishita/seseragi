import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"

declare const __ssrg$brand$Snapshot: unique symbol;
export type Snapshot = {
  readonly "negative": number;
  readonly "negativeZero": number;
  readonly "inverted": boolean;
  readonly [__ssrg$brand$Snapshot]: true;
};
export const negateInt = (value: number) => _ssrg_int_subtract(0, value)
export const negateFloat = (value: number) => -(value)
export const invert = (value: boolean) => !(value)
export const negative: number = _ssrg_int_subtract(0, 2);
export const negativeZero: number = -(0.0);
export const inverted: boolean = !(true);
export const minimum: number = _ssrg_int_subtract(0, 9007199254740991);
export const values: ReadonlyArray<number> = [_ssrg_int_subtract(0, 1), _ssrg_int_subtract(0, 2), _ssrg_int_subtract(0, 3)];
export const floats: ReadonlyArray<number> = [-(1.0), -(0.0), -(6.022e23)];
export const flags: ReadonlyArray<boolean> = [!(true), !(false)];
export const tuple: readonly [number, number, boolean] = [_ssrg_int_subtract(0, 4), -(2.5), !(false)] as const;
export const record: { readonly "inverted": boolean; readonly "negative": number } = ({ "negative": _ssrg_int_subtract(0, 5), "inverted": !(true) } as const);
export const snapshot: Snapshot = (({ "negative": _ssrg_int_subtract(0, 6), "negativeZero": -(0.0), "inverted": !(false) } as const) as unknown as Snapshot);
