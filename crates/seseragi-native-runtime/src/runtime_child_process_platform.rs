fn native_child_status(status: std::process::ExitStatus) -> V {
    if let Some(code) = status.code() {
        return V::adt(
            "std/child-process::ChildExited",
            V::Int(if cfg!(windows) {
                i64::from(code as u32)
            } else {
                i64::from(code).max(0)
            }),
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        let signal = status.signal().unwrap_or(0);
        let portable = match signal {
            1 => Some("std/process::Hangup"),
            2 => Some("std/process::Interrupt"),
            3 => Some("std/process::Quit"),
            15 => Some("std/process::Terminate"),
            n if n == if cfg!(target_os = "macos") { 30 } else { 10 } => Some("std/process::User1"),
            n if n == if cfg!(target_os = "macos") { 31 } else { 12 } => Some("std/process::User2"),
            _ => None,
        };
        if let Some(tag) = portable {
            return V::adt("std/child-process::ChildSignaled", V::adt(tag, V::Unit));
        }
        let names = if cfg!(any(
            target_vendor = "apple",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly"
        )) {
            [
                "SIGHUP",
                "SIGINT",
                "SIGQUIT",
                "SIGILL",
                "SIGTRAP",
                "SIGABRT",
                "SIGEMT",
                "SIGFPE",
                "SIGKILL",
                "SIGBUS",
                "SIGSEGV",
                "SIGSYS",
                "SIGPIPE",
                "SIGALRM",
                "SIGTERM",
                "SIGURG",
                "SIGSTOP",
                "SIGTSTP",
                "SIGCONT",
                "SIGCHLD",
                "SIGTTIN",
                "SIGTTOU",
                "SIGIO",
                "SIGXCPU",
                "SIGXFSZ",
                "SIGVTALRM",
                "SIGPROF",
                "SIGWINCH",
                "SIGINFO",
                "SIGUSR1",
                "SIGUSR2",
            ]
        } else {
            [
                "SIGHUP",
                "SIGINT",
                "SIGQUIT",
                "SIGILL",
                "SIGTRAP",
                "SIGABRT",
                "SIGBUS",
                "SIGFPE",
                "SIGKILL",
                "SIGUSR1",
                "SIGSEGV",
                "SIGUSR2",
                "SIGPIPE",
                "SIGALRM",
                "SIGTERM",
                "SIGSTKFLT",
                "SIGCHLD",
                "SIGCONT",
                "SIGSTOP",
                "SIGTSTP",
                "SIGTTIN",
                "SIGTTOU",
                "SIGURG",
                "SIGXCPU",
                "SIGXFSZ",
                "SIGVTALRM",
                "SIGPROF",
                "SIGWINCH",
                "SIGIO",
                "SIGPWR",
                "SIGSYS",
            ]
        };
        let host = usize::try_from(signal - 1)
            .ok()
            .and_then(|index| names.get(index))
            .copied()
            .unwrap_or("host terminated");
        return V::adt("std/child-process::ChildHostTerminated", V::text(host));
    }
    #[cfg(not(unix))]
    V::adt(
        "std/child-process::ChildHostTerminated",
        V::text("host terminated"),
    )
}
#[cfg(unix)]
fn native_child_read<P: std::io::Read + std::os::fd::AsRawFd>(
    pipe: &mut P,
    bytes: &mut [u8],
) -> std::io::Result<Option<usize>> {
    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }
    #[cfg(any(target_os = "linux", target_os = "illumos", target_os = "solaris"))]
    type PollCount = usize;
    #[cfg(not(any(target_os = "linux", target_os = "illumos", target_os = "solaris")))]
    type PollCount = u32;
    unsafe extern "C" {
        fn poll(fds: *mut PollFd, count: PollCount, timeout: i32) -> i32;
    }
    let mut fd = PollFd {
        fd: pipe.as_raw_fd(),
        events: 1,
        revents: 0,
    };
    let ready = unsafe { poll(&mut fd, 1, 0) };
    if ready < 0 {
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::Interrupted {
            return Ok(None);
        }
        return Err(error);
    }
    if ready == 0 {
        return Ok(None);
    }
    match pipe.read(bytes) {
        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => Ok(None),
        result => result.map(Some),
    }
}
#[cfg(windows)]
fn native_child_read<P: std::io::Read + std::os::windows::io::AsRawHandle>(
    pipe: &mut P,
    bytes: &mut [u8],
) -> std::io::Result<Option<usize>> {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn PeekNamedPipe(
            handle: *mut c_void,
            buffer: *mut c_void,
            size: u32,
            read: *mut u32,
            available: *mut u32,
            remaining: *mut u32,
        ) -> i32;
    }
    let mut available = 0;
    if unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut available,
            std::ptr::null_mut(),
        )
    } == 0
    {
        let error = std::io::Error::last_os_error();
        return if error.raw_os_error() == Some(109) {
            Ok(Some(0))
        } else {
            Err(error)
        };
    }
    if available == 0 {
        return Ok(None);
    }
    let limit = bytes.len().min(available as usize);
    pipe.read(&mut bytes[..limit]).map(Some)
}
#[cfg(not(any(unix, windows)))]
fn native_child_read<P: std::io::Read>(_: &mut P, _: &mut [u8]) -> std::io::Result<Option<usize>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "child output is unavailable",
    ))
}
