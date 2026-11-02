mod native_process_signal_host {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};

    // Each three-bit digit stores one pending kind, oldest first. One atomic
    // update preserves arrival order while coalescing repeated kinds.
    static PENDING: AtomicUsize = AtomicUsize::new(0);
    static SHUTDOWN: AtomicBool = AtomicBool::new(false);
    static FIRST: AtomicUsize = AtomicUsize::new(0);
    static HANDLERS: std::sync::Mutex<[(usize, usize, bool); 6]> =
        std::sync::Mutex::new([(0, 0, false); 6]);

    fn number(index: usize) -> Option<i32> {
        match index {
            0 => Some(2),
            1 => Some(15),
            #[cfg(unix)]
            2 => Some(1),
            #[cfg(unix)]
            3 => Some(3),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            4 => Some(10),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            5 => Some(12),
            #[cfg(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "dragonfly"
            ))]
            4 => Some(30),
            #[cfg(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "dragonfly"
            ))]
            5 => Some(31),
            _ => None,
        }
    }
    pub(super) fn supported(index: usize) -> bool {
        cfg!(any(unix, windows)) && number(index).is_some()
    }

    #[cfg(any(unix, windows))]
    unsafe extern "C" {
        fn signal(number: i32, handler: usize) -> usize;
        fn _exit(code: i32) -> !;
    }

    fn receive(index: usize) {
        // Signal handlers touch only lock-free atomics. User callbacks run on
        // the runtime thread after the scheduler observes the pending queue.
        let mut pending = PENDING.load(SeqCst);
        loop {
            let mut remaining = pending;
            let mut shift = 0;
            while remaining != 0 && remaining & 7 != index + 1 {
                shift += 3;
                remaining >>= 3;
            }
            if remaining != 0 {
                break;
            }
            match PENDING.compare_exchange(
                pending,
                pending | ((index + 1) << shift),
                SeqCst,
                SeqCst,
            ) {
                Ok(_) => break,
                Err(current) => pending = current,
            }
        }
        if index < 2
            && SHUTDOWN.load(SeqCst)
            && FIRST
                .compare_exchange(0, index + 1, SeqCst, SeqCst)
                .is_err()
        {
            #[cfg(any(unix, windows))]
            unsafe {
                _exit(if index == 1 { 143 } else { 130 })
            }
        }
    }
    #[cfg(any(unix, windows))]
    extern "C" fn handler(received: i32) {
        if let Some(index) = (0..6).find(|&index| number(index) == Some(received)) {
            #[cfg(windows)]
            unsafe {
                signal(received, handler as *const () as usize);
            }
            receive(index);
        }
    }

    #[cfg(windows)]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
    }
    #[cfg(windows)]
    unsafe extern "system" fn console_handler(event: u32) -> i32 {
        if event == 0 {
            receive(0);
            1
        } else {
            0
        }
    }

    pub(super) struct Handler(usize);
    impl Handler {
        pub(super) fn index(&self) -> usize {
            self.0
        }
        pub(super) fn install(index: usize) -> Result<Self, super::NativeExit> {
            if !supported(index) {
                return Err(super::NativeExit::defects(vec![
                    "native host cannot install the requested process signal".into(),
                ]));
            }
            let mut handlers = HANDLERS.lock().unwrap();
            let slot = &mut handlers[index];
            if slot.0 == 0 {
                #[cfg(any(unix, windows))]
                unsafe {
                    let previous = signal(number(index).unwrap(), handler as *const () as usize);
                    if previous == usize::MAX {
                        return Err(super::NativeExit::defects(vec![
                            "native host could not register a process signal handler".into(),
                        ]));
                    }
                    #[cfg(windows)]
                    if index == 0 {
                        // A process without a console can still observe CRT
                        // signals. Console notifications require a console.
                        slot.2 = SetConsoleCtrlHandler(Some(console_handler), 1) != 0;
                    }
                    slot.1 = previous;
                }
            }
            slot.0 += 1;
            Ok(Self(index))
        }
    }
    impl Drop for Handler {
        fn drop(&mut self) {
            let mut handlers = HANDLERS.lock().unwrap();
            let slot = &mut handlers[self.0];
            slot.0 -= 1;
            if slot.0 == 0 {
                #[cfg(any(unix, windows))]
                unsafe {
                    signal(number(self.0).unwrap(), slot.1);
                    #[cfg(windows)]
                    if self.0 == 0 && slot.2 {
                        SetConsoleCtrlHandler(Some(console_handler), 0);
                    }
                }
            }
        }
    }

    pub(super) fn drain() -> Vec<usize> {
        let mut pending = PENDING.swap(0, SeqCst);
        let mut signals = Vec::new();
        while pending != 0 {
            signals.push((pending & 7) - 1);
            pending >>= 3;
        }
        signals
    }
    pub(super) fn shutdown(enabled: bool) {
        SHUTDOWN.store(false, SeqCst);
        FIRST.store(0, SeqCst);
        SHUTDOWN.store(enabled, SeqCst);
    }
    pub(super) fn first_termination() -> Option<usize> {
        FIRST.load(SeqCst).checked_sub(1)
    }

    #[cfg(test)]
    pub(super) static TEST_ACCESS: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[cfg(all(test, unix))]
    mod tests {
        use super::*;
        extern "C" fn previous_handler(_: i32) {}

        #[test]
        fn last_close_and_partial_registration_failure_restore_the_previous_handler() {
            let _access = TEST_ACCESS.lock().unwrap();
            struct Restore(usize);
            impl Drop for Restore {
                fn drop(&mut self) {
                    unsafe {
                        signal(number(4).unwrap(), self.0);
                    }
                }
            }
            let previous = previous_handler as *const () as usize;
            let _restore = Restore(unsafe { signal(number(4).unwrap(), previous) });
            {
                let first = Handler::install(4).unwrap_or_else(|_| panic!());
                let second = Handler::install(4).unwrap_or_else(|_| panic!());
                assert_eq!(HANDLERS.lock().unwrap()[4].0, 2);
                drop(first);
                assert_eq!(HANDLERS.lock().unwrap()[4].0, 1);
                drop(second);
                assert_eq!(HANDLERS.lock().unwrap()[4].0, 0);
                assert_eq!(unsafe { signal(number(4).unwrap(), previous) }, previous);
            }
            let registered = [4, usize::MAX]
                .into_iter()
                .map(Handler::install)
                .collect::<Result<Vec<_>, _>>();
            assert!(registered.is_err());
            assert_eq!(HANDLERS.lock().unwrap()[4].0, 0);
            assert_eq!(unsafe { signal(number(4).unwrap(), previous) }, previous);
        }
    }
}
