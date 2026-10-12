interface Hidden { readonly id: string; }
export interface Snapshot { readonly value: Hidden; readonly enabled?: boolean; }
export interface Snapshot { readonly label: string; }
export type Union = { readonly tag: "a"; readonly value: string } | { readonly tag: "b"; readonly value: number };
export type Intersection = { readonly id: string } & { readonly label: string };
export type Recursive = { readonly value: string; readonly next?: Recursive };
export declare class Container<T> {
  readonly value: T;
  constructor(value: T);
  read(this: Container<T>): T;
  replace(value: T): this;
}
