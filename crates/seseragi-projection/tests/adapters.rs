use seseragi_interop::*;
use seseragi_projection::generate_adapters;
use std::collections::BTreeMap;
fn graph() -> SemanticGraph {
    let mut graph = SemanticGraph::default();
    for (id, domain) in [(0, PrimitiveDomain::String), (1, PrimitiveDomain::Integer)] {
        graph
            .types
            .insert(TypeId(id), SemanticType::Primitive { domain });
    }
    graph
        .types
        .insert(TypeId(2), SemanticType::Nullable { value: TypeId(0) });
    graph.types.insert(
        TypeId(3),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([
                (
                    "name".into(),
                    ShapeField {
                        type_ref: TypeId(0),
                        optional: false,
                        readonly: true,
                    },
                ),
                (
                    "count".into(),
                    ShapeField {
                        type_ref: TypeId(1),
                        optional: false,
                        readonly: true,
                    },
                ),
                (
                    "note".into(),
                    ShapeField {
                        type_ref: TypeId(2),
                        optional: true,
                        readonly: true,
                    },
                ),
            ]),
        },
    );
    graph
        .types
        .insert(TypeId(4), SemanticType::Nullable { value: TypeId(3) });
    graph.types.insert(
        TypeId(5),
        SemanticType::Literal {
            value: LiteralValue::String("ok".into()),
        },
    );
    graph.types.insert(
        TypeId(6),
        SemanticType::Literal {
            value: LiteralValue::String("error".into()),
        },
    );
    graph.types.insert(
        TypeId(7),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([
                (
                    "kind".into(),
                    ShapeField {
                        type_ref: TypeId(5),
                        optional: false,
                        readonly: true,
                    },
                ),
                (
                    "message".into(),
                    ShapeField {
                        type_ref: TypeId(0),
                        optional: false,
                        readonly: true,
                    },
                ),
            ]),
        },
    );
    graph.types.insert(
        TypeId(8),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([
                (
                    "kind".into(),
                    ShapeField {
                        type_ref: TypeId(6),
                        optional: false,
                        readonly: true,
                    },
                ),
                (
                    "code".into(),
                    ShapeField {
                        type_ref: TypeId(1),
                        optional: false,
                        readonly: true,
                    },
                ),
            ]),
        },
    );
    graph.types.insert(
        TypeId(9),
        SemanticType::Union {
            members: vec![TypeId(7), TypeId(8)],
        },
    );
    for (name, id) in [("Customer", 3), ("CustomerOrNull", 4), ("Event", 9)] {
        graph.symbols.insert(
            name.into(),
            SemanticSymbol {
                deprecations: Vec::new(),
                public_name: name.into(),
                declaration_name: name.into(),
                kind: SymbolKind::Type,
                type_ref: TypeId(id),
                origins: vec![],
                runtime: None,
            },
        );
    }
    graph.sources.insert(
        "fixture".into(),
        SourceIdentity {
            digest: "0".repeat(64),
            byte_length: 1,
        },
    );
    for id in graph.types.keys().copied().collect::<Vec<_>>() {
        graph.type_origins.insert(
            id,
            vec![Origin {
                source: "fixture".into(),
                start: 0,
                end: 1,
                rule: "fixture-domain".into(),
            }],
        );
    }
    graph.identity = Some(ArtifactIdentity {
        frontend: ProducerIdentity {
            name: "common-graph-fixture".into(),
            version: "1".into(),
        },
        settings_digest: "1".repeat(64),
        source_digests: BTreeMap::from([("fixture".into(), "0".repeat(64))]),
        runtime_contract_digest: None,
    });
    graph
}
fn policies(graph: &SemanticGraph) -> ProjectionPolicySet {
    ProjectionPolicySet {
        schema: 1,
        kind: PROJECTION_POLICY_KIND.into(),
        producer: ProducerIdentity {
            name: "safe-json-adapters".into(),
            version: "1".into(),
        },
        graph_digest: graph.digest().unwrap(),
        artifact_identity: graph.identity.clone().unwrap(),
        policies: vec![
            ProjectionPolicy {
                selector: ProjectionSelector::Symbol {
                    symbol: "Customer".into(),
                },
                function_name: "decodeCustomer".into(),
                mapping: AdapterMapping::JsonSnapshot,
            },
            ProjectionPolicy {
                selector: ProjectionSelector::Type {
                    type_ref: TypeId(4),
                },
                function_name: "decodeCustomerOrNull".into(),
                mapping: AdapterMapping::JsonNullable,
            },
            ProjectionPolicy {
                selector: ProjectionSelector::Symbol {
                    symbol: "Event".into(),
                },
                function_name: "decodeEvent".into(),
                mapping: AdapterMapping::JsonTaggedUnion {
                    discriminator: "kind".into(),
                },
            },
        ],
    }
}
#[test]
fn separate_semantic_source_has_fresh_policy_provenance_and_cannot_change_graph() {
    let graph = graph();
    let before = graph.clone();
    let policies = policies(&graph);
    let generated = generate_adapters(&graph, &policies).unwrap();
    assert_eq!(graph, before);
    assert_eq!(generated.input_boundary, "std/json::Json");
    assert_eq!(generated.applied_policies.len(), 3);
    assert!(generated
        .applied_policies
        .iter()
        .all(|p| !p.origins.is_empty()));
    assert!(generated.semantic_source.contains("Maybe<Maybe<String>>"));
    assert!(generated
        .semantic_source
        .contains("pub type DecodeEventValue"));
    assert!(!generated.semantic_source.contains("foreign \"typescript\""));
    assert_eq!(generate_adapters(&graph, &policies).unwrap(), generated);
    let mut stale = policies.clone();
    stale.artifact_identity.runtime_contract_digest = Some("2".repeat(64));
    assert!(generate_adapters(&graph, &stale).is_err());
}
#[test]
fn explicit_policy_still_rejects_identity_recursion_undefined_and_codegen_injection() {
    let base = graph();
    for replacement in [
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::new(),
        },
        SemanticType::Opaque {
            identity: "HostClass".into(),
            arguments: vec![],
        },
        SemanticType::Shape {
            identity: ShapeIdentity::MutableIdentity,
            fields: BTreeMap::new(),
        },
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([(
                "self".into(),
                ShapeField {
                    type_ref: TypeId(3),
                    optional: false,
                    readonly: true,
                },
            )]),
        },
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([(
                "x\"; code".into(),
                ShapeField {
                    type_ref: TypeId(0),
                    optional: false,
                    readonly: true,
                },
            )]),
        },
    ] {
        let mut graph = base.clone();
        graph.types.insert(TypeId(3), replacement);
        assert!(generate_adapters(&graph, &policies(&graph)).is_err());
    }
    let mut graph = base;
    graph
        .types
        .insert(TypeId(2), SemanticType::Optional { value: TypeId(0) });
    assert!(generate_adapters(&graph, &policies(&graph)).is_err());
}
#[test]
fn generated_snapshot_nullable_and_tagged_adapters_build_and_execute_checked_failures() {
    let graph = graph();
    let generated = generate_adapters(&graph, &policies(&graph)).unwrap();
    let program = format!(
        "{}\n{}",
        generated.semantic_source,
        r#"
fn input text: String -> json.Json = match json.parse text { Left _ -> json.JsonNull; Right value -> value }
fn label value: Either<json.DecodeError, { count: Int, name: String, note: Maybe<Maybe<String>> }> -> String = match value {
  Left _ -> "bad"
  Right record -> match record.note { Nothing -> "absent"; Just Nothing -> "null"; Just (Just text) -> text }
}
fn nullableLabel value: Either<json.DecodeError, Maybe<{ count: Int, name: String, note: Maybe<Maybe<String>> }>> -> String = match value {
  Left _ -> "bad"
  Right Nothing -> "nothing"
  Right (Just _) -> "just"
}
fn eventLabel value: Either<json.DecodeError, DecodeEventValue> -> String = match value {
  Left _ -> "bad"
  Right (DecodeEventValueCase0 payload) -> payload.message
  Right (DecodeEventValueCase1 payload) -> show payload.code
}
pub effect fn main -> Unit with Console fails ConsoleError = do {
  println $ label (decodeCustomer (input "{\"count\":1,\"name\":\"Mio\"}"))
  println $ label (decodeCustomer (input "{\"count\":1,\"name\":\"Mio\",\"note\":null}"))
  println $ label (decodeCustomer (input "{\"count\":1,\"name\":\"Mio\",\"note\":\"Ada\"}"))
  println $ label (decodeCustomer (input "{\"count\":\"bad\",\"name\":\"Mio\"}"))
  println $ nullableLabel (decodeCustomerOrNull (input "null"))
  println $ nullableLabel (decodeCustomerOrNull (input "{\"count\":1,\"name\":\"Mio\"}"))
  println $ eventLabel (decodeEvent (input "{\"kind\":\"ok\",\"message\":\"success\"}"))
  println $ eventLabel (decodeEvent (input "{\"kind\":\"error\",\"code\":7}"))
  println $ eventLabel (decodeEvent (input "{\"kind\":\"unknown\",\"message\":\"bad\"}"))
  println $ eventLabel (decodeEvent (input "{\"kind\":\"ok\",\"message\":1}"))
}
"#
    );
    let compiled = seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        "semantic-adapters.ssrg",
        "fixture/semantic-adapters",
        &program,
    ))
    .unwrap_or_else(|e| panic!("{e:?}\n{program}"));
    let dir = tempfile::tempdir().unwrap();
    seseragi_runtime::build_main(
        &compiled,
        dir.path(),
        seseragi_runtime::BuildTarget::Process,
    )
    .unwrap();
    let output = std::process::Command::new("bun")
        .arg(dir.path().join("entry.ts"))
        .output()
        .unwrap();
    if !output.status.success() {
        let debug = dir.keep();
        panic!(
            "adapter execution defect; debug artifact: {}\nstdout: {}\nstderr: {}",
            debug.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "absent\nnull\nAda\nbad\nnothing\njust\nsuccess\n7\nbad\nbad\n"
    );
}

