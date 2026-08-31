export interface Node {
  readonly kind: string;
}

export declare function parse(text: string): Node;
export declare function parse(bytes: Uint8Array): Node;
