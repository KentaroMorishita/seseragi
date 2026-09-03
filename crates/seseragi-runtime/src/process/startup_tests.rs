use super::{entry_source, prepare_directory, web_entry::web_entry_source};
use crate::{FailureRenderer, MainContract, ProcessRunOptions, RandomSeed};
use std::{fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn execute_startup(browser: bool, seed: RandomSeed) -> std::process::Output {
    let fixture = Fixture(prepare_directory().unwrap());
    crate::stage_typescript_package(&fixture.0).unwrap();
    fs::write(
        fixture.0.join("application.ts"),
        "import { empty } from '@seseragi/runtime/map';\n\
         import { processHashSeed } from '@seseragi/runtime/hash';\n\
         const topLevelMap = empty();\n\
         console.log('application:' + processHashSeed());\n\
         throw new Error('application fixture boundary');\n\
         export const main = () => topLevelMap;\n",
    )
    .unwrap();
    let contract = MainContract {
        environment: vec![],
        failure_renderer: FailureRenderer::Never,
    };
    let options = ProcessRunOptions {
        hash_seed: seed,
        ..ProcessRunOptions::default()
    };
    let source = if browser {
        web_entry_source(&contract, "./application.ts", None, options)
    } else {
        entry_source(&contract, "./application.ts", None, options)
    };
    fs::write(
        fixture.0.join("entry.ts"),
        format!("Object.defineProperty(globalThis, 'crypto', {{ value: undefined }});\n{source}"),
    )
    .unwrap();
    Command::new("bun")
        .arg("run")
        .arg(fixture.0.join("entry.ts"))
        .current_dir(&fixture.0)
        .output()
        .unwrap()
}

#[test]
fn process_and_web_fixed_seeds_precede_top_level_map_initialization() {
    for (browser, seed) in [
        (false, -7),
        (true, -11),
        (false, i64::MIN),
        (false, i64::MAX),
        (true, i64::MIN),
        (true, i64::MAX),
    ] {
        let output = execute_startup(browser, RandomSeed::Fixed(seed));
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("application:{seed}\n")
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("application fixture boundary"));
    }
}

#[test]
fn missing_entropy_stops_process_and_web_before_any_application_code() {
    for browser in [false, true] {
        let output = execute_startup(browser, RandomSeed::Entropy);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty(), "application must not evaluate");
        assert!(String::from_utf8_lossy(&output.stderr).contains("secure entropy is unavailable"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("application fixture boundary"));
    }
}
