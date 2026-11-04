#![allow(dead_code)]
use seseragi_native_runtime::*;

#[derive(Default)]
struct Host {
    fail: String,
    written: Vec<u8>,
    closes: usize,
}
impl Host {
    fn stage(&self, stage: &str) -> std::io::Result<()> {
        if self.fail.split('+').any(|failure| failure == stage) {
            Err(if stage == "close" {
                std::io::ErrorKind::PermissionDenied
            } else {
                std::io::ErrorKind::AlreadyExists
            }
            .into())
        } else {
            Ok(())
        }
    }
}
impl NativeFileHost for Host {
    type Handle = ();
    fn open(&mut self, _: &str, _: Option<&str>) -> std::io::Result<()> {
        self.stage("open")
    }
    fn read(&mut self, _: &mut (), _: &mut [u8]) -> std::io::Result<usize> {
        self.stage("read")?;
        Ok(0)
    }
    fn write(&mut self, _: &mut (), bytes: &[u8]) -> std::io::Result<usize> {
        self.stage("write")?;
        self.written.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self, _: &mut ()) -> std::io::Result<()> {
        self.stage("flush")
    }
    fn close(&mut self, _: ()) -> std::io::Result<()> {
        self.closes += 1;
        self.stage("close")
    }
}

fn main() {
    let path = native_path("output-雪.txt");
    for api in ["text", "bytes", "read-text", "read-bytes"] {
        let reading = api.starts_with("read-");
        for mode in ["Replace", "Append", "CreateNew"] {
            for content in [String::new(), "A\0雪🙂\n".into(), "x".repeat(65535) + "🙂"] {
                for failure in [
                    "ok",
                    "open",
                    "read",
                    "write",
                    "flush",
                    "close",
                    "write+close",
                    "flush+close",
                ] {
                    let mut host = Host {
                        fail: failure.into(),
                        ..Host::default()
                    };
                    let operation = match api {
                        "text" => NativeFileOperation::WriteText,
                        "bytes" => NativeFileOperation::WriteBytes,
                        "read-text" => NativeFileOperation::ReadText,
                        _ => NativeFileOperation::ReadBytes,
                    };
                    let captures = if reading {
                        vec![path.clone()]
                    } else {
                        vec![
                            V::adt(
                                match mode {
                                    "Replace" => "std/fs::Replace",
                                    "Append" => "std/fs::Append",
                                    _ => "std/fs::CreateNew",
                                },
                                V::Unit,
                            ),
                            if api == "text" {
                                V::text(&content)
                            } else {
                                V::Bytes(Shared::new(content.as_bytes().to_vec()))
                            },
                            path.clone(),
                        ]
                    };
                    let reply = native_run_file_request(
                        &mut host,
                        operation.request(&captures),
                        &std::sync::atomic::AtomicBool::new(false),
                    );
                    let (outcome, _) = operation.outcome(&captures, reply, false);
                    let result = match outcome {
                        Ok(_) => "success".into(),
                        Err(NativeExit::Failure(error)) => native_filesystem_display(
                            if reading && api == "read-text" {
                                "std/fs::FileTextError"
                            } else {
                                "std/fs::FileSystemError"
                            },
                            &error,
                        ),
                        _ => panic!("unexpected defect or cancellation"),
                    };
                    let expected_content =
                        if reading || failure == "open" || failure.contains("write") {
                            &[][..]
                        } else {
                            content.as_bytes()
                        };
                    assert_eq!(host.written, expected_content);
                    assert_eq!(host.closes, usize::from(failure != "open"));
                    println!("{api}/{mode}/{}/{failure}: {result}", content.len());
                }
            }
        }
    }
    drop(path);
    clear_dictionaries();
    assert_eq!(LIVE.with(std::cell::Cell::get), 0);
}
