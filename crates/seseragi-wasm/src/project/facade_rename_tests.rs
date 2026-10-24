use super::{analyze_project, compile_project};
use serde_json::{json, Value};

const MODEL: &str = include_str!(
    "../../../../examples/spec/fixtures/rust-backend-projects/facade-generic-renames/vendor/model/src/lib.ssrg"
);
const MAIN: &str = include_str!(
    "../../../../examples/spec/fixtures/rust-backend-projects/facade-generic-renames/src/main.ssrg"
);

fn request(type_name: &str, trait_name: &str, main: &str) -> String {
    json!({
        "schema": 1,
        "entry": "app/main.ssrg",
        "files": [
            {"path": "model/lib.ssrg", "source": MODEL},
            {
                "path": "app/facade.ssrg",
                "source": format!(
                    "pub import {{Box as {type_name}, Render as {trait_name}, rendered, boxed}} from \"../model/lib\"\n"
                )
            },
            {
                "path": "app/main.ssrg",
                "source": main
                    .replace("ExportedBox as Foreign", &format!("{type_name} as Foreign"))
                    .replace("ExportedTrait as Display", &format!("{trait_name} as Display"))
            }
        ]
    })
    .to_string()
}

#[test]
fn facade_renames_preserve_generic_constraints_through_the_project_api() {
    for (label, type_name, trait_name) in [
        ("control", "Box", "Render"),
        ("type", "ExportedBox", "Render"),
        ("trait", "Box", "ExportedTrait"),
        ("both", "ExportedBox", "ExportedTrait"),
    ] {
        let request = request(type_name, trait_name, MAIN);
        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "success", "{label}: {compiled}");
        assert_eq!(analyzed["status"], "success", "{label}: {analyzed}");
        assert_eq!(compiled["modules"].as_array().unwrap().len(), 3);
        assert_eq!(analyzed["documents"].as_array().unwrap().len(), 3);
        assert_eq!(compiled["entry"]["path"], "app/main.ssrg");
        assert_eq!(
            compiled["entry"]["contract"]["environment"][0]["service"],
            "console"
        );
        if let Ok(directory) = std::env::var("SESERAGI_FACADE881_WASM_CAPTURE") {
            let directory = std::path::Path::new(&directory).join(label);
            std::fs::create_dir_all(&directory).unwrap();
            for (name, value) in [
                ("request", serde_json::from_str(&request).unwrap()),
                ("analyzed", analyzed),
                ("compiled", compiled),
            ] {
                std::fs::write(
                    directory.join(format!("{name}.json")),
                    serde_json::to_vec_pretty(&value).unwrap(),
                )
                .unwrap();
            }
        }
    }
}

#[test]
fn facade_renames_keep_wrong_nominal_arguments_positioned_and_unemitted() {
    let argument = "(Local 1)";
    let source = format!("{MAIN}\npub fn wrong unit: Unit -> String = retained {argument} True\n");
    let start = source.rfind(argument).unwrap();
    let request = request("ExportedBox", "ExportedTrait", &source);
    let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
    let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
    assert_eq!(compiled["status"], "failure", "{compiled}");
    assert!(compiled.get("modules").is_none());
    assert_eq!(compiled["problems"], json!([]));
    assert_eq!(compiled["diagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(compiled["diagnostics"][0]["path"], "app/main.ssrg");
    let diagnostics = &compiled["diagnostics"][0]["diagnostics"];
    assert_eq!(diagnostics["diagnostics"].as_array().unwrap().len(), 1);
    let diagnostic = &diagnostics["diagnostics"][0];
    assert_eq!(diagnostic["code"], "SES-T0101");
    assert_eq!(diagnostic["messageKey"], "call.argument-type-mismatch");
    assert_eq!(
        diagnostic["primary"],
        json!({"start": start, "end": start + argument.len()})
    );
    let document = analyzed["documents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|document| document["path"] == "app/main.ssrg")
        .unwrap();
    assert_eq!(&document["document"]["diagnostics"], diagnostics);
    if let Ok(directory) = std::env::var("SESERAGI_FACADE881_WASM_CAPTURE") {
        let directory = std::path::Path::new(&directory).join("wrong-nominal");
        std::fs::create_dir_all(&directory).unwrap();
        for (name, value) in [
            ("request", serde_json::from_str(&request).unwrap()),
            ("analyzed", analyzed),
            ("compiled", compiled),
        ] {
            std::fs::write(
                directory.join(format!("{name}.json")),
                serde_json::to_vec_pretty(&value).unwrap(),
            )
            .unwrap();
        }
    }
}
