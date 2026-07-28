use crate::{typed::TypedResolution, SymbolNamespace};
use seseragi_project::standard_html_tag_props;
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticEdit, DiagnosticFix, DiagnosticSeverity,
    RelatedDiagnostic, SurfaceBlockItem, SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem,
    SurfaceExpr, SurfaceImplMember, SurfaceRecordItem,
};

pub(super) fn collect_html_diagnostics(
    declaration: &SurfaceDecl,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match declaration {
        SurfaceDecl::Let { body, .. }
        | SurfaceDecl::EffectFn { body, .. }
        | SurfaceDecl::Fn { body, .. }
        | SurfaceDecl::Operator { body, .. } => {
            if let Some(body) = body {
                visit_expression(body, resolution, diagnostics);
            }
        }
        SurfaceDecl::Trait { methods, .. } | SurfaceDecl::Instance { methods, .. } => {
            for method in methods {
                if let Some(body) = &method.body {
                    visit_expression(body, resolution, diagnostics);
                }
            }
        }
        SurfaceDecl::Impl { members, .. } => {
            for member in members {
                let body = match member {
                    SurfaceImplMember::Method { method, .. } => method.body.as_ref(),
                    SurfaceImplMember::Operator { body, .. } => body.as_ref(),
                };
                if let Some(body) = body {
                    visit_expression(body, resolution, diagnostics);
                }
            }
        }
        SurfaceDecl::Newtype { .. }
        | SurfaceDecl::Alias { .. }
        | SurfaceDecl::Type { .. }
        | SurfaceDecl::Struct { .. } => {}
    }
}

fn visit_expression(
    expression: &SurfaceExpr,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let SurfaceExpr::Application {
        function, argument, ..
    } = expression
    {
        collect_tag_call(function, argument, resolution, diagnostics);
    }

    match expression {
        SurfaceExpr::Template { parts, .. } => {
            for part in parts {
                if let seseragi_syntax::SurfaceTemplatePart::Interpolation { value, .. } = part {
                    visit_expression(value, resolution, diagnostics);
                }
            }
        }
        SurfaceExpr::Member { receiver, .. }
        | SurfaceExpr::Prefix {
            operand: receiver, ..
        }
        | SurfaceExpr::Lambda { body: receiver, .. }
        | SurfaceExpr::Grouped {
            value: receiver, ..
        } => visit_expression(receiver, resolution, diagnostics),
        SurfaceExpr::Application {
            function, argument, ..
        }
        | SurfaceExpr::Assignment {
            target: function,
            value: argument,
            ..
        }
        | SurfaceExpr::Binary {
            left: function,
            right: argument,
            ..
        } => {
            visit_expression(function, resolution, diagnostics);
            visit_expression(argument, resolution, diagnostics);
        }
        SurfaceExpr::EffectfulFor { source, body, .. } => {
            visit_expression(source, resolution, diagnostics);
            visit_expression(body, resolution, diagnostics);
        }
        SurfaceExpr::Tuple { elements, .. }
        | SurfaceExpr::Array { elements, .. }
        | SurfaceExpr::List { elements, .. } => {
            for element in elements {
                visit_expression(element, resolution, diagnostics);
            }
        }
        SurfaceExpr::Record { items, .. } | SurfaceExpr::Struct { items, .. } => {
            for item in items {
                visit_expression(item.value(), resolution, diagnostics);
            }
        }
        SurfaceExpr::ArrayComprehension {
            element, clauses, ..
        }
        | SurfaceExpr::ListComprehension {
            element, clauses, ..
        } => {
            visit_expression(element, resolution, diagnostics);
            for clause in clauses {
                match clause {
                    SurfaceComprehensionClause::Generator { source, .. } => {
                        visit_expression(source, resolution, diagnostics)
                    }
                    SurfaceComprehensionClause::Guard { condition, .. } => {
                        visit_expression(condition, resolution, diagnostics)
                    }
                }
            }
        }
        SurfaceExpr::InfixChain { first, steps, .. } => {
            visit_expression(first, resolution, diagnostics);
            for step in steps {
                visit_expression(&step.operand, resolution, diagnostics);
            }
        }
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visit_expression(condition, resolution, diagnostics);
            visit_expression(then_branch, resolution, diagnostics);
            visit_expression(else_branch, resolution, diagnostics);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            visit_expression(scrutinee, resolution, diagnostics);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    visit_expression(guard, resolution, diagnostics);
                }
                visit_expression(&arm.body, resolution, diagnostics);
            }
        }
        SurfaceExpr::Block { items, result, .. } => {
            for item in items {
                match item {
                    SurfaceBlockItem::Let { value, .. }
                    | SurfaceBlockItem::Function { value, .. } => {
                        visit_expression(value, resolution, diagnostics)
                    }
                }
            }
            visit_expression(result, resolution, diagnostics);
        }
        SurfaceExpr::Do { items, result, .. } => {
            for item in items {
                match item {
                    SurfaceDoItem::Bind { value, .. }
                    | SurfaceDoItem::Let { value, .. }
                    | SurfaceDoItem::Expression { value, .. } => {
                        visit_expression(value, resolution, diagnostics)
                    }
                }
            }
            if let Some(result) = result {
                visit_expression(result, resolution, diagnostics);
            }
        }
        SurfaceExpr::Unit { .. }
        | SurfaceExpr::Integer { .. }
        | SurfaceExpr::Float { .. }
        | SurfaceExpr::String { .. }
        | SurfaceExpr::Boolean { .. }
        | SurfaceExpr::Name { .. }
        | SurfaceExpr::Error { .. } => {}
    }
}

