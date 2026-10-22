use super::entry_source;
use crate::{
    build_main, run_main, validate_main_process_entry, BuildError, BuildTarget, EnvironmentBinding,
    FailureRenderer, HostService, MainContract, ProcessRunOptions, RunError,
};
use seseragi_driver::{
    compile_module, CompileInput, ContractVersion, ProviderBuildMetadata, ProviderLockMetadata,
    ProviderResolution, ProviderSelectionMetadata,
};
use std::fs;

fn contract(bindings: &[(&str, HostService)]) -> MainContract {
    MainContract {
        environment: bindings
            .iter()
            .map(|(field, service)| EnvironmentBinding {
                field: (*field).to_owned(),
                service: *service,
            })
            .collect(),
        failure_renderer: FailureRenderer::Never,
    }
}

fn selection(field: &str, service: &str) -> ProviderSelectionMetadata {
    ProviderSelectionMetadata {
        field: field.to_owned(),
        service: service.to_owned(),
        required_contract_version: ContractVersion { major: 1, minor: 0 },
        provider_contract_version: ContractVersion { major: 1, minor: 0 },
        provider: format!("fixture/{field}#provider"),
        package_version: "1.0.0".to_owned(),
        package_source: "toolchain".to_owned(),
        package_digest: "fixture".to_owned(),
        artifact_digest: "fixture".to_owned(),
        backend_family: "typescript".to_owned(),
        backend_abi_major: 1,
        target: "bun-process".to_owned(),
        entry_module: format!("fixture/{field}/provider"),
        entry_export: "provider".to_owned(),
        runtime_features: Vec::new(),
        host_packages: Vec::new(),
        traces: Vec::new(),
    }
}

fn resolution(selected: Vec<ProviderSelectionMetadata>) -> ProviderResolution {
    ProviderResolution {
        lock: ProviderLockMetadata {
            schema: 1,
            providers: selected.clone(),
        },
        build: ProviderBuildMetadata {
            schema: 1,
            target: "bun-process".to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: Vec::new(),
            providers: selected.clone(),
        },
        selected,
    }
}

#[test]
fn every_provider_required_service_rejects_missing_selection_without_panicking() {
    for (host, service) in [
        (
            HostService::ChildProcesses,
            "std/child-process::ChildProcesses",
        ),
        (HostService::Clock, "std/clock::Clock"),
        (HostService::Random, "std/random::Random"),
        (HostService::Entropy, "std/entropy::Entropy"),
        (HostService::HttpClient, "std/http::HttpClient"),
        (HostService::HttpServer, "std/http/server::HttpServer"),
        (
            HostService::WebSocketClient,
            "std/websocket::WebSocketClient",
        ),
        (
            HostService::WebSocketServer,
            "std/websocket/server::WebSocketServer",
        ),
        (HostService::Postgres, "seseragi/postgres::Postgres"),
        (HostService::Sqlite, "seseragi/sqlite::Sqlite"),
    ] {
        let contract = contract(&[("missing", host)]);
        for providers in [None, Some(resolution(Vec::new()))] {
            let error = entry_source(
                &contract,
                "./main.ts",
                providers.as_ref(),
                ProcessRunOptions::default(),
            )
            .unwrap_err();
            assert!(error.contains("`missing`"), "{error}");
            assert!(error.contains(service), "{error}");
            assert!(error.contains("use a package"), "{error}");
        }
    }
}

#[test]
fn selection_is_matched_to_each_environment_field_and_service() {
    let contract = contract(&[
        ("first", HostService::HttpClient),
        ("second", HostService::HttpClient),
        ("clock", HostService::Clock),
    ]);
    let first = selection("first", "std/http::HttpClient");
    let second = selection("second", "std/http::HttpClient");
    let clock = selection("clock", "std/clock::Clock");
    let source = entry_source(
        &contract,
        "./main.ts",
        Some(&resolution(vec![second.clone(), clock, first.clone()])),
        ProcessRunOptions::default(),
    )
    .unwrap();
    assert!(source.contains("providerLoader0.load(\"fixture/first#provider\")"));
    assert!(source.contains("providerLoader1.load(\"fixture/second#provider\")"));
    assert!(source.contains("providerLoader2.load(\"fixture/clock#provider\")"));
    let partial = entry_source(
        &contract,
        "./main.ts",
        Some(&resolution(vec![first, second])),
        ProcessRunOptions::default(),
    )
    .unwrap_err();
    assert!(partial.contains("`clock`"), "{partial}");
}

#[test]
fn malformed_provider_selections_are_errors_before_entry_generation() {
    let contract = contract(&[("client", HostService::HttpClient)]);
    let valid = selection("client", "std/http::HttpClient");
    let mut wrong_field = valid.clone();
    wrong_field.field = "other".to_owned();
    let mut wrong_service = valid.clone();
    wrong_service.service = "std/clock::Clock".to_owned();
    let mut wrong_target = valid.clone();
    wrong_target.target = "browser".to_owned();
    let mut empty_provider = valid.clone();
    empty_provider.provider.clear();
    let mut empty_module = valid.clone();
    empty_module.entry_module.clear();
    let mut empty_export = valid.clone();
    empty_export.entry_export.clear();
    for (selected, expected) in [
        (vec![valid.clone(), valid], "multiple resolved providers"),
        (vec![wrong_field], "requires a resolved provider"),
        (vec![wrong_service], "requires a resolved provider"),
        (vec![wrong_target], "instead of `bun-process`"),
        (vec![empty_provider], "empty provider identity or entry"),
        (vec![empty_module], "empty provider identity or entry"),
        (vec![empty_export], "empty provider identity or entry"),
    ] {
        let error = entry_source(
            &contract,
            "./main.ts",
            Some(&resolution(selected)),
            ProcessRunOptions::default(),
        )
        .unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("std/http::HttpClient"), "{error}");
    }
}

#[test]
fn existing_optional_native_adapters_keep_their_standalone_fallbacks() {
    let source = entry_source(
        &contract(&[
            ("timeZones", HostService::TimeZones),
            ("fileSystem", HostService::FileSystem),
        ]),
        "./main.ts",
        None,
        ProcessRunOptions::default(),
    )
    .unwrap();
    assert!(source.contains("seseragi/runtime#timezones"));
    assert!(source.contains("seseragi/runtime-bun#filesystem"));
}

#[test]
fn standalone_public_runner_and_builder_return_invalid_entry_before_publication() {
    let source = "import * as http from \"std/http\"\n\npub effect fn main -> Unit\nwith httpClient: http.HttpClient =\n  succeed ()\n";
    let compiled = compile_module(CompileInput::new("main.ssrg", "fixture/main", source)).unwrap();
    let error = validate_main_process_entry(&compiled).unwrap_err();
    assert!(matches!(error, RunError::InvalidEntry(_)), "{error}");
    let error = run_main(&compiled).unwrap_err();
    assert!(matches!(error, RunError::InvalidEntry(_)), "{error}");
    let fixture = tempfile::tempdir().unwrap();
    let output = fixture.path().join("out");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("sentinel"), "previous output").unwrap();
    let error = build_main(&compiled, &output, BuildTarget::Process).unwrap_err();
    assert!(matches!(error, BuildError::InvalidEntry(_)), "{error}");
    assert_eq!(
        fs::read_to_string(output.join("sentinel")).unwrap(),
        "previous output"
    );
    assert_eq!(fs::read_dir(&output).unwrap().count(), 1);
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 1);
}
