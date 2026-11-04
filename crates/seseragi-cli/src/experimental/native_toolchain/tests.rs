use super::*;

const HOSTS: [&str; 4] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
];

#[test]
fn rust_version_is_the_workspaces_exact_release() {
    let manifest: toml::Value = include_str!("../../../../../rust-toolchain.toml")
        .parse()
        .unwrap();
    assert_eq!(
        version(),
        manifest["toolchain"]["channel"].as_str().unwrap()
    );
    assert_eq!(
        version()
            .split('.')
            .filter(|part| part.parse::<u32>().is_ok())
            .count(),
        3
    );
    assert!(HOSTS.contains(&host().unwrap()));
}

#[test]
fn linker_and_sdk_diagnostics_explain_each_distribution() {
    for host in HOSTS {
        let error = compilation_error(host, "error: linking with linker failed: SDK is missing");
        assert!(error.contains(host), "{error}");
        assert!(error.contains("SDK is missing"), "{error}");
        assert!(
            error.contains(if host.contains("apple") {
                "xcode-select --install"
            } else if host.contains("windows") {
                "Desktop development with C++"
            } else {
                "build-essential"
            }),
            "{error}"
        );
        let ordinary = compilation_error(host, "error: cannot find value `missing` in this scope");
        assert!(!ordinary.contains("Install"), "{ordinary}");
        let std = compilation_error(host, "error[E0463]: can't find crate for `std`");
        assert!(
            std.contains("standard library is missing or incompatible"),
            "{std}"
        );
        assert!(std.contains(version()), "{std}");
        assert!(std.contains(host), "{std}");
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn write_download(path: &Path, source: &str) -> Result<(), String> {
        // As with curl, only the downloader process opens the output for writing.
        // A concurrent fork in this test process cannot inherit that descriptor
        // and keep the executable busy after the downloader has returned.
        let output = Command::new("/bin/sh")
            .args(["-c", "printf '%s' \"$2\" > \"$1\"", "write-download"])
            .arg(path)
            .arg(source)
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }
        Ok(())
    }

    fn script(path: &Path, source: &str) {
        write_download(path, source).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn fake(directory: &Path, host: &str) {
        fs::create_dir_all(directory.join("bin")).unwrap();
        let suffix = if host.contains("windows") { ".exe" } else { "" };
        script(
            &directory.join(format!("bin/rustc{suffix}")),
            &format!(
                "#!/bin/sh\nprintf 'rustc {}\\nhost: {host}\\n'\n",
                version()
            ),
        );
        script(
            &directory.join(format!("bin/cargo{suffix}")),
            &format!("#!/bin/sh\nprintf 'cargo {}\\n'\n", version()),
        );
        let std = directory.join("lib/rustlib").join(host).join("lib");
        fs::create_dir_all(&std).unwrap();
        fs::write(std.join("libstd-test.rlib"), "std").unwrap();
    }

    fn load(directory: &Path, host: &str) -> Result<Toolchain, String> {
        Toolchain::load(directory.to_owned(), directory.join("cargo-home"), host)
    }

    #[test]
    fn all_distributions_reject_wrong_versions_hosts_and_incomplete_components() {
        for host in HOSTS {
            let root = tempfile::tempdir().unwrap();
            fake(root.path(), host);
            assert!(load(root.path(), host).is_ok());
            for tool in ["rustc", "cargo"] {
                let suffix = if host.contains("windows") { ".exe" } else { "" };
                let path = root.path().join(format!("bin/{tool}{suffix}"));
                script(&path, &format!("#!/bin/sh\nprintf '{tool} 0.1.0\\n'\n"));
                let error = load(root.path(), host).err().unwrap();
                assert!(error.contains("version mismatch"), "{error}");
                assert!(error.contains(host), "{error}");
                assert!(error.contains(version()), "{error}");
                assert!(error.contains("0.1.0"), "{error}");
                fs::remove_file(&path).unwrap();
                let error = load(root.path(), host).err().unwrap();
                assert!(error.contains("missing or cannot run"), "{error}");
                fake(root.path(), host);
            }
            let wrong_host = if host.contains("windows") {
                "wrong-pc-windows-msvc"
            } else {
                "wrong-host"
            };
            let error = load(root.path(), wrong_host).err().unwrap();
            assert!(error.contains("host mismatch"), "{error}");
            fs::remove_file(
                root.path()
                    .join("lib/rustlib")
                    .join(host)
                    .join("lib/libstd-test.rlib"),
            )
            .unwrap();
            let error = load(root.path(), host).err().unwrap();
            assert!(error.contains("standard library is missing"), "{error}");
            assert!(error.contains(host), "{error}");
        }
    }

    #[test]
    fn provisioning_is_atomic_reusable_offline_and_serializes_concurrent_installs() {
        let cache = tempfile::tempdir().unwrap();
        let count = std::sync::atomic::AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..3 {
                let count = &count;
                let cache = cache.path();
                scope.spawn(move || {
                    let toolchain = Toolchain::prepare(cache, HOSTS[0], |stage, host| {
                        count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        fake(
                            &stage
                                .join("rustup/toolchains")
                                .join(format!("{}-{host}", version())),
                            host,
                        );
                        Ok(())
                    })
                    .unwrap();
                    assert!(toolchain
                        .directory
                        .starts_with(fs::canonicalize(cache).unwrap()));
                });
            }
        });
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
        let cached = Toolchain::prepare(cache.path(), HOSTS[0], |_, _| {
            panic!("offline cache must not download")
        })
        .unwrap();
        assert!(cached.directory.is_dir());
        assert!(fs::read_dir(cache.path().join("toolchains"))
            .unwrap()
            .all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".install-")));
    }

    #[test]
    fn interrupted_or_invalid_install_is_removed_and_retry_can_succeed() {
        let cache = tempfile::tempdir().unwrap();
        for incomplete in [false, true] {
            let error = Toolchain::prepare(cache.path(), HOSTS[2], |stage, _| {
                fs::write(stage.join("partial"), "incomplete").unwrap();
                if incomplete {
                    Ok(())
                } else {
                    Err("offline".into())
                }
            })
            .err()
            .unwrap();
            assert!(
                error.contains(if incomplete { "missing" } else { "offline" }),
                "{error}"
            );
            assert_eq!(
                fs::read_dir(cache.path().join("toolchains"))
                    .unwrap()
                    .count(),
                1
            );
        }
        let cached = Toolchain::prepare(cache.path(), HOSTS[2], |stage, host| {
            fake(
                &stage
                    .join("rustup/toolchains")
                    .join(format!("{}-{host}", version())),
                host,
            );
            Ok(())
        })
        .unwrap();
        fs::remove_file(cached.directory.join("bin/cargo")).unwrap();
        let error = Toolchain::prepare(cache.path(), HOSTS[2], |_, _| {
            panic!("corrupt cache needs an explicit diagnostic")
        })
        .err()
        .unwrap();
        assert!(error.contains("remove"), "{error}");
        assert!(error.contains("retry while online"), "{error}");
    }

    #[test]
    fn installer_is_checksum_checked_and_uses_only_private_homes_and_fixed_arguments() {
        for host in HOSTS {
            let directory = tempfile::tempdir().unwrap();
            let payload = "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$CARGO_HOME-arguments\"\nprintf '%s\\n' \"$RUSTUP_HOME\" > \"$CARGO_HOME-rustup\"\nprintf '%s\\n' \"$RUSTUP_DIST_SERVER\" > \"$CARGO_HOME-server\"\n";
            let expected = format!("{:x}  rustup-init\n", Sha256::digest(payload));
            let mut urls = Vec::new();
            install_with(directory.path(), host, |url, path| {
                urls.push(url.to_owned());
                write_download(
                    path,
                    if url.ends_with(".sha256") {
                        &expected
                    } else {
                        payload
                    },
                )
            })
            .unwrap();
            assert_eq!(urls.len(), 2);
            assert!(urls.iter().all(|url| url.starts_with(&format!(
                "https://static.rust-lang.org/rustup/archive/{RUSTUP_VERSION}/{host}/"
            ))));
            assert_eq!(urls[0], format!("{}.sha256", urls[1]));
            assert!(urls[1].ends_with(if host.contains("windows") {
                "rustup-init.exe"
            } else {
                "rustup-init"
            }));
            let arguments = fs::read_to_string(directory.path().join("cargo-arguments")).unwrap();
            assert_eq!(arguments, format!("-y\n--no-modify-path\n--profile\nminimal\n--default-host\n{host}\n--default-toolchain\n{}\n", version()));
            assert_eq!(
                fs::read_to_string(directory.path().join("cargo-rustup"))
                    .unwrap()
                    .trim(),
                directory.path().join("rustup").to_str().unwrap()
            );
            assert_eq!(
                fs::read_to_string(directory.path().join("cargo-server"))
                    .unwrap()
                    .trim(),
                "https://static.rust-lang.org"
            );
            let error = install_with(directory.path(), host, |_, path| {
                write_download(path, payload)
            })
            .unwrap_err();
            assert!(error.contains("checksum mismatch"), "{error}");
            fs::remove_file(directory.path().join("cargo-arguments")).unwrap();
            let error = install_with(directory.path(), host, |url, path| {
                write_download(
                    path,
                    if url.ends_with(".sha256") {
                        "invalid"
                    } else {
                        payload
                    },
                )
            })
            .unwrap_err();
            assert!(error.contains("checksum mismatch"), "{error}");
            assert!(!directory.path().join("cargo-arguments").exists());
        }
    }
}
