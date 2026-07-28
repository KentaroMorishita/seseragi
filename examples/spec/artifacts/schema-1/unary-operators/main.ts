import { subtract as _ssrg_int64_subtract } from "@seseragi/runtime/int64"

declare const __ssrg$brand$Snapshot: unique symbol;
export type Snapshot = {
  readonly "negative": bigint;
  readonly "negativeZero": number;
  readonly "inverted": boolean;
  readonly [__ssrg$brand$Snapshot]: true;
};
export const negative: bigint = _ssrg_int64_subtract(0n, 2n);
export const negativeZero: number = -(0.0);
export const inverted: boolean = !(true);
export const minimum: bigint = _ssrg_int64_subtract(0n, 9223372036854775808n);
export const values: ReadonlyArray<bigint> = [_ssrg_int64_subtract(0n, 1n), _ssrg_int64_subtract(0n, 2n), _ssrg_int64_subtract(0n, 3n)];
export const floats: ReadonlyArray<number> = [-(1.0), -(0.0), -(6.022e23)];
export const flags: ReadonlyArray<boolean> = [!(true), !(false)];
export const tuple: readonly [bigint, number, boolean] = [_ssrg_int64_subtract(0n, 4n), -(2.5), !(false)] as const;
export const record: { readonly "inverted": boolean; readonly "negative": bigint } = ({ "negative": _ssrg_int64_subtract(0n, 5n), "inverted": !(true) } as const);
export const snapshot: Snapshot = (({ "negative": _ssrg_int64_subtract(0n, 6n), "negativeZero": -(0.0), "inverted": !(false) } as const) as unknown as Snapshot);
export const negateInt = (value: bigint) => _ssrg_int64_subtract(0n, value)
export const negateFloat = (value: number) => -(value)
export const invert = (value: boolean) => !(value)
