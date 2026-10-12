export type Keys<T> = keyof T;
export type Member<T, K extends keyof T = keyof T> = T[K];
export type Element<T> = T extends Array<infer U> ? U : never;
export type Selected = 1 extends number ? string : boolean;
export type Open<T> = T extends string ? string : number;
export type Mapped = { readonly [K in "id" | "name"]?: string };
export type MappedOpen<T> = { [K in keyof T]: T[K] };
export type Literal = "ready" | "busy" | 42 | true;
export type Template = `status:${"ready" | "busy"}`;
