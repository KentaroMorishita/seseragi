#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeEntropyError {
    Unavailable,
    ReadFailure,
}

impl NativeEntropyError {
    fn exit(self) -> NativeExit {
        NativeExit::failure(V::adt(
            match self {
                Self::Unavailable => "std/entropy::EntropyUnavailable",
                Self::ReadFailure => "std/entropy::EntropyReadFailure",
            },
            V::Unit,
        ))
    }
}

#[cfg(any(unix, test))]
fn native_entropy_read(
    source: std::io::Result<impl std::io::Read>,
    bytes: &mut [u8],
) -> Result<(), NativeEntropyError> {
    let mut source = source.map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::Unsupported => {
            NativeEntropyError::Unavailable
        }
        _ => NativeEntropyError::ReadFailure,
    })?;
    source
        .read_exact(bytes)
        .map_err(|_| NativeEntropyError::ReadFailure)
}

#[cfg(unix)]
fn native_entropy_fill(bytes: &mut [u8]) -> Result<(), NativeEntropyError> {
    native_entropy_read(std::fs::File::open("/dev/urandom"), bytes)
}

#[cfg(windows)]
fn native_entropy_fill(bytes: &mut [u8]) -> Result<(), NativeEntropyError> {
    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(
            algorithm: *mut std::ffi::c_void,
            buffer: *mut u8,
            length: u32,
            flags: u32,
        ) -> i32;
    }
    // The system RNG takes a null algorithm handle and a writable buffer that
    // stays alive for the call. EntropySize bounds its length to at most 1 MiB.
    let status = unsafe {
        BCryptGenRandom(
            std::ptr::null_mut(),
            bytes.as_mut_ptr(),
            u32::try_from(bytes.len()).expect("EntropySize fits in u32"),
            2,
        )
    };
    if status >= 0 {
        Ok(())
    } else {
        Err(NativeEntropyError::ReadFailure)
    }
}

#[cfg(not(any(unix, windows)))]
fn native_entropy_fill(_bytes: &mut [u8]) -> Result<(), NativeEntropyError> {
    Err(NativeEntropyError::Unavailable)
}

fn native_entropy_secure_bytes(
    size: &V,
    fill: impl FnOnce(&mut [u8]) -> Result<(), NativeEntropyError>,
) -> NativeOutcome {
    assert!(
        size.tagged("std/entropy::EntropySize"),
        "Rust ABI: expected EntropySize"
    );
    let V::Int(size) = size.payload() else {
        panic!("Rust ABI: invalid EntropySize")
    };
    assert!(
        (1..=1_048_576).contains(&size),
        "Entropy byte size is outside the standard range"
    );
    let mut bytes = vec![0; size as usize];
    fill(&mut bytes).map_err(NativeEntropyError::exit)?;
    Ok(V::Bytes(Shared::new(bytes)))
}

fn native_entropy_run(captures: &[V], _context: &mut NativeContext) -> NativeOutcome {
    let [size] = captures else {
        panic!("Rust ABI: expected EntropySize argument")
    };
    native_entropy_secure_bytes(size, native_entropy_fill)
}

pub fn native_entropy_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/entropy::entropySize", [V::Int(size)]) => {
            let size = checked_int_value(*size);
            if size <= 0 {
                native_left(V::adt("std/entropy::NonPositiveEntropySize", V::Int(size)))
            } else if size > 1_048_576 {
                native_left(V::adt("std/entropy::EntropySizeTooLarge", V::Int(size)))
            } else {
                native_right(V::adt("std/entropy::EntropySize", V::Int(size)))
            }
        }
        ("std/entropy::secureBytes", [_]) => V::leaf(arguments, native_entropy_run),
        _ => panic!("Rust ABI: unadmitted Entropy operation"),
    }
}

#[cfg(test)]
mod entropy_tests {
    use super::*;
    use std::io::{self, Read};

    thread_local! {
        static READS: Cell<u8> = const { Cell::new(0) };
    }

    fn size(value: i64) -> V {
        let result = native_entropy_operation("std/entropy::entropySize", vec![V::Int(value)]);
        assert!(result.tagged("std/prelude::Right"));
        result.payload()
    }

    fn failure(outcome: NativeOutcome, tag: &str) {
        let Err(NativeExit::Failure(error)) = outcome else {
            panic!("expected typed entropy failure")
        };
        assert!(error.value.tagged(tag));
        assert!(matches!(error.value.payload(), V::Unit));
    }

