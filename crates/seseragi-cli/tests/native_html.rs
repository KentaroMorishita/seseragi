use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

fn without_final_newline(value: &str) -> &str {
    value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix('\n'))
        .unwrap_or(value)
}

fn compare(source: &Path, expected: &Path, scratch: &Path) {
    let cli = env!("CARGO_BIN_EXE_seseragi");
    let ts = successful(bounded(Command::new(cli).arg("run").arg(source), scratch));
    let expected = fs::read_to_string(expected).unwrap();
    assert_eq!(
        without_final_newline(&String::from_utf8_lossy(&ts.stdout)),
        without_final_newline(&expected),
        "{} / TS",
        source.display()
    );
    compare_native(source, &ts.stdout, scratch);
}

fn compare_native(source: &Path, expected: &[u8], scratch: &Path) {
    let cli = env!("CARGO_BIN_EXE_seseragi");
    let native = scratch.join("native");
    successful(bounded(
        Command::new(cli)
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        scratch,
    ));
    let output = successful(bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        scratch,
    ));
    assert_eq!(output.stdout, expected, "{} / native", source.display());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
        "{}: {output:?}",
        source.display()
    );
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn style_image_and_document_match_ts() {
    let scratch = tempfile::tempdir().unwrap();
    let fixture = root().join("examples/spec/fixtures/rust-backend/html-style-image.ssrg");
    compare(&fixture, &fixture.with_extension("stdout"), scratch.path());
}

#[test]
fn every_standard_html_element_and_its_attributes_match_ts() {
    let scratch = tempfile::tempdir().unwrap();
    let mut source = String::from("import * as html from \"std/web/html\"\npub effect fn main = match html.parseWebUrl \"/image?a=1&b=2\" { Left error -> println (show error); Right url -> do {\n");
    for (index, tag) in seseragi_project::STANDARD_HTML_TAGS.iter().enumerate() {
        let attributes = match tag.name {
            "meta" => "charSet: \"utf-8\", name: \"description\", content: \"<&>\", httpEquiv: \"refresh\",",
            "link" => "rel: \"icon\", href: url,",
            "a" => "href: url, target: \"_blank\", rel: \"noopener\", download: True,",
            "img" => "src: url, alt: \"<&>\", width: 32, height: 24, loading: \"lazy\",",
            "source" => "src: url, media: \"screen\", mimeType: \"image/png\",",
            "video" => "src: url, width: 32, height: 24,",
            "audio" => "src: url,",
            "button" => "disabled: True, buttonType: \"submit\", name: \"save\", value: \"<&>\", autoFocus: True,",
            "form" => "name: \"form\", autoComplete: \"off\",",
            "label" => "htmlFor: \"field\",",
            "input" => "value: \"<&>\", checked: True, name: \"field\", disabled: True, required: True, readOnly: True, multiple: True, placeholder: \"<&>\", autoComplete: \"off\", autoFocus: True, min: \"1\", max: \"9\", step: \"2\", pattern: \"[0-9]\", inputType: \"number\",",
            "textarea" => "value: \"<&>\", name: \"field\", disabled: True, required: True, readOnly: True, placeholder: \"<&>\", autoComplete: \"off\", autoFocus: True, rows: 2, cols: 4,",
            "select" => "name: \"field\", value: \"<&>\", disabled: True, required: True, multiple: True, autoFocus: True,",
            "option" => "value: \"<&>\", selected: True, disabled: True,",
            "th" | "td" => "colSpan: 2, rowSpan: 3,",
            "details" | "dialog" => "open: True,",
            _ => "",
        };
        let children = if tag.void_element || tag.name == "textarea" {
            ""
        } else {
            "children: \"<&>\","
        };
        source.push_str(&format!("let node{index}: html.Html<Unit> = html.{} {{ id: \"id\", class: \"class\", title: \"<&>\\\"'\", hidden: True, style: html.style {{ color: \"red\" }}, role: \"group\", tabIndex: -1, lang: \"ja\", dir: \"ltr\", draggable: False, contentEditable: True, inert: True, ariaLabel: \"<&>\", ariaBusy: False, ariaExpanded: True, ariaHidden: False, ariaSelected: True, {attributes} {children} }}\nprintln (html.renderToString node{index})\n", tag.name));
    }
    source.push_str("} }\n");
    let input = scratch.path().join("elements.ssrg");
    fs::write(&input, source).unwrap();
    let ts = successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&input),
        scratch.path(),
    ));
    assert_eq!(
        ts.stdout.iter().filter(|byte| **byte == b'\n').count(),
        seseragi_project::STANDARD_HTML_TAGS.len()
    );
    compare_native(&input, &ts.stdout, scratch.path());
}

