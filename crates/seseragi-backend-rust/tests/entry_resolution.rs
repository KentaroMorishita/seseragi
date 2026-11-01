use seseragi_backend_rust::{emit_rust_module, emit_rust_project};
use seseragi_driver::{
    compile_core_module, compile_core_project, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;

#[test]
fn native_entry_uses_the_module_symbol_even_when_a_method_has_the_same_name() {
    let checked = compile_core_module(CompileInput::new(
        "entry.ssrg", "entry/main",
        "struct Box { value: Int }\nimpl Box { fn main self: Box -> Int = self.value }\npub effect fn main = println \"entry\"\n",
    )).unwrap();
    let mut module = checked.core_ir;
    module
        .functions
        .sort_by_key(|function| function.symbol == "entry/main::main");
    let artifact = emit_rust_module(&module, "main").unwrap();
    assert_eq!(artifact.metadata.entry, "entry/main::main");
    assert_eq!(artifact.metadata.entry_kind, "effect");
    module
        .functions
        .retain(|function| function.symbol != "entry/main::main");
    let error = emit_rust_module(&module, "main").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.module.as_deref(), Some("entry/main"));
    let origin = error.origin.unwrap();
    assert_eq!(origin.source, "entry.ssrg");
    assert_eq!((origin.start, origin.end), (0, 0));
}

#[test]
fn a_facade_display_name_cannot_be_selected_as_a_native_entry() {
    let mut graph = ModuleGraph::new();
    graph.add_module("entry/model".to_owned(), []).unwrap();
    graph
        .add_module(
            "entry/facade".to_owned(),
            [("./model".to_owned(), "entry/model".to_owned())],
        )
        .unwrap();
    let checked = compile_core_project(
        graph,
        [
            ProjectModuleInput::new(
                "model.ssrg",
                "entry/model",
                "pub effect fn other = println \"model\"\n",
                "model.js",
            ),
            ProjectModuleInput::new(
                "facade.ssrg",
                "entry/facade",
                "pub import { other as main } from \"./model\"\n",
                "facade.js",
            ),
        ],
    )
    .unwrap();
    let modules = checked
        .order
        .iter()
        .map(|id| &checked.modules[id].core_ir)
        .collect::<Vec<_>>();
    let error = emit_rust_project(&modules, "entry/facade", "main").unwrap_err();
    assert_eq!(error.module.as_deref(), Some("entry/facade"));
    assert_eq!(
        error.message,
        "Rust backend: entry function `entry/facade::main` not found"
    );
    let origin = error.origin.unwrap();
    assert_eq!(origin.source, "facade.ssrg");
    assert_eq!((origin.start, origin.end), (0, 0));
}

#[test]
fn a_missing_entry_in_an_empty_module_has_a_module_location() {
    let checked = compile_core_module(CompileInput::new("empty.ssrg", "entry/empty", "")).unwrap();
    let error = emit_rust_module(&checked.core_ir, "main").unwrap_err();
    assert_eq!(error.module.as_deref(), Some("entry/empty"));
    let origin = error.origin.unwrap();
    assert_eq!((origin.start, origin.end), (0, 0));
}
