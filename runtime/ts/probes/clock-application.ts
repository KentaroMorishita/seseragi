import type {
  ClockEnvironment,
  Duration,
  Instant,
} from "@seseragi/runtime/clock"
import { now, sleep } from "@seseragi/runtime/clock"
import { type Effect, effectFunctor, flatMap } from "@seseragi/runtime/effect"

/** Application code depends on the Clock capability, never a provider identity. */
export function observeThenSleep(
  duration: Duration
): Effect<ClockEnvironment, never, Instant> {
  return flatMap(now(), (instant) =>
    effectFunctor.map(() => instant)(sleep(duration))
  )
}
