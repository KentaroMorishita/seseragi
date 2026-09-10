//! Compose bundler JS -> generated TS mappings with compiler TS -> Seseragi maps.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

#[derive(Clone, Debug, Default, PartialEq)]
struct Segment {
    column: i64,
    source: Option<i64>,
    line: i64,
    original_column: i64,
    name: Option<i64>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Map {
    version: u32,
    #[serde(default)]
    file: String,
    #[serde(default)]
    source_root: String,
    sources: Vec<String>,
    sources_content: Vec<String>,
    names: Vec<String>,
    mappings: String,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}
const DIGITS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
fn decode(text: &str) -> Result<Vec<Vec<Segment>>, String> {
    let (mut source, mut line, mut column, mut name) = (0, 0, 0, 0);
    text.split(';')
        .map(|row| {
            let mut generated = 0;
            row.split(',')
                .filter(|item| !item.is_empty())
                .map(|item| {
                    let mut values = Vec::new();
                    let (mut value, mut shift) = (0i64, 0u32);
                    for ch in item.bytes() {
                        let digit = DIGITS
                            .iter()
                            .position(|value| *value == ch)
                            .ok_or("invalid source-map VLQ")?
                            as i64;
                        if shift > 55 {
                            return Err("source-map VLQ overflow".to_owned());
                        }
                        value |= (digit & 31) << shift;
                        if digit & 32 == 0 {
                            values.push(if value & 1 == 0 {
                                value >> 1
                            } else {
                                -(value >> 1)
                            });
                            value = 0;
                            shift = 0;
                        } else {
                            shift += 5;
                        }
                    }
                    if shift != 0 || !matches!(values.len(), 1 | 4 | 5) {
                        return Err("invalid source-map segment".to_owned());
                    }
                    generated += values[0];
                    if values.len() == 1 {
                        return Ok(Segment {
                            column: generated,
                            ..Default::default()
                        });
                    }
                    source += values[1];
                    line += values[2];
                    column += values[3];
                    let mapped_name = if values.len() == 5 {
                        name += values[4];
                        Some(name)
                    } else {
                        None
                    };
                    if generated < 0 || source < 0 || line < 0 || column < 0 {
                        return Err("negative source-map coordinate".to_owned());
                    }
                    Ok(Segment {
                        column: generated,
                        source: Some(source),
                        line,
                        original_column: column,
                        name: mapped_name,
                    })
                })
                .collect()
        })
        .collect()
}
fn encode(rows: &[Vec<Segment>]) -> String {
    fn vlq(value: i64) -> String {
        let mut value = if value < 0 {
            ((-value) as u64) << 1 | 1
        } else {
            (value as u64) << 1
        };
        let mut text = String::new();
        loop {
            let mut digit = (value & 31) as usize;
            value >>= 5;
            if value != 0 {
                digit |= 32;
            }
            text.push(DIGITS[digit] as char);
            if value == 0 {
                break;
            }
        }
        text
    }
    let (mut source, mut line, mut column, mut name) = (0, 0, 0, 0);
    rows.iter()
        .map(|row| {
            let mut generated = 0;
            row.iter()
                .map(|segment| {
                    let mut text = vlq(segment.column - generated);
                    generated = segment.column;
                    if let Some(index) = segment.source {
                        text += &vlq(index - source);
                        source = index;
                        text += &vlq(segment.line - line);
                        line = segment.line;
                        text += &vlq(segment.original_column - column);
                        column = segment.original_column;
                        if let Some(index) = segment.name {
                            text += &vlq(index - name);
                            name = index;
                        }
                    }
                    text
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join(";")
}
fn logical_path(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                parts.push(value.to_str().ok_or("source path must be UTF-8")?)
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err("source map escapes artifact root".to_owned());
                }
            }
            _ => return Err("absolute source-map path".to_owned()),
        }
    }
    Ok(parts.join("/"))
}
fn compiler_maps(directory: &Path, maps: &mut BTreeMap<String, Map>) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            compiler_maps(&path, maps)?;
        } else if path.extension().is_some_and(|extension| extension == "map") {
            let map: Map =
                serde_json::from_slice(&fs::read(&path).map_err(|error| error.to_string())?)
                    .map_err(|error| error.to_string())?;
            if map.sources.len() == 1 && map.sources[0].starts_with("seseragi://") {
                maps.insert(map.file.trim_start_matches("./").to_owned(), map);
            }
        }
    }
    Ok(())
}
pub(crate) fn compose(directory: &Path, output: &Path) -> Result<(), String> {
    let path = directory.join(format!("{}.map", output.display()));
    let mut outer: Map =
        serde_json::from_slice(&fs::read(&path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let mut maps = BTreeMap::new();
    compiler_maps(&directory.join("dist"), &mut maps)?;
    let single = directory.join("main.ts.map");
    if single.exists() {
        let map: Map =
            serde_json::from_slice(&fs::read(single).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        maps.insert("main.ts".to_owned(), map);
    }
    let mut inner = BTreeMap::new();
    for index in 0..outer.sources.len() {
        let logical = logical_path(
            &output
                .parent()
                .unwrap_or(Path::new(""))
                .join(&outer.sources[index]),
        )?;
        if let Some(map) = maps.get(&logical) {
            outer.sources[index] = map.sources[0].clone();
            outer.sources_content[index] = map.sources_content[0].clone();
            inner.insert(index as i64, decode(&map.mappings)?);
        } else {
            outer.sources[index] = if let Some(runtime) = logical.strip_prefix("node_modules/") {
                format!("seseragi-runtime://{runtime}")
            } else {
                format!("seseragi-generated://{logical}")
            };
        }
    }
    let mut rows = decode(&outer.mappings)?;
    for row in &mut rows {
        for segment in row {
            if let Some(map) = segment.source.and_then(|source| inner.get(&source)) {
                if let Some(original) = map.get(segment.line as usize).and_then(|row| {
                    row.iter().rev().find(|item| {
                        item.column <= segment.original_column && item.source.is_some()
                    })
                }) {
                    segment.line = original.line;
                    segment.original_column = original.original_column;
                } else {
                    segment.source = None;
                    segment.name = None;
                }
            }
        }
    }
    outer.mappings = encode(&rows);
    outer.source_root = String::new();
    outer.file = output.file_name().unwrap().to_string_lossy().into_owned();
    fs::write(
        path,
        serde_json::to_vec(&outer).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vlq_round_trips_sparse_lines_and_negative_deltas() {
        let text = "AAAA,EAAC,CAAD;A;AACA;;ACDA";
        assert_eq!(encode(&decode(text).unwrap()), text);
        assert!(decode("!").is_err());
        assert!(decode("g").is_err());
    }
}

#[cfg(test)]
mod composition_tests {
    use super::*;
    #[test]
    fn composes_coordinates_and_preserves_unicode_source_content() {
        let directory =
            std::env::temp_dir().join(format!("seseragi-compose-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let inner = serde_json::json!({"version":3,"file":"main.ts","sources":["seseragi://main"],"sourcesContent":["// 雪\n\npub main"],"names":[],"mappings":"AAEI"});
        let outer = serde_json::json!({"version":3,"sources":["main.ts"],"sourcesContent":["generated"],"names":[],"mappings":"AAAA"});
        fs::write(
            directory.join("main.ts.map"),
            serde_json::to_vec(&inner).unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("entry.js.map"),
            serde_json::to_vec(&outer).unwrap(),
        )
        .unwrap();
        compose(&directory, Path::new("entry.js")).unwrap();
        let result: Map =
            serde_json::from_slice(&fs::read(directory.join("entry.js.map")).unwrap()).unwrap();
        let rows = decode(&result.mappings).unwrap();
        assert_eq!(rows[0][0].line, 2);
        assert_eq!(rows[0][0].original_column, 4);
        assert_eq!(result.sources, vec!["seseragi://main"]);
        assert!(result.sources_content[0].contains("雪"));
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn minification_preserves_same_compiled_program_result() {
        let directory =
            std::env::temp_dir().join(format!("seseragi-minify-parity-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "main.ssrg",
                "main",
                "pub effect fn main = println \"雪42\"\n",
            )
            .with_profile(seseragi_project::BuildProfile::Release),
        )
        .unwrap();
        let mut results = Vec::new();
        for minify in [false, true] {
            let output = directory.join(if minify { "minified" } else { "original" });
            crate::build_main_with_artifact_options(
                &compiled,
                &output,
                crate::BuildTarget::Process,
                crate::ProcessRunOptions::default(),
                crate::artifact::ArtifactOptions {
                    source_map: Some(crate::artifact::SourceMapPolicy::Emit),
                    minify: Some(minify),
                },
            )
            .unwrap();
            let result = std::process::Command::new("bun")
                .arg("entry.js")
                .current_dir(&output)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            results.push(result.stdout);
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[0], "雪42\n".as_bytes());
        fs::remove_dir_all(directory).unwrap();
    }
}
