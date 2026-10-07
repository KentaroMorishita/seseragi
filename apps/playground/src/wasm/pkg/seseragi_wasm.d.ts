/* tslint:disable */
/* eslint-disable */

/**
 * Analyzes every source in an in-memory browser workspace after linking its
 * local imports to the same typed public interfaces used by compilation.
 */
export function analyze_project(request: string): string;

/**
 * Analyzes one source without lowering, code generation, Effect execution,
 * or DOM mounting. The returned occurrence tables back hover and Reference
 * queries while diagnostics remain identical to compile responses.
 */
export function analyze_single_file(source_name: string, module_id: string, source: string): string;

/**
 * Compiles an in-memory browser workspace through the shared project driver.
 */
export function compile_project(request: string): string;

/**
 * Compiles one already-identified source with the same driver used by the
 * native CLI and LSP, returning a versioned JSON envelope for JavaScript.
 */
export function compile_single_file(source_name: string, module_id: string, source: string): string;

export function convert_workspace_bindings(json: string): string;

/**
 * Formats one path selected from the versioned workspace request.
 */
export function format_project_file(request: string, path: string): string;

/**
 * Formats one workspace path with an explicit source-column width.
 */
export function format_project_file_with_options(request: string, path: string, line_width: number): string;

/**
 * Formats one source snapshot with the same formatter used by the native CLI
 * and LSP, returning either the complete canonical source or shared parser
 * diagnostics. Invalid source is never returned as a rewritten document.
 */
export function format_single_file(source_name: string, source: string): string;

/**
 * Formats one source snapshot with an explicit source-column width.
 */
export function format_single_file_with_options(source_name: string, source: string, line_width: number): string;

/**
 * Returns stable metadata for the committed browser artifact.
 *
 * Native CLI and LSP binaries expose their Git commit and dirty state. The
 * browser artifact intentionally omits those mutable fields so a WASM package
 * generated from identical sources stays fresh across worktrees and commits.
 */
export function toolchain_version_json(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly memcmp: (a: number, b: number, c: number) => number;
    readonly analyze_project: (a: number, b: number) => [number, number];
    readonly compile_project: (a: number, b: number) => [number, number];
    readonly format_project_file: (a: number, b: number, c: number, d: number) => [number, number];
    readonly format_project_file_with_options: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly analyze_single_file: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly compile_single_file: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly format_single_file: (a: number, b: number, c: number, d: number) => [number, number];
    readonly format_single_file_with_options: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly toolchain_version_json: () => [number, number];
    readonly convert_workspace_bindings: (a: number, b: number) => [number, number];
    readonly iswspace: (a: number) => number;
    readonly iswalpha: (a: number) => number;
    readonly iswdigit: (a: number) => number;
    readonly strlen: (a: number) => number;
    readonly abort: () => void;
    readonly calloc: (a: number, b: number) => number;
    readonly free: (a: number) => void;
    readonly malloc: (a: number) => number;
    readonly realloc: (a: number, b: number) => number;
    readonly memcpy: (a: number, b: number, c: number) => number;
    readonly strncmp: (a: number, b: number, c: number) => number;
    readonly memset: (a: number, b: number, c: number) => number;
    readonly memmove: (a: number, b: number, c: number) => number;
    readonly memchr: (a: number, b: number, c: number) => number;
    readonly strchr: (a: number, b: number) => number;
    readonly strcmp: (a: number, b: number) => number;
    readonly strncat: (a: number, b: number, c: number) => number;
    readonly strncpy: (a: number, b: number, c: number) => number;
    readonly iswalnum: (a: number) => number;
    readonly iswblank: (a: number) => number;
    readonly iswlower: (a: number) => number;
    readonly towupper: (a: number) => number;
    readonly iswpunct: (a: number) => number;
    readonly iswupper: (a: number) => number;
    readonly towlower: (a: number) => number;
    readonly iswxdigit: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
