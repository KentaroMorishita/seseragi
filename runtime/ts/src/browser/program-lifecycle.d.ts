export interface BrowserProgram<Exports, Ownership> {
  readonly exports: Exports
  dispose(): void
  ownership(): Ownership
}

export interface ProgramLifetime {
  load<Exports, Ownership>(
    create: () => Promise<BrowserProgram<Exports, Ownership>>,
    closedError: () => Error
  ): Promise<BrowserProgram<Exports, Ownership>>
  close(): Promise<void>
}

export interface ProgramReloadHost {
  addEventListener(
    type: string,
    listener: (event: { preventDefault(): void }) => void,
    options?: { once?: boolean }
  ): void
  readonly location: { reload(): void }
}

export function createProgramLifetime(): ProgramLifetime
export interface ProgramApplicationFrame {
  readonly closing: boolean
  onShutdown(shutdown: () => void | Promise<void>): void
  cleanup(action: () => void | Promise<void>): void
}
export interface ProgramApplication {
  run(body: (frame: ProgramApplicationFrame) => Promise<void>): Promise<void>
  close(): Promise<void>
}
export function createProgramApplication(
  programs: ProgramLifetime
): ProgramApplication
export const loadOwnedProgram: ProgramLifetime["load"]
export const closePrograms: ProgramLifetime["close"]
export const runProgramApplication: ProgramApplication["run"]
export const closeProgramApplication: ProgramApplication["close"]
export function installProgramReload(
  host?: ProgramReloadHost,
  reload?: () => void
): void
