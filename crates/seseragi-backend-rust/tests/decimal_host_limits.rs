use seseragi_backend_rust::{emit_rust_module_with_options, RustSourceFile, RustTargetOptions};
use seseragi_driver::{compile_core_module, CompileInput};
use std::process::Command;

#[path = "support/native.rs"]
mod native;

#[test]
fn decimal_host_limits_report_defects_after_resource_cleanup() {
    for expression in [
        "value \"1e18446744073709551616\" + decimal.fromInt 1",
        "value \"1e18446744073709551616\" - decimal.fromInt 1",
        "match decimal.divide (chosen 9007199254740991) (decimal.fromInt 3) (decimal.fromInt 1) { Right result -> result; Left _ -> decimal.fromInt 0 }",
    ] {
        let source = format!(
            r#"import * as decimal from "std/decimal"
import * as effects from "std/effect"
import * as numbers from "std/number"
fn value text:String -> decimal.Decimal = match decimal.parse text {{ Right result -> result; Left _ -> decimal.fromInt 0 }}
fn chosen digits:Int -> decimal.DecimalContext = match decimal.context digits numbers.HalfEven {{ Right result -> result; Left _ -> chosen 1 }}
effect fn ignore _error:ConsoleError -> Unit with Console = effects.succeed ()
effect fn say text:String -> Unit with Console = println text |> effects.recover ignore
pub effect fn main = effects.scoped (do {{
  effects.acquireRelease (say "acquired") (\_:Unit -> say "released")
  let result = {expression}
  say "returned"
}})
"#
        );
        let checked = compile_core_module(CompileInput::new(
            "decimal-host-limits.ssrg",
            "native/decimal-host-limits",
            &source,
        ))
        .unwrap();
        let mut options = RustTargetOptions::default();
        options.sources.insert(
            checked.core_ir.module.clone(),
            RustSourceFile {
                path: "decimal-host-limits.ssrg".into(),
                text: source,
            },
        );
        let artifact = emit_rust_module_with_options(&checked.core_ir, "main", &options).unwrap();
        let scratch = native::Scratch::new();
        let output = Command::new(scratch.compile(&artifact.source))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(output.status.code(), Some(70), "{expression}: {stderr}");
        assert_eq!(output.stdout, b"acquired\nreleased\n", "{expression}");
        assert!(
            stderr.starts_with("decimal-host-limits.ssrg:10:"),
            "{stderr}"
        );
        assert!(
            stderr.contains(": Seseragi Defect: Decimal canonical spelling exceeds host limits"),
            "{stderr}"
        );
        assert!(!stderr.contains("panicked at"), "{stderr}");
        assert!(stderr.contains("native managed-live=0 "), "{stderr}");
    }
}

#[test]
fn product_allocation_failure_becomes_a_defect_instead_of_aborting() {
    let artifact = native::emit(
        "import * as decimal from \"std/decimal\"\npub fn probe -> decimal.Decimal = decimal.fromInt 12 * decimal.fromInt 34\n",
    );
    let entry = artifact.source.rfind("fn main()").unwrap();
    let generated = format!(
        "{}\n{}",
        &artifact.source[..entry],
        r#"
struct FailingAllocator;
static FAIL_NEXT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[global_allocator]
static ALLOCATOR: FailingAllocator = FailingAllocator;
unsafe impl std::alloc::GlobalAlloc for FailingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        if FAIL_NEXT.swap(false, std::sync::atomic::Ordering::SeqCst) {
            std::ptr::null_mut()
        } else {
            std::alloc::GlobalAlloc::alloc(&std::alloc::System, layout)
        }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: std::alloc::Layout) {
        std::alloc::GlobalAlloc::dealloc(&std::alloc::System, pointer, layout);
    }
}
fn main() {
    std::process::exit(run_native_entry(|| V::leaf(vec![], |_, _| {
        FAIL_NEXT.store(true, std::sync::atomic::Ordering::SeqCst);
        let product = native_decimal_coefficient_multiply(
            std::hint::black_box("12"),
            std::hint::black_box("34"),
        );
        Ok(V::text(&product))
    }), false));
}
"#
    );
    let scratch = native::Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(70), "{stderr}");
    assert!(output.stdout.is_empty());
    assert!(
        stderr.starts_with("Seseragi Defect: Decimal coefficient exceeds host limits\n"),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked at"), "{stderr}");
    assert!(stderr.contains("native managed-live=0 "), "{stderr}");
}
