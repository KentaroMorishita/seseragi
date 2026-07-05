export interface Config {
  readonly label: string;
  readonly limit?: number;
}

export declare function fetchName(id: bigint): Promise<string>;
export declare function first<T>(values: readonly T[]): T | undefined;
export declare function useConfig(config: Config): void;
