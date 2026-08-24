#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeEffectOperation {
    pub(crate) core_name: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

macro_rules! surface_operation {
    ($canonical:literal, $feature:literal, $local:literal, $module:literal, $export:literal) => {
        RuntimeEffectOperation {
            core_name: $canonical,
            runtime_feature: $feature,
            local_name: $local,
            module: $module,
            export_name: $export,
            source_map_name: $export,
        }
    };
}

const RUNTIME_EFFECT_OPERATIONS: &[RuntimeEffectOperation] = &[
    RuntimeEffectOperation {
        core_name: "stdin.readLine",
        runtime_feature: "effect.stdin.readLine",
        local_name: "_ssrg_stdin_readLine",
        module: "@seseragi/runtime/stdin",
        export_name: "readLine",
        source_map_name: "readLine",
    },
    RuntimeEffectOperation {
        core_name: "console.print",
        runtime_feature: "effect.console.print",
        local_name: "_ssrg_console_print",
        module: "@seseragi/runtime/console",
        export_name: "print",
        source_map_name: "print",
    },
    RuntimeEffectOperation {
        core_name: "console.println",
        runtime_feature: "effect.console.println",
        local_name: "_ssrg_console_println",
        module: "@seseragi/runtime/console",
        export_name: "println",
        source_map_name: "println",
    },
    RuntimeEffectOperation {
        core_name: "effect.succeed",
        runtime_feature: "effect.core.succeed",
        local_name: "_ssrg_effect_succeed",
        module: "@seseragi/runtime/effect",
        export_name: "succeed",
        source_map_name: "succeed",
    },
    RuntimeEffectOperation {
        core_name: "effect.fail",
        runtime_feature: "effect.core.fail",
        local_name: "_ssrg_effect_fail",
        module: "@seseragi/runtime/effect",
        export_name: "fail",
        source_map_name: "fail",
    },
    RuntimeEffectOperation {
        core_name: "effect.mapError",
        runtime_feature: "effect.core.mapError",
        local_name: "_ssrg_effect_mapError",
        module: "@seseragi/runtime/effect",
        export_name: "mapError",
        source_map_name: "mapError",
    },
    RuntimeEffectOperation {
        core_name: "effect.fromEither",
        runtime_feature: "effect.core.fromEither",
        local_name: "_ssrg_effect_fromEither",
        module: "@seseragi/runtime/effect",
        export_name: "fromEither",
        source_map_name: "fromEither",
    },
    RuntimeEffectOperation {
        core_name: "effect.flatMap",
        runtime_feature: "effect.core.flatMap",
        local_name: "_ssrg_effect_flatMap",
        module: "@seseragi/runtime/effect",
        export_name: "flatMap",
        source_map_name: "flatMap",
    },
    surface_operation!(
        "std/effect::succeed",
        "effect.core.succeed",
        "_ssrg_effect_succeed",
        "@seseragi/runtime/effect",
        "succeed"
    ),
    surface_operation!(
        "std/effect::fail",
        "effect.core.fail",
        "_ssrg_effect_fail",
        "@seseragi/runtime/effect",
        "fail"
    ),
    surface_operation!(
        "std/effect::defer",
        "effect.core.defer",
        "_ssrg_effect_defer",
        "@seseragi/runtime/effect",
        "defer"
    ),
    surface_operation!(
        "std/effect::mapError",
        "effect.core.mapError",
        "_ssrg_effect_mapError",
        "@seseragi/runtime/effect",
        "mapError"
    ),
    surface_operation!(
        "std/effect::recover",
        "effect.core.recover",
        "_ssrg_effect_recover",
        "@seseragi/runtime/effect",
        "recover"
    ),
    surface_operation!(
        "std/effect::provide",
        "effect.core.provide",
        "_ssrg_effect_provide",
        "@seseragi/runtime/effect",
        "provide"
    ),
    surface_operation!(
        "std/effect::service",
        "effect.core.service",
        "_ssrg_effect_service",
        "@seseragi/runtime/effect",
        "service"
    ),
    surface_operation!(
        "std/effect::provideSome",
        "effect.core.provide-some",
        "_ssrg_effect_provideSome",
        "@seseragi/runtime/effect",
        "provideSome"
    ),
    surface_operation!(
        "std/effect::attempt",
        "effect.core.attempt",
        "_ssrg_effect_attempt",
        "@seseragi/runtime/effect",
        "attempt"
    ),
    surface_operation!(
        "std/effect::fromEither",
        "effect.core.fromEither",
        "_ssrg_effect_fromEither",
        "@seseragi/runtime/effect",
        "fromEither"
    ),
    surface_operation!(
        "std/effect::fromMaybe",
        "effect.core.from-maybe",
        "_ssrg_effect_fromMaybe",
        "@seseragi/runtime/effect",
        "fromMaybe"
    ),
    surface_operation!(
        "std/effect::acquireRelease",
        "effect.resource.acquire-release",
        "_ssrg_effect_acquireRelease",
        "@seseragi/runtime/effect",
        "acquireRelease"
    ),
    surface_operation!(
        "std/effect::scoped",
        "effect.resource.scoped",
        "_ssrg_effect_scoped",
        "@seseragi/runtime/effect",
        "scoped"
    ),
    surface_operation!(
        "std/effect::ScheduleStop",
        "effect.schedule.stop",
        "_ssrg_effect_ScheduleStop",
        "@seseragi/runtime/effect",
        "ScheduleStop"
    ),
    surface_operation!(
        "std/effect::ScheduleContinue",
        "effect.schedule.continue",
        "_ssrg_effect_ScheduleContinue",
        "@seseragi/runtime/effect",
        "ScheduleContinue"
    ),
    surface_operation!(
        "std/effect::NegativeRecurrences",
        "effect.schedule.negative-recurrences",
        "_ssrg_effect_NegativeRecurrences",
        "@seseragi/runtime/effect",
        "NegativeRecurrences"
    ),
    surface_operation!(
        "std/effect::schedule",
        "effect.schedule.custom",
        "_ssrg_effect_schedule",
        "@seseragi/runtime/effect",
        "schedule"
    ),
    surface_operation!(
        "std/effect::recurs",
        "effect.schedule.recurs",
        "_ssrg_effect_recurs",
        "@seseragi/runtime/effect",
        "recurs"
    ),
    surface_operation!(
        "std/effect::spaced",
        "effect.schedule.spaced",
        "_ssrg_effect_spaced",
        "@seseragi/runtime/effect",
        "spaced"
    ),
    surface_operation!(
        "std/effect::whileInput",
        "effect.schedule.while-input",
        "_ssrg_effect_whileInput",
        "@seseragi/runtime/effect",
        "whileInput"
    ),
    surface_operation!(
        "std/effect::retry",
        "effect.temporal.retry",
        "_ssrg_effect_retry",
        "@seseragi/runtime/effect",
        "retry"
    ),
    surface_operation!(
        "std/effect::repeat",
        "effect.temporal.repeat",
        "_ssrg_effect_repeat",
        "@seseragi/runtime/effect",
        "repeat"
    ),
    surface_operation!(
        "std/effect::timeout",
        "effect.temporal.timeout",
        "_ssrg_effect_timeout",
        "@seseragi/runtime/effect",
        "timeout"
    ),
    surface_operation!(
        "std/effect::timeoutFail",
        "effect.temporal.timeout-fail",
        "_ssrg_effect_timeoutFail",
        "@seseragi/runtime/effect",
        "timeoutFail"
    ),
    surface_operation!(
        "std/effect::FiberSucceeded",
        "effect.fiber.exit.succeeded",
        "_ssrg_effect_FiberSucceeded",
        "@seseragi/runtime/effect",
        "FiberSucceeded"
    ),
    surface_operation!(
        "std/effect::FiberFailed",
        "effect.fiber.exit.failed",
        "_ssrg_effect_FiberFailed",
        "@seseragi/runtime/effect",
        "FiberFailed"
    ),
    surface_operation!(
        "std/effect::FiberCancelled",
        "effect.fiber.exit.cancelled",
        "_ssrg_effect_FiberCancelled",
        "@seseragi/runtime/effect",
        "FiberCancelled"
    ),
    surface_operation!(
        "std/effect::NonPositiveParallelism",
        "effect.parallelism.non-positive",
        "_ssrg_effect_NonPositiveParallelism",
        "@seseragi/runtime/effect",
        "NonPositiveParallelism"
    ),
    surface_operation!(
        "std/effect::fork",
        "effect.fiber.fork",
        "_ssrg_effect_fork",
        "@seseragi/runtime/effect",
        "fork"
    ),
    surface_operation!(
        "std/effect::await",
        "effect.fiber.await",
        "_ssrg_effect_await",
        "@seseragi/runtime/effect",
        "awaitFiber"
    ),
    surface_operation!(
        "std/effect::poll",
        "effect.fiber.poll",
        "_ssrg_effect_poll",
        "@seseragi/runtime/effect",
        "poll"
    ),
    surface_operation!(
        "std/effect::join",
        "effect.fiber.join",
        "_ssrg_effect_join",
        "@seseragi/runtime/effect",
        "join"
    ),
    surface_operation!(
        "std/effect::interrupt",
        "effect.fiber.interrupt",
        "_ssrg_effect_interrupt",
        "@seseragi/runtime/effect",
        "interrupt"
    ),
    surface_operation!(
        "std/effect::yieldNow",
        "effect.scheduler.yield",
        "_ssrg_effect_yieldNow",
        "@seseragi/runtime/effect",
        "yieldNow"
    ),
    surface_operation!(
        "std/effect::race",
        "effect.fiber.race",
        "_ssrg_effect_race",
        "@seseragi/runtime/effect",
        "race"
    ),
    surface_operation!(
        "std/effect::parallelism",
        "effect.parallelism.bounded",
        "_ssrg_effect_parallelism",
        "@seseragi/runtime/effect",
        "parallelism"
    ),
    surface_operation!(
        "std/effect::unboundedParallelism",
        "effect.parallelism.unbounded",
        "_ssrg_effect_unboundedParallelism",
        "@seseragi/runtime/effect",
        "unboundedParallelism"
    ),
    surface_operation!(
        "std/effect::forEachParallel",
        "effect.parallelism.for-each",
        "_ssrg_effect_forEachParallel",
        "@seseragi/runtime/effect",
        "forEachParallel"
    ),
    surface_operation!(
        "std/effect::traverseParallel",
        "effect.parallelism.traverse",
        "_ssrg_effect_traverseParallel",
        "@seseragi/runtime/effect",
        "traverseParallel"
    ),
    surface_operation!(
        "std/deferred::make",
        "effect.deferred.make",
        "_ssrg_deferred_make",
        "@seseragi/runtime/deferred",
        "make"
    ),
    surface_operation!(
        "std/deferred::await",
        "effect.deferred.await",
        "_ssrg_deferred_await",
        "@seseragi/runtime/deferred",
        "awaitDeferred"
    ),
    surface_operation!(
        "std/deferred::poll",
        "effect.deferred.poll",
        "_ssrg_deferred_poll",
        "@seseragi/runtime/deferred",
        "poll"
    ),
    surface_operation!(
        "std/deferred::complete",
        "effect.deferred.complete",
        "_ssrg_deferred_complete",
        "@seseragi/runtime/deferred",
        "complete"
    ),
    surface_operation!(
        "std/deferred::succeed",
        "effect.deferred.succeed",
        "_ssrg_deferred_succeed",
        "@seseragi/runtime/deferred",
        "succeed"
    ),
    surface_operation!(
        "std/deferred::fail",
        "effect.deferred.fail",
        "_ssrg_deferred_fail",
        "@seseragi/runtime/deferred",
        "fail"
    ),
    surface_operation!(
        "std/queue::NonPositiveCapacity",
        "effect.queue.non-positive-capacity",
        "_ssrg_queue_NonPositiveCapacity",
        "@seseragi/runtime/queue",
        "NonPositiveCapacity"
    ),
    surface_operation!(
        "std/queue::QueueClosed",
        "effect.queue.closed",
        "_ssrg_queue_QueueClosed",
        "@seseragi/runtime/queue",
        "QueueClosed"
    ),
    surface_operation!(
        "std/queue::bounded",
        "effect.queue.bounded",
        "_ssrg_queue_bounded",
        "@seseragi/runtime/queue",
        "bounded"
    ),
    surface_operation!(
        "std/queue::unbounded",
        "effect.queue.unbounded",
        "_ssrg_queue_unbounded",
        "@seseragi/runtime/queue",
        "unbounded"
    ),
    surface_operation!(
        "std/queue::offer",
        "effect.queue.offer",
        "_ssrg_queue_offer",
        "@seseragi/runtime/queue",
        "offer"
    ),
    surface_operation!(
        "std/queue::take",
        "effect.queue.take",
        "_ssrg_queue_take",
        "@seseragi/runtime/queue",
        "take"
    ),
    surface_operation!(
        "std/queue::tryOffer",
        "effect.queue.try-offer",
        "_ssrg_queue_tryOffer",
        "@seseragi/runtime/queue",
        "tryOffer"
    ),
    surface_operation!(
        "std/queue::tryTake",
        "effect.queue.try-take",
        "_ssrg_queue_tryTake",
        "@seseragi/runtime/queue",
        "tryTake"
    ),
    surface_operation!(
        "std/queue::size",
        "effect.queue.size",
        "_ssrg_queue_size",
        "@seseragi/runtime/queue",
        "size"
    ),
    surface_operation!(
        "std/queue::close",
        "effect.queue.close",
        "_ssrg_queue_close",
        "@seseragi/runtime/queue",
        "close"
    ),
    surface_operation!(
        "std/semaphore::NonPositivePermits",
        "effect.semaphore.non-positive-permits",
        "_ssrg_semaphore_NonPositivePermits",
        "@seseragi/runtime/semaphore",
        "NonPositivePermits"
    ),
    surface_operation!(
        "std/semaphore::make",
        "effect.semaphore.make",
        "_ssrg_semaphore_make",
        "@seseragi/runtime/semaphore",
        "make"
    ),
    surface_operation!(
        "std/semaphore::acquire",
        "effect.semaphore.acquire",
        "_ssrg_semaphore_acquire",
        "@seseragi/runtime/semaphore",
        "acquire"
    ),
    surface_operation!(
        "std/semaphore::release",
        "effect.semaphore.release",
        "_ssrg_semaphore_release",
        "@seseragi/runtime/semaphore",
        "release"
    ),
    surface_operation!(
        "std/semaphore::withPermit",
        "effect.semaphore.with-permit",
        "_ssrg_semaphore_withPermit",
        "@seseragi/runtime/semaphore",
        "withPermit"
    ),
    surface_operation!(
        "std/semaphore::available",
        "effect.semaphore.available",
        "_ssrg_semaphore_available",
        "@seseragi/runtime/semaphore",
        "available"
    ),
    surface_operation!(
        "std/ref::make",
        "effect.ref.make",
        "_ssrg_ref_make",
        "@seseragi/runtime/ref",
        "make"
    ),
    surface_operation!(
        "std/ref::get",
        "effect.ref.get",
        "_ssrg_ref_get",
        "@seseragi/runtime/ref",
        "get"
    ),
    surface_operation!(
        "std/ref::set",
        "effect.ref.set",
        "_ssrg_ref_set",
        "@seseragi/runtime/ref",
        "set"
    ),
    surface_operation!(
        "std/ref::update",
        "effect.ref.update",
        "_ssrg_ref_update",
        "@seseragi/runtime/ref",
        "update"
    ),
    surface_operation!(
        "std/ref::modify",
        "effect.ref.modify",
        "_ssrg_ref_modify",
        "@seseragi/runtime/ref",
        "modify"
    ),
    surface_operation!(
        "std/time::NegativeDuration",
        "time.duration.negative",
        "_ssrg_time_NegativeDuration",
        "@seseragi/runtime/clock",
        "NegativeDuration"
    ),
    surface_operation!(
        "std/time::DurationOutsideRange",
        "time.duration.outside-range",
        "_ssrg_time_DurationOutsideRange",
        "@seseragi/runtime/clock",
        "DurationOutsideRange"
    ),
    surface_operation!(
        "std/time::zeroDuration",
        "time.duration.zero",
        "_ssrg_time_zeroDuration",
        "@seseragi/runtime/clock",
        "zeroDuration"
    ),
    surface_operation!(
        "std/time::nanoseconds",
        "time.duration.nanoseconds",
        "_ssrg_time_nanoseconds",
        "@seseragi/runtime/clock",
        "nanoseconds"
    ),
    surface_operation!(
        "std/time::milliseconds",
        "time.duration.milliseconds",
        "_ssrg_time_milliseconds",
        "@seseragi/runtime/clock",
        "milliseconds"
    ),
    surface_operation!(
        "std/time::seconds",
        "time.duration.seconds",
        "_ssrg_time_seconds",
        "@seseragi/runtime/clock",
        "seconds"
    ),
    surface_operation!(
        "std/time::minutes",
        "time.duration.minutes",
        "_ssrg_time_minutes",
        "@seseragi/runtime/clock",
        "minutes"
    ),
    surface_operation!(
        "std/time::hours",
        "time.duration.hours",
        "_ssrg_time_hours",
        "@seseragi/runtime/clock",
        "hours"
    ),
    surface_operation!(
        "std/time::toNanoseconds",
        "time.duration.to-nanoseconds",
        "_ssrg_time_toNanoseconds",
        "@seseragi/runtime/clock",
        "toNanoseconds"
    ),
    surface_operation!(
        "std/time::addDuration",
        "time.duration.add",
        "_ssrg_time_addDuration",
        "@seseragi/runtime/clock",
        "addDuration"
    ),
];

