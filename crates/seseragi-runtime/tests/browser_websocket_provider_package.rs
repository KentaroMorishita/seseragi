use seseragi_runtime::stage_typescript_package;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn staged_browser_websocket_provider_resolves_through_runtime_package() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "seseragi-browser-websocket-provider-{}-{nonce}",
        std::process::id()
    ));

    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }

    stage_typescript_package(&root).unwrap();

    let provider =
        fs::read_to_string(root.join("node_modules/seseragi/runtime-browser/websocket-client.ts"))
            .unwrap();

    assert!(provider.contains("@seseragi/runtime/browser/provider-websocket"));
    assert!(!provider.contains("../websocket-host-provider"));
    let implementation = fs::read_to_string(
        root.join("node_modules/@seseragi/runtime/src/browser/provider-websocket.ts"),
    )
    .unwrap();
    assert!(implementation.contains("@seseragi/runtime/websocket-host-provider"));
    assert!(!implementation.contains("../websocket-host-provider"));
    assert!(root
        .join("node_modules/@seseragi/runtime/src/websocket-host-provider.ts")
        .is_file());

    fs::remove_dir_all(root).unwrap();
}
