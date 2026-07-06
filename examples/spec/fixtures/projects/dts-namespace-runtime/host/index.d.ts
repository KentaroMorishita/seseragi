export namespace Metrics {
  export function count(values: readonly number[]): number;

  export namespace Format {
    export function label(value: number): string;
  }
}