fn collect_tag_call(
    function: &SurfaceExpr,
    argument: &SurfaceExpr,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(tag_name) = html_tag_name(function, resolution) else {
        return;
    };
    let Some((tag, fields)) = standard_html_tag_props(&tag_name) else {
        return;
    };
    let SurfaceExpr::Record { items, span } = argument else {
        return;
    };
    let allowed = fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<Vec<_>>();
    let has_click = items.iter().any(|item| field_name(item) == Some("onClick"));

    for item in items {
        let SurfaceRecordItem::Field {
            name, name_span, ..
        } = item
        else {
            continue;
        };

        if tag.void_element && name == "children" {
            diagnostics.push(void_children_diagnostic(&tag_name, *name_span));
            continue;
        }
        if !allowed.iter().any(|allowed| allowed == name) {
            diagnostics.push(unknown_prop_diagnostic(
                &tag_name, name, *name_span, &allowed,
            ));
        }
        if matches!(
            name.as_str(),
            "preventClickDefault" | "stopClickPropagation"
        ) && !has_click
        {
            diagnostics.push(event_control_without_handler_diagnostic(
                &tag_name, name, *name_span,
            ));
        }
    }

    if items
        .iter()
        .any(|item| matches!(item, SurfaceRecordItem::Spread { .. }))
    {
        return;
    }
    for field in fields.iter().filter(|field| !field.optional) {
        if !items
            .iter()
            .any(|item| field_name(item) == Some(field.name.as_str()))
        {
            diagnostics.push(missing_required_prop_diagnostic(
                &tag_name,
                &field.name,
                *span,
            ));
        }
    }
}

fn html_tag_name(function: &SurfaceExpr, resolution: &TypedResolution<'_>) -> Option<String> {
    if let SurfaceExpr::Member {
        receiver, field, ..
    } = function
    {
        let SurfaceExpr::Name { name, .. } = receiver.as_ref() else {
            return None;
        };
        let is_html_namespace = resolution
            .resolved()
            .dependencies
            .iter()
            .filter(|dependency| dependency.specifier == "std/web/html")
            .flat_map(|dependency| &dependency.imports)
            .any(|import| {
                import.namespace == "namespace"
                    && import.local_name.as_deref() == Some(name.as_str())
            });
        return is_html_namespace.then(|| field.clone());
    }

    let SurfaceExpr::Name { name, span } = function else {
        return None;
    };
    if let Some(canonical) = resolution
        .target(*span, SymbolNamespace::Value)
        .and_then(|target| resolution.symbol(target))
        .and_then(|symbol| symbol.canonical.as_deref())
        .and_then(|canonical| canonical.strip_prefix("std/web/html::"))
    {
        return Some(canonical.to_owned());
    }
    resolution
        .resolved()
        .dependencies
        .iter()
        .filter(|dependency| dependency.specifier == "std/web/html")
        .flat_map(|dependency| &dependency.imports)
        .find(|import| {
            import.namespace == "value"
                && import.local_name.as_deref().unwrap_or(&import.name) == name
        })
        .map(|import| import.name.clone())
}