    #[test]
    fn configuration_preserves_error_kinds_values_and_inclusive_limits() {
        for (value, tag) in [
            (-9_007_199_254_740_991, "std/entropy::NonPositiveEntropySize"),
            (-1, "std/entropy::NonPositiveEntropySize"),
            (0, "std/entropy::NonPositiveEntropySize"),
            (1_048_577, "std/entropy::EntropySizeTooLarge"),
            (9_007_199_254_740_991, "std/entropy::EntropySizeTooLarge"),
        ] {
            let result =
                native_entropy_operation("std/entropy::entropySize", vec![V::Int(value)]);
            assert!(result.tagged("std/prelude::Left"));
            assert!(result.payload().tagged(tag));
            assert_eq!(result.payload().payload().render(), value.to_string());
        }
        for value in [1, 65_536, 65_537, 1_048_576] {
            let result = size(value);
            assert!(result.tagged("std/entropy::EntropySize"));
            assert_eq!(result.payload().render(), value.to_string());
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    #[cfg(any(unix, windows))]
    fn os_source_fills_the_entire_request_on_each_cold_execution() {
        let mut context = NativeContext::default();
        for length in [1, 32, 65_536, 65_537, 1_048_576] {
            let plan = native_entropy_operation("std/entropy::secureBytes", vec![size(length)]);
            for _ in 0..2 {
                let value = context.root(plan.clone()).unwrap_or_else(|_| panic!());
                assert_eq!(native_bytes(&value).len(), length as usize);
            }
        }
        assert!(context.random.borrow().state.is_none());
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn unavailable_and_partial_read_failures_never_publish_bytes() {
        let size = size(4);
        failure(
            native_entropy_secure_bytes(&size, |bytes| {
                native_entropy_read(
                    Err::<io::Empty, _>(io::Error::from(io::ErrorKind::NotFound)),
                    bytes,
                )
            }),
            "std/entropy::EntropyUnavailable",
        );
        failure(
            native_entropy_secure_bytes(&size, |bytes| {
                native_entropy_read(
                    Err::<io::Empty, _>(io::Error::from(io::ErrorKind::PermissionDenied)),
                    bytes,
                )
            }),
            "std/entropy::EntropyReadFailure",
        );
        failure(
            native_entropy_secure_bytes(&size, |bytes| {
                native_entropy_read(Ok(io::Cursor::new([17, 29])), bytes)
            }),
            "std/entropy::EntropyReadFailure",
        );
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _bytes: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            }
        }
        failure(
            native_entropy_secure_bytes(&size, |bytes| {
                native_entropy_read(Ok(Failing), bytes)
            }),
            "std/entropy::EntropyReadFailure",
        );
        let result = native_entropy_secure_bytes(&size, |bytes| {
            native_entropy_read(Ok(io::Cursor::new([1, 2, 3, 4])), bytes)
        })
        .unwrap_or_else(|_| panic!());
        assert_eq!(native_bytes(&result), &[1, 2, 3, 4]);
        drop((size, result));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn short_and_interrupted_reads_are_completed_before_success() {
        struct ShortRead(u8);
        impl Read for ShortRead {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                self.0 += 1;
                if self.0 == 1 {
                    return Err(io::Error::from(io::ErrorKind::Interrupted));
                }
                bytes[0] = self.0;
                Ok(1)
            }
        }
        let result = native_entropy_secure_bytes(&size(4), |bytes| {
            native_entropy_read(Ok(ShortRead(0)), bytes)
        })
        .unwrap_or_else(|_| panic!());
        assert_eq!(native_bytes(&result), &[2, 3, 4, 5]);
        drop(result);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn cancelled_plans_skip_the_source_and_reexecution_keeps_previous_bytes_immutable() {
        fn read(captures: &[V], _: &mut NativeContext) -> NativeOutcome {
            native_entropy_secure_bytes(&captures[0], |bytes| {
                READS.with(|reads| {
                    reads.set(reads.get() + 1);
                    bytes.fill(reads.get());
                });
                Ok(())
            })
        }
        READS.with(|reads| reads.set(0));
        let plan = V::leaf(vec![size(4)], read);
        assert_eq!(READS.with(Cell::get), 0);
        let mut context = NativeContext {
            cancelled: true,
            ..NativeContext::default()
        };
        assert!(matches!(context.root(plan.clone()), Err(NativeExit::Cancelled)));
        assert_eq!(READS.with(Cell::get), 0);
        context.cancelled = false;
        let first = context.root(plan.clone()).unwrap_or_else(|_| panic!());
        let second = context.root(plan.clone()).unwrap_or_else(|_| panic!());
        assert_eq!(native_bytes(&first), &[1, 1, 1, 1]);
        assert_eq!(native_bytes(&second), &[2, 2, 2, 2]);
        assert_eq!(READS.with(Cell::get), 2);
        drop((plan, first, second));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