#[test]
fn literal_escape_and_nullary_tag_variants_generate_valid_native_source() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(5),
        SemanticType::Literal {
            value: LiteralValue::String("ok\n\u{001f}\"\\".into()),
        },
    );
    graph.types.insert(
        TypeId(7),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([(
                "kind".into(),
                ShapeField {
                    type_ref: TypeId(5),
                    optional: false,
                    readonly: true,
                },
            )]),
        },
    );
    let generated = generate_adapters(&graph, &policies(&graph)).unwrap();
    assert!(generated.semantic_source.contains("\\u{1f}"));
    assert!(generated
        .semantic_source
        .contains("| DecodeEventValueCase0\n"));
    seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        "escaped-adapters.ssrg",
        "fixture/escaped-adapters",
        &generated.semantic_source,
    ))
    .unwrap();
}

#[test]
fn adapter_names_cannot_shadow_the_standard_decoder_used_by_generated_code() {
    let graph = graph();
    let mut set = policies(&graph);
    set.policies[0].function_name = "decodeJson".to_owned();
    assert!(generate_adapters(&graph, &set).is_err());
}

#[test]
fn checked_json_snapshot_preserves_underscore_and_prototype_keys_on_both_hosts() {
    let mut graph = graph();
    let fields = ["__proto__", "_id", "constructor", "toString"]
        .into_iter()
        .map(|name| {
            (
                name.to_owned(),
                ShapeField {
                    type_ref: TypeId(0),
                    optional: false,
                    readonly: true,
                },
            )
        })
        .collect();
    graph.types.insert(
        TypeId(3),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields,
        },
    );
    let mut policy = policies(&graph);
    policy.policies.truncate(1);
    let generated = generate_adapters(&graph, &policy).unwrap();
    let program = format!(
        "{}\n{}",
        generated.semantic_source,
        r#"
fn input text: String -> json.Json = match json.parse text { Left _ -> json.JsonNull; Right value -> value }
pub effect fn main -> Unit with Console fails ConsoleError = do {
  match decodeCustomer (input "{\"__proto__\":\"kept\",\"_id\":\"user-1\",\"constructor\":\"ctor\",\"toString\":\"named\"}") {
    Left _ -> println "unexpected rejection"
    Right value -> do { println value.__proto__; println value._id; println value.constructor; println value.toString }
  }
  match decodeCustomer (input "{\"__proto__\":{},\"_id\":\"user-1\",\"constructor\":\"ctor\",\"toString\":\"named\"}") {
    Left _ -> println "rejected"
    Right _ -> println "unexpected acceptance"
  }
}
"#
    );
    for profile in [
        seseragi_project::BuildProfile::Development,
        seseragi_project::BuildProfile::Release,
    ] {
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "prototype-adapter.ssrg",
                "fixture/prototype-adapter",
                &program,
            )
            .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:?}\n{program}"));
        assert!(compiled.generated.typescript.contains("[\"__proto__\"]:"));
        let directory = tempfile::tempdir().unwrap();
        seseragi_runtime::build_main(
            &compiled,
            directory.path(),
            seseragi_runtime::BuildTarget::Process,
        )
        .unwrap();
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(directory.path().join("artifact-manifest.json")).unwrap(),
        )
        .unwrap();
        for host in ["node", "bun"] {
            let output = std::process::Command::new(host)
                .arg(manifest["entry"].as_str().unwrap())
                .current_dir(directory.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{profile:?}/{host}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stdout),
                "kept\nuser-1\nctor\nnamed\nrejected\n",
                "{profile:?}/{host}"
            );
        }
    }
    // The bare wildcard and code-bearing spellings remain unrepresentable.
    for name in ["_", "_x; injected", "_x-y"] {
        let mut invalid = graph.clone();
        invalid.types.insert(
            TypeId(3),
            SemanticType::Shape {
                identity: ShapeIdentity::ReadonlySnapshot,
                fields: BTreeMap::from([(
                    name.to_owned(),
                    ShapeField {
                        type_ref: TypeId(0),
                        optional: false,
                        readonly: true,
                    },
                )]),
            },
        );
        let mut rejected_policy = policies(&invalid);
        rejected_policy.policies.truncate(1);
        assert!(generate_adapters(&invalid, &rejected_policy).is_err());
    }
}
