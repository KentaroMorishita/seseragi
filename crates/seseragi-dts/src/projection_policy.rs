//! Optional JSON policy artifacts apply after immutable declaration interpretation.
use crate::{ConversionDiagnostic, ConversionResult, DiagnosticSeverity, EntryConfig};
use seseragi_interop::{
    ProjectionEvidence, ProjectionPolicySet, SemanticClassification, SemanticCompatibilityReport,
    SymbolKind,
};

/// The adapter consumes checked std/json::Json data. It never changes raw ABI
/// signatures or silently installs conversion into a foreign function call.
pub fn apply_projection_policy(
    result: &mut ConversionResult,
    entry: &EntryConfig,
    source: &str,
) -> Result<(), String> {
    let path = entry
        .projection_policy
        .as_deref()
        .ok_or("entry has no projection_policy artifact path")?;
    let Some(graph) = result.semantic.as_ref() else {
        return Ok(());
    };
    let attempt = serde_json::from_str::<ProjectionPolicySet>(source)
        .map_err(|error| format!("invalid projection policy JSON: {error}"))
        .and_then(|set| seseragi_projection::generate_adapters(graph, &set));
    let adapters = match attempt {
        Ok(adapters) => adapters,
        Err(message) => {
            result.diagnostics.push(ConversionDiagnostic {
                code: "SES-F0105".into(),
                severity: DiagnosticSeverity::Error,
                message,
                file: path.to_owned(),
                start: 0,
                end: source.len(),
                symbol: None,
            });
            if let Some(report) = result.compatibility.as_mut() {
                let evidence = graph.symbols.keys().filter(|id| !graph.private_symbols.contains(*id)).map(|id| ProjectionEvidence {
                    symbol: id.clone(), classification: SemanticClassification::PolicyRequired,
                    rule: "projection-policy-rejected".into(), reason: "semantic policy is invalid/stale; no raw or semantic output was published".into(), policy: None,
                }).collect::<Vec<_>>();
                // Preserve prior raw projection reasons alongside the rejection.
                merge_evidence(report, graph, &evidence)?;
            }
            result.generated = None;
            return Ok(());
        }
    };
    let mut evidence = vec![];
    for applied in &adapters.applied_policies {
        let target = seseragi_interop::projection_node(graph, applied.type_ref).map(|(id, _)| id);
        for (identity, symbol) in graph.symbols.iter().filter(|(id, symbol)| {
            !graph.private_symbols.contains(*id) && symbol.kind == SymbolKind::Type
        }) {
            if target.is_some()
                && seseragi_interop::projection_node(graph, symbol.type_ref).map(|(id, _)| id)
                    == target
            {
                evidence.push(ProjectionEvidence { symbol: identity.clone(), classification: SemanticClassification::SafeBoundary,
                    rule: "applied-semantic-json-policy".into(),
                    reason: format!("{} is an explicit checked {} adapter ({:?}); raw foreign identity/call ABI remains separate",applied.function_name,applied.input_boundary,applied.mapping), policy: Some(applied.producer.clone()) });
            }
        }
    }
    if let Some(report) = result.compatibility.as_mut() {
        merge_evidence(report, graph, &evidence)?;
    }
    if let Some(generated) = result.generated.as_mut() {
        generated.adapters = Some(adapters);
    }
    refresh_compatibility_identity(result)
}

fn merge_evidence(
    report: &mut SemanticCompatibilityReport,
    graph: &seseragi_interop::SemanticGraph,
    evidence: &[ProjectionEvidence],
) -> Result<(), String> {
    let extra =
        SemanticCompatibilityReport::from_projection(graph, evidence).map_err(|e| e.to_string())?;
    let classes = [
        SemanticClassification::Direct,
        SemanticClassification::Normalized,
        SemanticClassification::OpaquePreserved,
        SemanticClassification::SafeBoundary,
        SemanticClassification::PolicyRequired,
        SemanticClassification::RuntimeAbiUnresolved,
        SemanticClassification::Uninterpreted,
    ];
    let rank = |class| {
        classes
            .iter()
            .position(|candidate| *candidate == class)
            .unwrap()
    };
    for (id, next) in extra.symbols {
        let current = report
            .symbols
            .get_mut(&id)
            .ok_or("missing compatibility public symbol")?;
        if let Some(next_class) = next.projection_classification {
            if current
                .projection_classification
                .is_none_or(|old| rank(next_class) > rank(old))
            {
                current.projection_classification = Some(next_class);
            }
            if rank(next_class) > rank(current.classification) {
                current.classification = next_class;
            }
            for reason in next.reasons.into_iter().filter(|reason| {
                evidence
                    .iter()
                    .any(|item| item.symbol == id && item.rule == reason.rule)
            }) {
                if !current.reasons.contains(&reason) {
                    current.reasons.push(reason);
                }
            }
        }
    }
    for count in &mut report.counts {
        count.symbols = report
            .symbols
            .iter()
            .filter(|(_, symbol)| symbol.classification == count.classification)
            .map(|(id, _)| id.clone())
            .collect();
        count.count = count.symbols.len();
    }
    Ok(())
}

/// Runtime linkage may add identity proof while preserving interpreted types and
/// existing projection outcomes. Keep CLI/report/WASM payload identity identical.
pub fn refresh_compatibility_identity(result: &mut ConversionResult) -> Result<(), String> {
    if let (Some(graph), Some(report)) = (result.semantic.as_ref(), result.compatibility.as_mut()) {
        report.graph_digest = graph.digest().map_err(|e| e.to_string())?;
        report.identity = graph.identity.clone();
        if let Some(generated) = result.generated.as_mut() {
            let mut artifact: serde_json::Value =
                serde_json::from_str(&generated.report).map_err(|e| e.to_string())?;
            artifact["compatibility"] = serde_json::to_value(report).map_err(|e| e.to_string())?;
            generated.report = format!(
                "{}\n",
                serde_json::to_string_pretty(&artifact).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(())
}
