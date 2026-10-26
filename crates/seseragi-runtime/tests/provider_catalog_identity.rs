#[test]
fn embedded_filesystem_provider_matches_the_portable_package_lock() {
    let project = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend-projects/decimal-file-aggregate");
    let lock = seseragi_project::read_and_validate_lockfile(&project).unwrap();
    let configuration = seseragi_runtime::bun_process_provider_configuration().unwrap();
    let locked = lock
        .providers
        .iter()
        .find(|provider| provider.service == "std/fs::FileSystem")
        .unwrap();
    let embedded = configuration
        .candidates
        .iter()
        .find(|candidate| candidate.manifest.identity == locked.provider)
        .unwrap();

    // Keep the committed lock frozen. Updating it here would hide checkout
    // line-ending changes in the raw contract/manifest bytes embedded by Rust.
    assert_eq!(embedded.package.content_digest, locked.package_digest);
    assert_eq!(embedded.artifact_digest, locked.artifact_digest);
    assert_eq!(embedded.package.version, locked.package_version);
    assert_eq!(embedded.package.source_identity, locked.package_source);
}
