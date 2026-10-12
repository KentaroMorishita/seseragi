export interface Schema<A> { readonly value: A; }
export declare function make<A>(value: A): Schema<A>;
export declare function read<A>(schema: Schema<A>): A;
export declare function select<A>(fallback: A, schema: Schema<A>): A;
export namespace nested { export function echo<A>(value: A): A; }
