use seseragi_driver::{compile_module, BuildProfile, CompileInput};

fn source(environment: &str, requirements: &str, stream: &str) -> String {
    format!(
        "import * as effects from \"std/effect\"\n\
         import * as fs from \"std/fs\"\n\
         import * as streams from \"std/stream\"\n\
         import * as bytes from \"std/bytes\"\n\
         import * as paths from \"std/path\"\n\
         pub effect fn cancel source: streams.Stream<{environment},String,bytes.Bytes> -> path: paths.Path -> Unit with {requirements} = do {{\n\
           worker <- effects.fork (fs.writeChunks fs.Replace ({stream}) path)\n\
           effects.interrupt worker\n\
         }}\n"
    )
}

#[test]
fn fork_infers_the_effect_of_a_write_from_a_computed_stream() {
    for (environment, requirements) in
        [("{}", "FileSystem"), ("{stdin:Stdin}", "FileSystem, Stdin")]
    {
        for stream in ["source", "streams.concat source source"] {
            let source = source(environment, requirements, stream);
            for profile in [BuildProfile::Development, BuildProfile::Release] {
                let compiled = compile_module(
                    CompileInput::new("fork-write.ssrg", "fixture/fork-write", &source)
                        .with_profile(profile),
                )
                .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
                assert!(compiled.diagnostics.diagnostics.is_empty());
            }
        }
    }
}

#[test]
fn forked_writes_retain_both_stream_and_filesystem_requirements() {
    for requirements in ["Stdin", "FileSystem"] {
        let source = source(
            "{stdin:Stdin}",
            requirements,
            "streams.concat source source",
        );
        let error = compile_module(CompileInput::new(
            "fork-write.ssrg",
            "fixture/fork-write",
            &source,
        ))
        .expect_err("fork must retain the stream and filesystem requirements");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message_key == "effect.explicit-environment-mismatch"),
            "{source}\n{error:#?}"
        );
    }
}