#[test]
fn image_urls_keep_ts_validation_and_original_spelling() {
    let scratch = tempfile::tempdir().unwrap();
    let urls = [
        "",
        "/images/a.png?a=1&b=2",
        "../a.png",
        "#image",
        "?q=photo",
        "https://example.com/image",
        "HTTP://example.com:80/image",
        "https:example.com",
        "http:/example.com",
        "//example.com/image",
        "mailto:hello@example.com",
        "tel:+123",
        "https://[::1]:443/a",
        "https://127.0.0.1/a",
        "https://0x7f.1/a",
        "https://%65xample.com/a",
        "https://example.com:65535/a",
        " https://example.com/a ",
        "https://@example.com/a",
        "https://:@example.com/a",
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "data:image/png;base64,a",
        "file:///tmp/a",
        "ftp://example.com/a",
        "https://user:pass@example.com/a",
        "//user@example.com/a",
        "https://example.com:65536/a",
        "https://example.com:+80/a",
        "http://",
        "https://[bad]/a",
        "https://exa mple.com/a",
        "https://%00example.com/a",
        "https://example.com/%00",
        "https://256.1.1.1/a",
        "https://1.2.3.999/a",
        "https://example.123/a",
        "https://example.com:abc/a",
        "mailto://user@example.com/a",
        "tel://user@example.com/a",
        "https://example.com/\nimage",
        "1invalid:value",
        "https:",
        "https:?q=1",
        "https:[bad",
        "https:javascript:alert(1)",
        "mailto://",
        "tel://256.1.1.1",
        "https://%22example.com/a",
        "https://\"example.com/a",
        "https://0xffffffffffffffffffff/a",
        "https://example.com../a",
        "https://example.com:00000000000000000080/a",
        "mailto://\\user@example.com/a",
        "mailto:////user@example.com/a",
    ];
    let mut source = String::from("import * as html from \"std/web/html\"\nfn allowed value: String -> Bool = match html.parseWebUrl value { Left _ -> False; Right _ -> True }\npub effect fn main = do {\n");
    let mut expected = String::new();
    let base = url::Url::parse("https://seseragi.invalid/").unwrap();
    for value in urls {
        source.push_str(&format!(
            "println (show (allowed {}))\n",
            serde_json::to_string(value).unwrap()
        ));
        let allowed = !value.chars().any(|c| c.is_ascii_control())
            && base.join(value).is_ok_and(|u| {
                matches!(u.scheme(), "http" | "https" | "mailto" | "tel")
                    && u.username().is_empty()
                    && u.password().is_none_or(|p| p.is_empty())
            });
        expected.push_str(if allowed { "True\n" } else { "False\n" });
    }
    source.push_str("}\n");
    let input = scratch.path().join("urls.ssrg");
    let output = scratch.path().join("urls.stdout");
    fs::write(&input, source).unwrap();
    fs::write(&output, expected).unwrap();
    compare(&input, &output, scratch.path());
}

fn imports_html(directory: &Path) -> bool {
    fs::read_dir(directory).unwrap().any(|entry| {
        let path = entry.unwrap().path();
        if path.is_dir() {
            imports_html(&path)
        } else {
            path.extension().is_some_and(|e| e == "ssrg")
                && fs::read_to_string(path).unwrap().contains("std/web/html")
        }
    })
}

#[test]
fn official_html_samples_and_tour_exercises_match_ts() {
    let scratch = tempfile::tempdir().unwrap();
    let root = root();
    let mut cases = Vec::new();
    for (directory, metadata_name) in [
        ("examples/samples", "sample.json"),
        ("examples/tour/lessons", "lesson.json"),
    ] {
        for entry in fs::read_dir(root.join(directory)).unwrap() {
            let path = entry.unwrap().path();
            let metadata = path.join(metadata_name);
            if !metadata.is_file() {
                continue;
            }
            let metadata: serde_json::Value =
                serde_json::from_slice(&fs::read(metadata).unwrap()).unwrap();
            let files = &metadata["files"];
            if metadata["interactive"] != true && imports_html(&path) {
                let source = if files["manifest"].is_string() {
                    path.clone()
                } else {
                    path.join(files["source"].as_str().unwrap())
                };
                cases.push((source, path.join(files["expectedOutput"].as_str().unwrap())));
            }
            if metadata_name == "lesson.json" {
                if let Some(exercise) = files["exercise"].as_str() {
                    let source = path.join(exercise);
                    if fs::read_to_string(&source)
                        .unwrap()
                        .contains("std/web/html")
                    {
                        cases.push((
                            source,
                            path.join(files["exerciseExpectedOutput"].as_str().unwrap()),
                        ));
                    }
                }
            }
        }
    }
    cases.sort();
    assert!(!cases.is_empty());
    for (source, expected) in &cases {
        compare(source, expected, scratch.path());
    }
    eprintln!("native HTML samples: {}/{}", cases.len(), cases.len());
}

