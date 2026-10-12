//! Proven authoring facts collected once from resolved identity and typed HIR.
use super::*;
use seseragi_syntax::Visibility;

fn warning(code: &str, key: &str, range: ByteSpan, reason: String) -> Diagnostic {
    let primary = ByteRange {
        start: range.start,
        end: range.end,
    };
    Diagnostic {
        id: String::new(),
        code: code.to_owned(),
        severity: DiagnosticSeverity::Warning,
        message_key: key.to_owned(),
        primary,
        related: vec![RelatedDiagnostic {
            message: reason,
            primary,
        }],
        fixes: Vec::new(),
        type_difference: None,
    }
}

pub(super) fn collect(resolved: &ResolvedModule, typed: &TypedModule) -> Vec<Diagnostic> {
    let mut facts = Vec::new();
    let used = resolved
        .references
        .iter()
        .filter_map(|r| r.target)
        .collect::<BTreeSet<_>>();
    let used_imports = resolved
        .imports
        .iter()
        .filter(|binding| used.contains(&binding.symbol))
        .map(|binding| {
            (
                binding.origin.start,
                binding.origin.end,
                binding.specifier.as_str(),
                binding.local_name.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    let mut symbols_by_identity: BTreeMap<&str, Vec<&crate::ResolvedSymbol>> = BTreeMap::new();
    for symbol in &resolved.symbols {
        if let Some(identity) = symbol.canonical.as_deref() {
            symbols_by_identity
                .entry(identity)
                .or_default()
                .push(symbol);
        }
    }
    let mut references_by_symbol: BTreeMap<SymbolId, Vec<ByteSpan>> = BTreeMap::new();
    for reference in &resolved.references {
        if let Some(target) = reference.target {
            references_by_symbol
                .entry(target)
                .or_default()
                .push(reference.origin);
        }
    }
    let mut warned_imports = BTreeSet::new();
    for import in &resolved.imports {
        let Some(symbol) = resolved.symbols.get(import.symbol.0 as usize) else {
            continue;
        };
        // Trait/operator evidence and retained inherent members can be used without
        // an occurrence of their spelling. Reexports are externally observable.
        if import.in_scope
            && import.member_owner.is_none()
            && import.reexported_as.is_none()
            && matches!(
                symbol.namespace,
                SymbolNamespace::Value | SymbolNamespace::Type | SymbolNamespace::Module
            )
            && !symbol.spelling.starts_with('_')
            && import.origin.start < import.origin.end
            && !used_imports.contains(&(
                import.origin.start,
                import.origin.end,
                import.specifier.as_str(),
                import.local_name.as_str(),
            ))
            && warned_imports.insert((
                import.origin.start,
                import.origin.end,
                import.specifier.as_str(),
                import.local_name.as_str(),
            ))
        {
            facts.push(warning("SES-L0302", "lint.unused-import", import.origin,
                format!("`{}` has no resolved references; removing its module load is not automatically safe", import.local_name)));
        }
    }
    for declaration in &typed.declarations {
        let (identities, visibility, origin, body) = match declaration {
            TypedDecl::Fn {
                symbol,
                visibility,
                origin,
                body,
                ..
            }
            | TypedDecl::EffectFn {
                symbol,
                visibility,
                origin,
                body,
                ..
            } => (vec![symbol.as_str()], *visibility, *origin, Some(body)),
            TypedDecl::Let {
                bindings,
                visibility,
                origin,
                value,
                ..
            } => (
                bindings.iter().map(|b| b.symbol.as_str()).collect(),
                *visibility,
                *origin,
                Some(value),
            ),
            TypedDecl::Alias {
                symbol,
                visibility,
                origin,
                ..
            }
            | TypedDecl::Struct {
                symbol,
                visibility,
                origin,
                ..
            } => (vec![symbol.as_str()], *visibility, *origin, None),
            TypedDecl::Adt {
                symbol,
                variants,
                visibility,
                origin,
                ..
            } => {
                let mut identities = vec![symbol.as_str()];
                identities.extend(variants.iter().map(|v| v.symbol.as_str()));
                (identities, *visibility, *origin, None)
            }
        };
        let symbols = identities
            .iter()
            .filter_map(|identity| symbols_by_identity.get(identity))
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        let prior_facts = facts.len();
        if let TypedDecl::Let {
            pattern,
            value,
            visibility: Visibility::Private,
            ..
        } = declaration
        {
            discarded_effect(pattern, value, &used, &mut facts);
        }
        let has_specific_discard = facts.len() > prior_facts;
        if visibility == Visibility::Private
            && !has_specific_discard
            && !symbols.is_empty()
            && symbols
                .iter()
                .all(|s| s.spelling != "main" && !s.spelling.starts_with('_'))
            && !symbols.iter().any(|symbol| {
                references_by_symbol.get(&symbol.id).is_some_and(|ranges| {
                    ranges
                        .iter()
                        .any(|range| !(origin.start <= range.start && range.end <= origin.end))
                })
            })
        {
            let symbol = symbols[0];
            facts.push(warning("SES-L0303", "lint.unused-private-declaration", symbol.origin,
                format!("`{}` is private and has no resolved references outside its own declaration; initialization may still be observable", symbol.spelling)));
        }
        if let Some(body) = body {
            super::walk_expression(body, &mut |expr| {
                inspect_expression(expr, &used, &mut facts)
            });
        }
    }
    // Instance methods participate in the same analysis, without a second LSP rule engine.
    for instance in &typed.instances {
        if let crate::TypedInstanceImplementation::UserDefined { methods } =
            &instance.implementation
        {
            for method in methods {
                super::walk_expression(&method.body, &mut |expr| {
                    inspect_expression(expr, &used, &mut facts)
                });
            }
        }
    }
    facts
}

fn is_discarded(pattern: &TypedPattern, used: &BTreeSet<SymbolId>) -> bool {
    match pattern {
        TypedPattern::Wildcard { .. } => true,
        TypedPattern::Binding { symbol, .. } => !used.contains(symbol),
        _ => false, // Partial destructuring does not prove the whole value discarded.
    }
}

fn discarded_effect(
    pattern: &TypedPattern,
    value: &TypedExpr,
    used: &BTreeSet<SymbolId>,
    facts: &mut Vec<Diagnostic>,
) {
    if is_discarded(pattern, used)
        && matches!(super::expression_type(value), Some(TypedType::Named { name, arguments }) if name == "Effect" && arguments.len() == 3)
    {
        let mut fact = warning("SES-L0304", "lint.discarded-effect", super::expression_origin(value),
            "This cold Effect value is never used; constructing it does not execute the operation. Bind or return it explicitly.".to_owned());
        if let Some((range, _)) = super::pattern_type(pattern) {
            fact.related.push(RelatedDiagnostic {
                message: "This binding does not retain the Effect for use".to_owned(),
                primary: ByteRange {
                    start: range.start,
                    end: range.end,
                },
            });
        }
        facts.push(fact);
    }
}

fn owns_subscription(value: &TypedExpr) -> bool {
    matches!(super::expression_type(value), Some(TypedType::Named { name, arguments })
        if name == "Effect" && arguments.len() == 3
        && matches!(&arguments[2], TypedType::ExternalNamed { canonical, .. } if canonical == "std/signal::Subscription"))
}

fn inspect_expression(expr: &TypedExpr, used: &BTreeSet<SymbolId>, facts: &mut Vec<Diagnostic>) {
    match expr {
        TypedExpr::Block { statements, .. } => {
            for stmt in statements {
                if let TypedBlockStatement::Let { pattern, value, .. } = stmt {
                    discarded_effect(pattern, value, used, facts);
                }
            }
        }
        TypedExpr::DoBlock { statements, .. } => {
            for stmt in statements {
                match stmt {
                    TypedDoStatement::PureLet { pattern, value, .. } => {
                        discarded_effect(pattern, value, used, facts)
                    }
                    TypedDoStatement::Effect { value } => {
                        // Only canonical standard operations with known ownership; a
                        // user type called Subscription is not ownership evidence.
                        if owns_subscription(value) {
                            facts.push(warning("SES-L0305", "lint.discarded-resource", super::expression_origin(value),
                                "The canonical signal subscription is acquired and discarded; retain its handle and call unsubscribe.".to_owned()));
                        }
                    }
                    TypedDoStatement::Bind { pattern, value, .. }
                        if is_discarded(pattern, used) =>
                    {
                        if owns_subscription(value) {
                            let mut fact = warning("SES-L0305", "lint.discarded-resource", super::expression_origin(value),
                                "The canonical signal subscription handle has no resolved uses; retain it for unsubscribe.".to_owned());
                            if let Some((range, _)) = super::pattern_type(pattern) {
                                fact.related.push(RelatedDiagnostic {
                                    message:
                                        "This binding does not retain the resource handle for use"
                                            .to_owned(),
                                    primary: ByteRange {
                                        start: range.start,
                                        end: range.end,
                                    },
                                });
                            }
                            facts.push(fact);
                        }
                    }
                    _ => {}
                }
            }
        }
        TypedExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            if let TypedExpr::Boolean { value, origin, .. } = condition.as_ref() {
                let unreachable = if *value { else_branch } else { then_branch };
                let mut diagnostic = warning(
                    "SES-L0306",
                    "lint.constant-condition",
                    *origin,
                    format!(
                        "The typed Bool literal is {value}; the other branch cannot be selected."
                    ),
                );
                let range = super::expression_origin(unreachable);
                diagnostic.related.push(RelatedDiagnostic {
                    message: "This branch cannot be selected".to_owned(),
                    primary: ByteRange {
                        start: range.start,
                        end: range.end,
                    },
                });
                facts.push(diagnostic);
            }
        }
        _ => {}
    }
}
