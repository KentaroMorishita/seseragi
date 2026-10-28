use pulldown_cmark::{html, BrokenLink, CowStr, Event, Options, Parser, Tag};
use serde_json::{json, Value};
use seseragi_driver::{CompiledProject, TypeDocument, TypeRenderOptions, TypeSchemeDocument};
use seseragi_project::{logical_module_id, LoadedLocalDocuments};
use seseragi_syntax::{
    extract_document_comments, ByteSpan, Deprecation, DocumentComment, DocumentComments,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

struct Source<'a> {
    path: String,
    text: &'a str,
    comments: DocumentComments,
}

struct Item {
    module: String,
    scope: &'static str,
    identity: String,
    namespace: String,
    name: String,
    display_name: String,
    owner: Option<String>,
    kind: String,
    signature: String,
    source_module: String,
    source_kind: &'static str,
    origin: ByteSpan,
    comment: Option<DocumentComment>,
    deprecated: Option<Deprecation>,
    metadata: Value,
}

impl Item {
    fn anchor(&self) -> String {
        format!(
            "api-{}",
            digest(&format!(
                "{}\0{}\0{}\0{}\0{}",
                self.scope, self.module, self.namespace, self.identity, self.display_name
            ))
        )
    }
}

pub(super) fn generate(
    project: &LoadedLocalDocuments,
    compiled: &CompiledProject,
    include_private: bool,
) -> Result<BTreeMap<String, String>, String> {
    let manifest = project
        .packages()
        .package(project.packages().root())
        .unwrap()
        .manifest();
    let public_modules = manifest
        .exports
        .values()
        .map(|path| path.as_str())
        .collect::<BTreeSet<_>>();
    let sources = project
        .modules()
        .map(|(identity, module)| {
            let id = logical_module_id(identity);
            (
                id,
                Source {
                    path: module.source_path().to_string_lossy().into_owned(),
                    text: module.source(),
                    comments: extract_document_comments(
                        module.source_path().to_string_lossy(),
                        module.source(),
                    ),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    // A method's interface origin belongs to its defining owner, including a
    // reexported nominal owner. Keep that source separate from the facade's
    // own import declaration; offsets are never applied to the wrong file.
    let defining_modules = compiled
        .modules
        .iter()
        .flat_map(|(id, module)| {
            let declarations = module
                .typed_interface
                .exports
                .iter()
                .filter(move |export| export.symbol.starts_with(&format!("{id}::")))
                .map(move |export| (export.symbol.clone(), id.clone()));
            let namespaces = module
                .typed_interface
                .namespace_deprecations
                .iter()
                .filter(move |namespace| namespace.symbol.starts_with(&format!("{id}::")))
                .map(move |namespace| (namespace.symbol.clone(), id.clone()));
            declarations.chain(namespaces)
        })
        .collect::<BTreeMap<_, _>>();
    let analysis = if include_private {
        let (graph, inputs) = super::document_inputs(project, None);
        Some(
            seseragi_driver::analyze_project(graph, inputs).map_err(|error| {
                format!("documentation private-scope analysis failed: {error:?}")
            })?,
        )
    } else {
        None
    };
    let mut items = Vec::new();
    let mut modules = Vec::new();
    let mut warnings = Vec::new();
    for identity in project.roots() {
        let module_id = logical_module_id(identity);
        let public = public_modules.contains(identity.path().as_str());
        if !public && !include_private {
            continue;
        }
        let scope = if public { "public" } else { "private" };
        let source = &sources[&module_id];
        let interface = &compiled.modules[&module_id].typed_interface;
        for range in &source.comments.unattached {
            warnings.push(warning(
                &module_id,
                *range,
                "document comment is not attached to a declaration or the module header",
            ));
        }
        for namespace in &interface.namespace_deprecations {
            let definition = defining_modules.get(&namespace.symbol);
            let source_module = definition.unwrap_or(&module_id);
            let origin = if definition.is_some() {
                namespace.declaration
            } else {
                namespace.publication.unwrap_or(namespace.declaration)
            };
            items.push(Item {
                module: module_id.clone(),
                scope,
                identity: namespace.symbol.clone(),
                namespace: "namespace".to_owned(),
                name: namespace.name.clone(),
                display_name: namespace.name.clone(),
                owner: None,
                kind: "foreign-namespace".to_owned(),
                signature: format!("namespace {}", namespace.name),
                source_module: source_module.clone(),
                origin,
                source_kind: if definition.is_some() {
                    "declaration"
                } else {
                    "reexport"
                },
                comment: attached_comment(&sources[source_module], origin),
                deprecated: Some(namespace.deprecation.clone()),
                metadata: serde_json::to_value(namespace).map_err(|error| error.to_string())?,
            });
        }
        for operator in &interface.operator_deprecations {
            let source = sources.get(&operator.provider_module).ok_or_else(|| {
                format!(
                    "documentation source for operator {} from {} is unavailable",
                    operator.spelling, operator.provider_module
                )
            })?;
            items.push(Item {
                module: module_id.clone(),
                scope,
                identity: operator.symbol(),
                namespace: "operator-implementation".to_owned(),
                name: operator.spelling.clone(),
                display_name: format!("{} ({})", operator.spelling, operator.instance),
                owner: Some(operator.instance.clone()),
                kind: "inherent-operator".to_owned(),
                signature: format!("operator {}", operator.spelling),
                source_module: operator.provider_module.clone(),
                source_kind: "declaration",
                origin: operator.declaration,
                comment: attached_comment(source, operator.declaration),
                deprecated: Some(operator.deprecation.clone()),
                metadata: serde_json::to_value(operator).map_err(|error| error.to_string())?,
            });
        }
        for export in &interface.exports {
            let scheme = TypeSchemeDocument::from_interface_scheme(&export.scheme)
                .render(TypeRenderOptions::default());
            let representation = export.representation.as_ref().map(|ty| {
                TypeDocument::from_interface_type(ty).render(TypeRenderOptions::default())
            });
            let kind = export
                .declaration_kind
                .as_deref()
                .unwrap_or(&export.namespace);
            let signature = format!(
                "{kind} {}: {scheme}{}",
                export.name,
                representation.map_or_else(String::new, |value| format!(" = {value}"))
            );
            items.push(Item {
                module: module_id.clone(),
                scope,
                identity: export.symbol.clone(),
                namespace: export.namespace.clone(),
                name: export.name.clone(),
                display_name: export.name.clone(),
                owner: None,
                kind: kind.to_owned(),
                signature,
                source_module: module_id.clone(),
                origin: export.declaration,
                source_kind: if export.symbol.starts_with(&format!("{module_id}::")) {
                    "declaration"
                } else {
                    "reexport"
                },
                comment: attached_comment(source, export.declaration),
                deprecated: export.deprecation.clone(),
                metadata: serde_json::to_value(export).map_err(|error| error.to_string())?,
            });
            let method_definition = defining_modules.get(&export.symbol);
            let method_source = method_definition.unwrap_or(&module_id);
            let source = &sources[method_source];
            for method in &export.methods {
                items.push(Item {
                    module: module_id.clone(),
                    scope,
                    identity: format!("{}::{}", export.symbol, method.name),
                    namespace: "method".to_owned(),
                    name: method.name.clone(),
                    display_name: format!("{}.{}", export.name, method.name),
                    owner: Some(export.symbol.clone()),
                    kind: if export.namespace == "trait" {
                        "trait-method"
                    } else {
                        "inherent-method"
                    }
                    .to_owned(),
                    signature: format!(
                        "fn {}: {}",
                        method.name,
                        TypeSchemeDocument::from_interface_scheme(&method.scheme)
                            .render(TypeRenderOptions::default())
                    ),
                    source_module: method_source.clone(),
                    origin: if method_definition.is_some() {
                        method.origin
                    } else {
                        export.declaration
                    },
                    source_kind: if method_definition.is_some() {
                        "declaration"
                    } else {
                        "reexport"
                    },
                    comment: method_definition
                        .and_then(|_| attached_comment(source, method.origin)),
                    deprecated: method.deprecation.clone(),
                    metadata: serde_json::to_value(method).map_err(|error| error.to_string())?,
                });
            }
        }
        if let Some(analysis) = &analysis {
            let public_identities = items
                .iter()
                .filter(|item| item.module == module_id)
                .map(|item| item.identity.clone())
                .collect::<BTreeSet<_>>();
            for symbol in analysis.documents[&module_id].workspace_symbols() {
                if public_identities.contains(&symbol.identity) {
                    continue;
                }
                let comment = source
                    .comments
                    .comments
                    .iter()
                    .filter(|comment| {
                        comment.declaration.is_some_and(|span| {
                            span.start <= symbol.definition.start
                                && symbol.definition.end <= span.end
                        })
                    })
                    .min_by_key(|comment| {
                        let span = comment.declaration.unwrap();
                        span.end - span.start
                    })
                    .cloned();
                let signature = symbol
                    .callable
                    .as_ref()
                    .map(|callable| callable.signature.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{} {}{}",
                            symbol.kind,
                            symbol.name,
                            symbol
                                .type_name
                                .as_ref()
                                .map_or_else(String::new, |ty| format!(": {ty}"))
                        )
                    });
                items.push(Item {
                    module: module_id.clone(),
                    scope: "private",
                    identity: symbol.identity.clone(),
                    namespace: symbol.namespace.clone(),
                    name: symbol.name.clone(),
                    display_name: symbol.name.clone(),
                    owner: symbol.member_owner.clone(),
                    kind: symbol.kind.clone(),
                    signature,
                    source_module: module_id.clone(),
                    origin: comment
                        .as_ref()
                        .and_then(|comment| comment.declaration)
                        .unwrap_or(symbol.definition),
                    source_kind: "declaration",
                    comment,
                    deprecated: symbol.deprecation.clone(),
                    metadata: json!({"analysis": symbol, "typeDocument": symbol.type_document}),
                });
            }
        }
        let instances = interface
            .instances
            .iter()
            .filter(|instance| {
                instance
                    .provider_module
                    .as_ref()
                    .is_none_or(|provider| provider == &module_id)
            })
            .collect::<Vec<_>>();
        modules.push(json!({"identity": module_id, "path": identity.path().as_str(), "scope": scope, "instances": instances,
            "documentation": source.comments.comments.iter().filter(|comment| comment.declaration.is_none()).map(|comment| json!({"markdown": comment.markdown, "range": comment.comment_range})).collect::<Vec<_>>() }));
    }
    items.sort_by(|a, b| {
        (
            &a.module,
            a.scope,
            a.origin.start,
            &a.namespace,
            &a.display_name,
            &a.identity,
        )
            .cmp(&(
                &b.module,
                b.scope,
                b.origin.start,
                &b.namespace,
                &b.display_name,
                &b.identity,
            ))
    });
    let mut body = format!("<h1>{}</h1>\n", escape(manifest.package.name.as_str()));
    let mut records = Vec::new();
    let mut used_sources = BTreeSet::new();
    for module in &modules {
        let id = module["identity"].as_str().unwrap();
        let source = &sources[id];
        used_sources.insert(id.to_owned());
        body.push_str(&format!(
            "<section><h2>{} <small>{}</small></h2>\n",
            escape(module["path"].as_str().unwrap()),
            module["scope"].as_str().unwrap()
        ));
        for comment in source
            .comments
            .comments
            .iter()
            .filter(|comment| comment.declaration.is_none())
        {
            body.push_str(&markdown(
                comment,
                id,
                module["scope"].as_str().unwrap(),
                None,
                None,
                &items,
                compiled,
                &mut warnings,
            ));
        }
        for item in items.iter().filter(|item| item.module == id) {
            let source = &sources[&item.source_module];
            if item.origin.start > item.origin.end
                || item.origin.end > source.text.len()
                || !source.text.is_char_boundary(item.origin.start)
                || !source.text.is_char_boundary(item.origin.end)
            {
                return Err(format!(
                    "invalid documentation declaration provenance for {}",
                    item.identity
                ));
            }
            used_sources.insert(item.source_module.clone());
            let line = source.text.as_bytes()[..item.origin.start]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count()
                + 1;
            let source_link = format!("sources/{}.html#L{line}", digest(&item.source_module));
            let source_metadata = json!({"module": item.source_module, "path": source.path, "kind": item.source_kind, "start": item.origin.start, "end": item.origin.end, "line": line, "href": source_link});
            let mut record = json!({"identity": item.identity, "namespace": item.namespace, "name": item.name, "displayName": item.display_name, "module": item.module, "scope": item.scope, "kind": item.kind, "signature": item.signature, "href": format!("index.html#{}", item.anchor()), "source": source_metadata, "metadata": item.metadata});
            if let Some(owner) = &item.owner {
                record["owner"] = json!(owner);
            }
            body.push_str(&format!(
                "<article id=\"{}\"><h3>{}</h3>\n",
                item.anchor(),
                escape(&item.display_name)
            ));
            if let Some(notice) = &item.deprecated {
                let mut metadata = json!({"message": notice.message});
                body.push_str(&format!(
                    "<aside class=\"deprecated\" role=\"note\">Deprecated: {}",
                    escape(&notice.message)
                ));
                if let Some(since) = &notice.since {
                    metadata["since"] = json!(since);
                    body.push_str(&format!(" <span>Since {}</span>", escape(since)));
                }
                body.push_str("</aside>\n");
                record["deprecated"] = metadata;
                record["deprecationSource"] = json!(notice);
            }
            // The notice is immediately before the signature, independent of
            // document comment rendering and its optional Markdown content.
            body.push_str(&format!(
                "<pre class=\"signature\"><code>{}</code></pre>\n<a href=\"{}\">{} source</a>\n",
                escape(&item.signature),
                source_link,
                item.source_kind
            ));
            if let Some(comment) = &item.comment {
                record["documentation"] = json!(comment.markdown);
                record["documentationRange"] = json!(comment.comment_range);
                body.push_str(&markdown(
                    comment,
                    id,
                    item.scope,
                    Some(&item.namespace),
                    item.owner.as_deref(),
                    &items,
                    compiled,
                    &mut warnings,
                ));
            }
            body.push_str("</article>\n");
            records.push(record);
        }
        if !module["instances"].as_array().unwrap().is_empty() {
            body.push_str("<h3>Instances and deriving</h3>\n<pre><code>");
            for instance in module["instances"].as_array().unwrap() {
                let identity = instance["identity"]
                    .as_str()
                    .unwrap_or_else(|| instance["trait"].as_str().unwrap_or("instance"));
                body.push_str(&escape(identity));
                body.push('\n');
            }
            body.push_str("</code></pre>\n");
        }
        body.push_str("</section>\n");
    }
    warnings.sort_by(|a, b| {
        (
            a["module"].as_str(),
            a["start"].as_u64(),
            a["message"].as_str(),
        )
            .cmp(&(
                b["module"].as_str(),
                b["start"].as_u64(),
                b["message"].as_str(),
            ))
    });
    warnings.dedup();
    for warning in &warnings {
        eprintln!(
            "Warning: {}:{}..{}: {}",
            warning["module"].as_str().unwrap(),
            warning["start"],
            warning["end"],
            warning["message"].as_str().unwrap()
        );
    }
    let mut artifacts = BTreeMap::from([
        ("index.html".to_owned(), page(manifest.package.name.as_str(), &body)),
        ("search-index.json".to_owned(), serde_json::to_string_pretty(&json!({"schema": 1, "kind": "seseragi-api-document", "generator": {"name": "seseragi", "version": env!("CARGO_PKG_VERSION")}, "package": manifest.package.name.as_str(), "modules": modules, "symbols": records, "warnings": warnings})).map_err(|error| error.to_string())? + "\n"),
    ]);
    for id in used_sources {
        let source = &sources[&id];
        let mut body = String::from("<pre><code>");
        for (index, line) in source.text.split_inclusive('\n').enumerate() {
            body.push_str(&format!(
                "<span id=\"L{}\">{}</span>",
                index + 1,
                escape(line)
            ));
        }
        body.push_str("</code></pre>\n");
        artifacts.insert(
            format!("sources/{}.html", digest(&id)),
            page(&source.path, &body),
        );
    }
    Ok(artifacts)
}

fn attached_comment(source: &Source<'_>, origin: ByteSpan) -> Option<DocumentComment> {
    source
        .comments
        .comments
        .iter()
        .find(|comment| comment.declaration == Some(origin))
        .cloned()
}

fn warning(module: &str, range: ByteSpan, message: &str) -> Value {
    json!({"module": module, "start": range.start, "end": range.end, "severity": "Warning", "message": message})
}

#[allow(clippy::too_many_arguments)]
fn markdown(
    comment: &DocumentComment,
    module: &str,
    scope: &str,
    namespace: Option<&str>,
    owner: Option<&str>,
    items: &[Item],
    compiled: &CompiledProject,
    warnings: &mut Vec<Value>,
) -> String {
    let callback = |link: BrokenLink<'_>| -> Option<(CowStr<'_>, CowStr<'_>)> {
        let reference = link.reference.trim();
        let name = reference
            .strip_prefix('(')
            .and_then(|text| text.strip_suffix(')'))
            .unwrap_or(reference);
        let visible = |item: &&Item| scope == "private" || item.scope == "public";
        let mut candidates = items
            .iter()
            .filter(visible)
            .filter(|item| {
                item.module == module
                    && (item.display_name == name
                        || item.name == name
                            && (item.owner.is_none()
                                || item.owner.as_deref() == owner
                                || item.kind == "trait-method"))
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            if let Some((prefix, name)) = name.rsplit_once('.') {
                candidates = items
                    .iter()
                    .filter(visible)
                    .filter(|item| {
                        let path = compiled.modules[&item.module]
                            .typed_interface
                            .module
                            .rsplit("::")
                            .next()
                            .unwrap_or(&item.module);
                        (path == prefix || path.replace('/', ".") == prefix)
                            && item.name == name
                            && item.owner.is_none()
                    })
                    .collect();
            }
        }
        if candidates.is_empty() {
            let symbols = compiled.modules[module]
                .typed_interface
                .dependencies
                .iter()
                .flat_map(|dependency| &dependency.imports)
                .filter(|import| import.local_name.as_deref().unwrap_or(&import.name) == name)
                .map(|import| import.symbol.as_str())
                .collect::<BTreeSet<_>>();
            candidates = items
                .iter()
                .filter(visible)
                .filter(|item| symbols.contains(item.identity.as_str()))
                .collect();
        }
        if let Some(namespace) = namespace {
            let same_namespace = candidates
                .iter()
                .copied()
                .filter(|item| item.namespace == namespace)
                .collect::<Vec<_>>();
            if !same_namespace.is_empty() {
                candidates = same_namespace;
            }
        }
        candidates.sort_by_key(|item| item.anchor());
        candidates.dedup_by(|a, b| a.anchor() == b.anchor());
        if let [target] = candidates.as_slice() {
            Some((
                CowStr::from(format!("#{}", target.anchor())),
                CowStr::from(target.display_name.clone()),
            ))
        } else {
            warnings.push(warning(module, ByteSpan { start: comment.original_offset(link.span.start), end: comment.original_offset(link.span.end) }, &format!("code link [{reference}] is missing, ambiguous, private, or outside the published API")));
            None
        }
    };
    let parser = Parser::new_with_broken_link_callback(
        &comment.markdown,
        Options::ENABLE_TABLES,
        Some(callback),
    );
    let events = parser.map(|event| match event {
        Event::Html(value) | Event::InlineHtml(value) => Event::Text(value),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: safe_url(dest_url),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: safe_url(dest_url),
            title,
            id,
        }),
        event => event,
    });
    let mut output = String::new();
    html::push_html(&mut output, events);
    output
}

fn safe_url(url: CowStr<'_>) -> CowStr<'_> {
    let normalized = url
        .trim()
        .chars()
        .filter(|character| !character.is_ascii_control())
        .collect::<String>()
        .to_ascii_lowercase();
    if normalized.contains(':')
        && !["http:", "https:", "mailto:"]
            .iter()
            .any(|prefix| normalized.starts_with(prefix))
    {
        CowStr::from("")
    } else {
        url
    }
}

fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn page(title: &str, body: &str) -> String {
    format!("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src https: http:\"><title>{}</title><style>body{{font:16px system-ui;max-width:64rem;margin:2rem auto;padding:0 1rem}}pre{{white-space:pre-wrap}}article{{margin:2rem 0}}.deprecated{{border-left:3px solid #a60;padding:.6rem}}small{{font-size:.7em}}</style></head><body>\n{body}</body></html>\n", escape(title))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
