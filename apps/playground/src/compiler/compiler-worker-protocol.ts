import type * as Service from "./wasm-service"

export type CompilerService = typeof Service
export type CompilerMethod = keyof CompilerService
export type CompilerResult<Method extends CompilerMethod> = Awaited<
  ReturnType<CompilerService[Method]>
>
export type CompilerRequest = {
  [Method in CompilerMethod]: {
    readonly id: number
    readonly method: Method
    readonly args: Parameters<CompilerService[Method]>
  }
}[CompilerMethod]
export type CompilerReply =
  | { readonly id: number; readonly ok: true; readonly result: unknown }
  | { readonly id: number; readonly ok: false; readonly error: string }