fn void_children_diagnostic(tag: &str, primary: ByteSpan) -> Diagnostic {
    Diagnostic {
        id: String::new(),
        code: "SES-T0701".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "web.html.void-children".to_owned(),
        primary: byte_range(primary),
        related: vec![RelatedDiagnostic {
            message: format!("`html.{tag}` is a void element and cannot receive `children`"),
            primary: byte_range(primary),
        }],
        fixes: Vec::new(),
    }
}

fn missing_required_prop_diagnostic(tag: &str, name: &str, primary: ByteSpan) -> Diagnostic {
    Diagnostic {
        id: String::new(),
        code: "SES-T0702".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "web.html.missing-required-prop".to_owned(),
        primary: byte_range(primary),
        related: vec![RelatedDiagnostic {
            message: format!("`html.{tag}` requires the `{name}` prop"),
            primary: byte_range(primary),
        }],
        fixes: Vec::new(),
    }
}

fn unknown_prop_diagnostic(
    tag: &str,
    name: &str,
    primary: ByteSpan,
    allowed: &[String],
) -> Diagnostic {
    let suggestion = closest(name, allowed);
    let message = suggestion.as_deref().map_or_else(
        || format!("`html.{tag}` has no standard prop `{name}`"),
        |suggestion| {
            format!("`html.{tag}` has no standard prop `{name}`; did you mean `{suggestion}`?")
        },
    );
    Diagnostic {
        id: String::new(),
        code: "SES-L0101".to_owned(),
        severity: DiagnosticSeverity::Warning,
        message_key: "web.html.unknown-prop".to_owned(),
        primary: byte_range(primary),
        related: vec![RelatedDiagnostic {
            message,
            primary: byte_range(primary),
        }],
        fixes: suggestion
            .map(|suggestion| DiagnosticFix {
                title: format!("Replace with `{suggestion}`"),
                edits: vec![DiagnosticEdit {
                    range: byte_range(primary),
                    replacement: suggestion,
                }],
            })
            .into_iter()
            .collect(),
    }
}

fn event_control_without_handler_diagnostic(
    tag: &str,
    name: &str,
    primary: ByteSpan,
) -> Diagnostic {
    Diagnostic {
        id: String::new(),
        code: "SES-L0101".to_owned(),
        severity: DiagnosticSeverity::Warning,
        message_key: "web.html.event-control-without-handler".to_owned(),
        primary: byte_range(primary),
        related: vec![RelatedDiagnostic {
            message: format!("`html.{tag}` prop `{name}` has no effect without `onClick`"),
            primary: byte_range(primary),
        }],
        fixes: Vec::new(),
    }
}

fn field_name(item: &SurfaceRecordItem) -> Option<&str> {
    match item {
        SurfaceRecordItem::Field { name, .. } => Some(name),
        SurfaceRecordItem::Spread { .. } => None,
    }
}

fn closest(requested: &str, candidates: &[String]) -> Option<String> {
    let max_distance = if requested.chars().count() >= 6 { 3 } else { 2 };
    candidates
        .iter()
        .map(|candidate| (edit_distance(requested, candidate), candidate))
        .filter(|(distance, _)| *distance <= max_distance)
        .min_by_key(|(distance, candidate)| (*distance, candidate.as_str()))
        .map(|(_, candidate)| candidate.clone())
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut previous = (0..=right.chars().count()).collect::<Vec<_>>();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_char) in right.chars().enumerate() {
            current.push(std::cmp::min(
                std::cmp::min(current[right_index] + 1, previous[right_index + 1] + 1),
                previous[right_index] + usize::from(left_char != right_char),
            ));
        }
        previous = current;
    }
    previous.last().copied().unwrap_or(left.chars().count())
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
