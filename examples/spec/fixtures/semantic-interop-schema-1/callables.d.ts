export declare function identity<T>(value: T): T;
export declare function constrained<T extends string = string>(value: T): T;
export declare function callback(use: (value: string) => number): number;
export declare function asynchronous(value: string): Promise<string>;
export declare function overloaded(value: string): string;
export declare function overloaded(value: number): number;
export namespace tools { function normalize(value: string): string; }
export namespace tools { function count(value: string): number; }
export type HigherRank = <T>(value: T) => T;
export type ThisReceiver = (this: { readonly id: string }, value: string) => string;
