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
        readonly_signal_parameters(declaration, resolved, &references_by_symbol, &mut facts);
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

fn complete_type(value: &TypedType) -> bool {
    match value {
        TypedType::Hole => false,
        TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
            arguments.iter().all(complete_type)
        }
        TypedType::Record { fields, .. } => {
            fields.iter().all(|field| complete_type(&field.type_ref))
        }
        TypedType::Tuple { elements } => elements.iter().all(complete_type),
        TypedType::Function { parameter, result } => {
            complete_type(parameter) && complete_type(result)
        }
        TypedType::RequirementMerge { operands } => operands.iter().all(complete_type),
    }
}

/// Prove the input capability consumed by this implementation, not the intent or
/// compatibility of arbitrary public consumers. Every resolved use must be a
/// direct canonical read; absence of a locally visible write is insufficient.
fn readonly_signal_parameters(
    declaration: &TypedDecl,
    resolved: &ResolvedModule,
    references: &BTreeMap<SymbolId, Vec<ByteSpan>>,
    facts: &mut Vec<Diagnostic>,
) {
    let (parameters, body, declaration_origin) = match declaration {
        TypedDecl::Fn {
            visibility: Visibility::Public,
            parameters,
            body,
            origin,
            ..
        }
        | TypedDecl::EffectFn {
            visibility: Visibility::Public,
            parameters,
            body,
            origin,
            ..
        } => (parameters, body, *origin),
        _ => return,
    };
    let eligible = parameters.iter().filter(|parameter| {
        matches!(parameter, crate::TypedParameter::Named { name, type_ref: TypedType::ExternalNamed { canonical, arguments, .. }, .. }
            if !name.starts_with('_') && canonical == "std/signal::MutableSignal"
            && arguments.len() == 1 && complete_type(&arguments[0]))
    }).collect::<Vec<_>>();
    if eligible.is_empty() {
        return;
    }
    // Index once per eligible function; duplicate/ambiguous spans stay unproven.
    let mut reference_targets = BTreeMap::new();
    for reference in &resolved.references {
        if reference.namespace != SymbolNamespace::Value {
            continue;
        }
        let target = if reference.candidates.is_empty() {
            reference.target
        } else {
            None
        };
        reference_targets
            .entry((reference.origin.start, reference.origin.end))
            .and_modify(|existing| *existing = None)
            .or_insert(target);
    }
    // The shared exhaustive walker covers every TypedExpr variant, but flattens
    // ancestry. Collect closure boundaries first so their reads cannot masquerade
    // as direct parameter consumption by the enclosing function.
    let mut captures = Vec::new();
    super::walk_expression(body, &mut |expression| match expression {
        TypedExpr::Lambda { origin, .. } => captures.push(*origin),
        TypedExpr::Block { statements, .. } => {
            for statement in statements {
                if let crate::TypedBlockStatement::Function { origin, .. } = statement {
                    captures.push(*origin);
                }
            }
        }
        _ => {}
    });
    for parameter in eligible {
        let crate::TypedParameter::Named {
            name,
            type_ref,
            origin,
        } = parameter
        else {
            continue;
        };
        let TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } = type_ref
        else {
            continue;
        };
        let symbols = resolved
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.kind == crate::SymbolKind::Parameter
                    && symbol.namespace == SymbolNamespace::Value
                    && symbol.origin == *origin
            })
            .collect::<Vec<_>>();
        let [symbol] = symbols.as_slice() else {
            continue;
        };
        let Some(uses) = references.get(&symbol.id) else {
            continue;
        };
        if uses.is_empty()
            || uses.iter().any(|range| {
                !(declaration_origin.start <= range.start && range.end <= declaration_origin.end)
                    || captures
                        .iter()
                        .any(|capture| capture.start <= range.start && range.end <= capture.end)
            })
        {
            continue;
        }
        let all_uses = uses
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<BTreeSet<_>>();
        let mut reads = BTreeMap::new();
        super::walk_expression(body, &mut |expression| {
            let TypedExpr::Call {
                callee,
                arguments: call_arguments,
                evidence,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                trait_dispatch,
                type_ref: result_type,
                origin: call_origin,
                ..
            } = expression
            else {
                return;
            };
            if callee != "std/signal::read"
                || !evidence.is_empty()
                || !deferred_evidence_parameters.is_empty()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
                || trait_dispatch.is_some()
            {
                return;
            }
            let [TypedExpr::Variable {
                type_ref: argument_type,
                origin: argument_origin,
                evidence: argument_evidence,
                ..
            }] = call_arguments.as_slice()
            else {
                return;
            };
            let TypedType::ExternalNamed {
                canonical: argument_canonical,
                arguments: argument_payload,
                ..
            } = argument_type
            else {
                return;
            };
            if argument_canonical != canonical
                || argument_payload != arguments
                || !argument_evidence.is_empty()
                || !crate::standard::standard_signal_read_call(argument_type)
                    .is_some_and(|read| &read.result == result_type)
            {
                return;
            }
            let span = (argument_origin.start, argument_origin.end);
            if reference_targets.get(&span) == Some(&Some(symbol.id)) {
                reads.insert(span, *call_origin);
            }
        });
        if reads.is_empty() || reads.keys().copied().collect::<BTreeSet<_>>() != all_uses {
            continue;
        }
        let mut fact = warning("SES-L0307", "lint.readonly-signal-parameter", *origin,
            format!("All {} resolved uses of public parameter `{name}` are direct canonical std/signal::read calls. Its {} input needs only Signal with the same payload; review the public callable contract before changing it.", all_uses.len(), super::render_type(type_ref)));
        fact.related.push(RelatedDiagnostic {
            message: "This public declaration exposes the callable contract; no automatic signature edit is offered.".to_owned(),
            primary: ByteRange { start: declaration_origin.start, end: declaration_origin.end },
        });
        for call in reads.values() {
            fact.related.push(RelatedDiagnostic {
                message: "Canonical std/signal::read consumes Signal<A>; the one-way MutableSignal<A> to Signal<A> coercion preserves the same payload and signal identity. Other owners may still mutate it.".to_owned(),
                primary: ByteRange { start: call.start, end: call.end },
            });
        }
        facts.push(fact);
    }
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
