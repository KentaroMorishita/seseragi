export type {
  Effect,
  EffectFailure,
  EffectResult,
  EffectSuccess,
  Unit,
} from "./effect";
export { fail, run, succeed, unit } from "./effect";

export type { Console, ConsoleEnvironment, ConsoleError } from "./console";
export { liveConsole, println } from "./console";
