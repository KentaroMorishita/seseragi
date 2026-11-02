use std::path::Path;

pub fn ts_defect_from_native(stderr: &[u8]) -> String {
    let text = std::str::from_utf8(stderr).unwrap();
    // Some native Int overflow paths omit the numeric runtime's message prefix.
    // Keep source positions, labels and the rest of each message exact.
    let diagnostic = text
        .split_inclusive('\n')
        .filter(|line| {
            !line.starts_with("native managed-live=")
                && !line.starts_with("native clock timers-live=")
        })
        .collect::<String>()
        .replace(
            ": Seseragi Defect: Int ",
            ": Seseragi Defect: Seseragi Int ",
        )
        .replace(
            ": Seseragi suppressed Defect: Int ",
            ": Seseragi suppressed Defect: Seseragi Int ",
        );
    assert!(diagnostic.contains(": Seseragi Defect: "), "{text}");
    diagnostic
}

pub fn defect_at_source(path: &Path, source: &str, marker: &str, message: &str) -> String {
    assert_eq!(source.matches(marker).count(), 1);
    let start = source.find(marker).unwrap();
    let prefix = &source[..start];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
    format!(
        "{}:{line}:{column}: Seseragi Defect: {message}\n",
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    )
}
