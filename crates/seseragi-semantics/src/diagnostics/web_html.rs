use crate::{typed::TypedResolution, SymbolNamespace};
use seseragi_project::is_standard_void_html_tag;
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceBlockItem,
    SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem, SurfaceExpr, SurfaceImplMember,
    SurfaceRecordItem,
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
        collect_void_tag_call(function, argument, resolution, diagnostics);
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
        | SurfaceExpr::String { .. }
        | SurfaceExpr::Boolean { .. }
        | SurfaceExpr::Name { .. }
        | SurfaceExpr::Error { .. } => {}
    }
}

fn collect_void_tag_call(
    function: &SurfaceExpr,
    argument: &SurfaceExpr,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(tag_name) = html_tag_name(function, resolution) else {
        return;
    };
    if !is_standard_void_html_tag(&tag_name) {
        return;
    }
    let SurfaceExpr::Record { items, .. } = argument else {
        return;
    };
    for item in items {
        let SurfaceRecordItem::Field {
            name, name_span, ..
        } = item
        else {
            continue;
        };
        if name == "children" {
            diagnostics.push(void_children_diagnostic(&tag_name, *name_span));
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

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
