//! CLI-produced program WASM files join the ordinary transactional Web build.
use crate::artifact::ArtifactProgramWasm;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct ProgramWasmAssets {
    pub name: String,
    pub files: BTreeMap<String, Vec<u8>>,
    pub inputs: Vec<(PathBuf, Vec<u8>)>,
}

pub(crate) fn check_inputs(programs: &[ProgramWasmAssets]) -> Result<(), String> {
    for program in programs {
        for (path, bytes) in &program.inputs {
            if fs::read(path).map_err(|e| e.to_string())? != *bytes {
                return Err(format!(
                    "source changed during program WASM build: {}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn stage(
    directory: &Path,
    programs: &[ProgramWasmAssets],
) -> Result<Vec<ArtifactProgramWasm>, String> {
    if programs.is_empty() {
        return Ok(Vec::new());
    }
    // A copied host dependency must not shadow the generated virtual package.
    fn reject_shadow(directory: &Path) -> Result<(), String> {
        if directory.join("node_modules/@seseragi/programs").exists() {
            return Err("host input collides with reserved @seseragi/programs package".into());
        }
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                reject_shadow(&entry.path())?;
            }
        }
        Ok(())
    }
    reject_shadow(directory)?;
    let host = directory.join("node_modules/@seseragi/programs");
    fs::create_dir_all(&host).map_err(|e| e.to_string())?;
    let mut exports = serde_json::Map::new();
    let mut result = Vec::new();
    for program in programs {
        if !seseragi_project::valid_program_name(&program.name)
            || exports.contains_key(&format!("./{}", program.name))
        {
            return Err("invalid or duplicate generated program name".into());
        }
        let names = [
            "program.rs",
            "program.wasm",
            "program.mjs",
            "program.d.mts",
            "program-wasm.json",
        ];
        if program.files.len() != names.len()
            || names.iter().any(|name| !program.files.contains_key(*name))
        {
            return Err("incomplete program WASM file set".into());
        }
        let metadata: serde_json::Value =
            serde_json::from_slice(&program.files["program-wasm.json"])
                .map_err(|e| e.to_string())?;
        if metadata["schema"] != 1
            || !matches!(
                metadata["abi"].as_str(),
                Some(
                    "seseragi-program-wasm-pure-v1"
                        | "seseragi-program-wasm-effect-v1"
                        | "seseragi-program-wasm-host-effect-v1"
                )
            )
        {
            return Err("unsupported program WASM asset ABI".into());
        }
        let inventory = metadata["files"]
            .as_array()
            .filter(|v| v.len() == 4)
            .ok_or("incomplete program WASM receipt")?;
        for name in &names[..4] {
            let bytes = &program.files[*name];
            if inventory
                .iter()
                .filter(|item| {
                    item["path"] == *name
                        && item["sha256"] == format!("{:x}", Sha256::digest(bytes))
                        && item["bytes"] == bytes.len()
                })
                .count()
                != 1
            {
                return Err(format!("program WASM receipt mismatch: {name}"));
            }
        }
        let base = format!("programs/{}", program.name);
        let public = directory.join(&base);
        fs::create_dir_all(public.parent().expect("fixed program prefix"))
            .map_err(|e| e.to_string())?;
        fs::create_dir(&public).map_err(|e| format!("program WASM output collision: {e}"))?;
        let private = host.join(&program.name);
        fs::create_dir(&private).map_err(|e| e.to_string())?;
        for (name, bytes) in &program.files {
            fs::write(public.join(name), bytes).map_err(|e| e.to_string())?;
            // Only JavaScript and declarations belong in the bundler package.
            if matches!(name.as_str(), "program.mjs" | "program.d.mts") {
                fs::write(private.join(name), bytes).map_err(|e| e.to_string())?;
            }
        }
        let wasm_path = format!("./{base}/program.wasm");
        let value = serde_json::to_string(&wasm_path).map_err(|e| e.to_string())?;
        let javascript = format!(
            "export * from \"./program.mjs\";\nimport {{ loadProgram as instantiate, ProgramWasmClosed }} from \"./program.mjs\";\nimport {{ loadOwnedProgram }} from \"../__lifecycle.mjs\";\nexport function loadProgram(bytes) {{ return loadOwnedProgram(() => instantiate(bytes), () => new ProgramWasmClosed(\"Browser program lifetime closed\")); }}\nexport const programWasmPath = {value};\n"
        );
        let types = format!(
            "export * from \"./program.mjs\";\nexport declare const programWasmPath: {value};\n"
        );
        for target in [&private, &public] {
            fs::write(target.join("index.mjs"), &javascript).map_err(|e| e.to_string())?;
            fs::write(target.join("index.d.mts"), &types).map_err(|e| e.to_string())?;
        }
        exports.insert(format!("./{}", program.name), serde_json::json!({"types":format!("./{}/index.d.mts",program.name),"default":format!("./{}/index.mjs",program.name)}));
        result.push(ArtifactProgramWasm {
            name: program.name.clone(),
            module: metadata["module"]
                .as_str()
                .ok_or("program receipt omitted canonical module")?
                .to_owned(),
            abi: metadata["abi"].as_str().unwrap().to_owned(),
            host_import: format!("@seseragi/programs/{}", program.name),
            manifest: format!("{base}/program-wasm.json"),
            wasm: format!("{base}/program.wasm"),
            loader: format!("{base}/index.mjs"),
            declarations: format!("{base}/index.d.mts"),
        });
    }
    let lifecycle = include_str!("../../../runtime/ts/src/browser/program-lifecycle.js");
    fs::write(host.join("__lifecycle.mjs"), lifecycle).map_err(|e| e.to_string())?;
    fs::write(directory.join("programs/__lifecycle.mjs"), lifecycle).map_err(|e| e.to_string())?;
    let lifecycle_types = include_str!("../../../runtime/ts/src/browser/program-lifecycle.d.ts");
    fs::write(host.join("__lifecycle.d.mts"), lifecycle_types).map_err(|e| e.to_string())?;
    fs::write(
        directory.join("programs/__lifecycle.d.mts"),
        lifecycle_types,
    )
    .map_err(|e| e.to_string())?;
    exports.insert(
        "./__lifecycle".into(),
        serde_json::json!({"types":"./__lifecycle.d.mts","default":"./__lifecycle.mjs"}),
    );
    fs::write(
        host.join("package.json"),
        serde_json::to_vec(
            &serde_json::json!({"name":"@seseragi/programs","type":"module","exports":exports}),
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(result)
}
