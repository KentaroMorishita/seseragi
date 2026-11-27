use seseragi_native_runtime::{
    native_http_connect_tcp, native_http_register_connector, NativeHttpConnectError,
    NativeHttpConnector, NativeHttpShutdown, NativeHttpTarget, NativeHttpTransport,
};
use std::io::{self, Cursor, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct RecordingShutdown {
    both: AtomicUsize,
}

impl NativeHttpShutdown for RecordingShutdown {
    fn shutdown_write(&self) -> io::Result<()> {
        Ok(())
    }

    fn shutdown_both(&self) -> io::Result<()> {
        self.both.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct Sink(Arc<Mutex<Vec<u8>>>);

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct FakeTlsProvider {
    sent: Arc<Mutex<Vec<u8>>>,
    shutdown: Arc<RecordingShutdown>,
}

impl NativeHttpConnector for FakeTlsProvider {
    fn connect(
        &self,
        target: &NativeHttpTarget,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<NativeHttpTransport, NativeHttpConnectError> {
        if cancelled.load(Ordering::Acquire) {
            return Err(NativeHttpConnectError::Cancelled);
        }
        if target.host == "bad-cert.example" {
            return Err(NativeHttpConnectError::Tls(
                "certificate verify failed".into(),
            ));
        }
        Ok(NativeHttpTransport {
            reader: Box::new(Cursor::new(
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok".to_vec(),
            )),
            writer: Box::new(Sink(self.sent.clone())),
            shutdown: self.shutdown.clone(),
        })
    }
}

#[test]
fn external_provider_registers_once_and_can_reuse_tcp_helper() {
    let provider = Arc::new(FakeTlsProvider {
        sent: Arc::new(Mutex::new(Vec::new())),
        shutdown: Arc::new(RecordingShutdown {
            both: AtomicUsize::new(0),
        }),
    });
    assert!(native_http_register_connector(provider.clone()).is_ok());
    assert!(native_http_register_connector(provider).is_err());

    let cancelled = Arc::new(AtomicBool::new(true));
    let target = NativeHttpTarget {
        https: true,
        host: "127.0.0.1".into(),
        port: 443,
        authority: "127.0.0.1:443".into(),
        path: "/".into(),
    };
    assert_eq!(
        native_http_connect_tcp(&target, &cancelled).unwrap_err(),
        NativeHttpConnectError::Cancelled
    );
}