#[test]
fn style_order_survives_branches_functions_and_overlapping_spreads() {
    let scratch = tempfile::tempdir().unwrap();
    let input = scratch.path().join("order.ssrg");
    fs::write(
        &input,
        r#"import * as html from "std/web/html"
alias Declarations = {
  color: String,
  zIndex: String,
  variables: { accentColor: String, secondColor: String },
  backgroundColor: String
}
fn choose reverse: Bool -> Declarations =
  if reverse then {
    color: "red", zIndex: "2",
    variables: { accentColor: "red", secondColor: "white" },
    backgroundColor: "black"
  } else {
    zIndex: "2", backgroundColor: "black", color: "red",
    variables: { secondColor: "white", accentColor: "red" }
  }
fn through declarations: Declarations -> Declarations = declarations
fn narrow value: { color: String } -> { color: String } = value
pub effect fn main = do {
  let convert: Declarations -> html.Style = html.style
  let first = through (choose True)
  let second = through (choose False)
  let variables = { ...second.variables, accentColor: "blue" }
  let updated = { ...second, ...first, color: "blue", variables: variables }
  let node: html.Html<Unit> = html.div { style: convert updated, children: "" }
  println (html.renderToString node)
  let other: html.Html<Unit> = html.div { style: convert first, children: "" }
  println (html.renderToString other)
  let narrower = { zIndex: "2", color: "red", variables: { accentColor: "red" } }
  let wider = { ...narrower, variables: { ...narrower.variables, secondColor: "white" }, backgroundColor: "black" }
  let third: html.Html<Unit> = html.div { style: convert wider, children: "" }
  println (html.renderToString third)
  let numeric = { zIndex: 2, color: 1 }
  let replaced = { ...numeric, color: "blue", zIndex: "2", variables: variables, backgroundColor: "black" }
  let fourth: html.Html<Unit> = html.div { style: convert replaced, children: "" }
  println (html.renderToString fourth)
  let wide = { zIndex: "2", extraColor: "white", color: "red" }
  let overridden = { ...narrow wide, color: "blue" }
  let fifth: html.Html<Unit> = html.div { style: html.style overridden, children: "" }
  println (html.renderToString fifth)
  let ordinary = { ...updated, x: 1 }
  println ordinary.color
  println (show ordinary)
  println (show (first == through (choose True)))
}
"#,
    )
    .unwrap();
    let ts = successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&input),
        scratch.path(),
    ));
    let text = String::from_utf8_lossy(&ts.stdout);
    assert!(text.starts_with("<div style=\"z-index: 2; background-color: black; color: blue; --second-color: white; --accent-color: blue\"></div>\n<div style=\"color: red; z-index: 2; --accent-color: red; --second-color: white; background-color: black\"></div>\n"), "{text}");
    compare_native(&input, &ts.stdout, scratch.path());
}

#[test]
fn style_flow_respects_local_function_and_closure_shadowing() {
    let scratch = tempfile::tempdir().unwrap();
    let input = scratch.path().join("shadow.ssrg");
    fs::write(
        &input,
        r#"import * as html from "std/web/html"
alias Declarations = { color: String, zIndex: String }
fn source -> Declarations = { zIndex: "2", color: "red" }
pub effect fn main = {
  let before = source ()
  fn source -> Declarations = { color: "blue", zIndex: "3" }
  let after = source ()
  let convert: Declarations -> html.Style = html.style
  let render: Declarations -> html.Style = {
    let convert: Declarations -> html.Style = \value -> convert value
    convert
  }
  let first: html.Html<Unit> = html.div { style: render before, children: "" }
  let second: html.Html<Unit> = html.div { style: render after, children: "" }
  do {
    println (html.renderToString first)
    println (html.renderToString second)
  }
}
"#,
    )
    .unwrap();
    let ts = successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&input),
        scratch.path(),
    ));
    assert_eq!(String::from_utf8_lossy(&ts.stdout), "<div style=\"z-index: 2; color: red\"></div>\n<div style=\"color: blue; z-index: 3\"></div>\n");
    compare_native(&input, &ts.stdout, scratch.path());
}