pub(crate) fn runtime_effect_operation(core_name: &str) -> Option<RuntimeEffectOperation> {
    RUNTIME_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.core_name == core_name)
}

pub(crate) fn runtime_effect_operation_for_feature(
    feature: &str,
) -> Option<RuntimeEffectOperation> {
    RUNTIME_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_console_println_runtime_abi() {
        let operation = runtime_effect_operation("console.println").unwrap();

        assert_eq!(operation.runtime_feature, "effect.console.println");
        assert_eq!(operation.module, "@seseragi/runtime/console");
        assert_eq!(operation.export_name, "println");
    }

    #[test]
    fn rejects_unknown_core_effect_operation() {
        assert!(runtime_effect_operation("stdin.readChunk").is_none());
    }

    #[test]
    fn resolves_console_print_runtime_abi() {
        let operation = runtime_effect_operation("console.print").unwrap();

        assert_eq!(operation.runtime_feature, "effect.console.print");
        assert_eq!(operation.export_name, "print");
    }

    #[test]
    fn resolves_cold_stdin_read_line_runtime_abi() {
        let operation = runtime_effect_operation("stdin.readLine").unwrap();

        assert_eq!(operation.runtime_feature, "effect.stdin.readLine");
        assert_eq!(operation.module, "@seseragi/runtime/stdin");
    }

    #[test]
    fn resolves_effect_composition_runtime_abi() {
        let operation = runtime_effect_operation("effect.flatMap").unwrap();

        assert_eq!(operation.runtime_feature, "effect.core.flatMap");
        assert_eq!(operation.export_name, "flatMap");
    }

    #[test]
    fn resolves_typed_failure_runtime_abi() {
        let operation = runtime_effect_operation("effect.fail").unwrap();

        assert_eq!(operation.runtime_feature, "effect.core.fail");
        assert_eq!(operation.export_name, "fail");
    }

    #[test]
    fn resolves_failure_mapping_runtime_abi() {
        let operation = runtime_effect_operation("effect.mapError").unwrap();

        assert_eq!(operation.runtime_feature, "effect.core.mapError");
        assert_eq!(operation.export_name, "mapError");
    }

    #[test]
    fn resolves_either_conversion_runtime_abi() {
        let operation = runtime_effect_operation("effect.fromEither").unwrap();

        assert_eq!(operation.runtime_feature, "effect.core.fromEither");
        assert_eq!(operation.module, "@seseragi/runtime/effect");
        assert_eq!(operation.export_name, "fromEither");
    }

    #[test]
    fn resolves_standard_effect_ref_and_duration_surfaces() {
        for (canonical, module) in [
            ("std/effect::defer", "@seseragi/runtime/effect"),
            ("std/effect::attempt", "@seseragi/runtime/effect"),
            ("std/effect::acquireRelease", "@seseragi/runtime/effect"),
            ("std/effect::scoped", "@seseragi/runtime/effect"),
            ("std/effect::retry", "@seseragi/runtime/effect"),
            ("std/effect::timeout", "@seseragi/runtime/effect"),
            ("std/ref::make", "@seseragi/runtime/ref"),
            ("std/ref::modify", "@seseragi/runtime/ref"),
            ("std/time::zeroDuration", "@seseragi/runtime/clock"),
            ("std/time::addDuration", "@seseragi/runtime/clock"),
        ] {
            assert_eq!(
                runtime_effect_operation(canonical)
                    .unwrap_or_else(|| panic!("missing runtime mapping for {canonical}"))
                    .module,
                module
            );
        }
    }
}
