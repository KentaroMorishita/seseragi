use crate::ResolveIssue;
use seseragi_syntax::{
    ByteSpan, SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem, SurfaceExpr,
    SurfaceImplMember, SurfaceInfixStep, SurfaceRecordItem, SurfaceTemplatePart,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Associativity {
    Left,
    Right,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fixity {
    rank: i32,
    associativity: Associativity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OperatorMeaning {
    Binary,
    Apply,
    Pipeline,
    TraitMethod {
        method: &'static str,
        method_operand_sources: [usize; 2],
    },
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OperatorSpec {
    fixity: Fixity,
    meaning: OperatorMeaning,
}

struct PendingOperator {
    spelling: String,
    span: ByteSpan,
    spec: OperatorSpec,
}

/// Resolves every flat infix chain after all local headers and dependency
/// interfaces are available. Imported fixities use the same table as local
/// declarations so linking never needs a second parser.
pub(super) fn normalize_operator_chains(
    declarations: &mut [SurfaceDecl],
    imported: impl IntoIterator<Item = (String, String, u32)>,
) -> Vec<ResolveIssue> {
    let mut issues = Vec::new();
    let mut custom = BTreeMap::new();

    for declaration in declarations.iter() {
        let SurfaceDecl::Operator {
            fixity,
            precedence,
            spelling,
            spelling_span,
            parameters,
            ..
        } = declaration
        else {
            continue;
        };
        register_custom_fixity(
            &mut custom,
            spelling,
            fixity,
            *precedence,
            parameters.len(),
            *spelling_span,
            &mut issues,
        );
    }
    for (spelling, fixity, precedence) in imported {
        // Linker diagnostics own imported declaration validity. A malformed
        // provider interface is ignored here rather than attributed to a
        // consumer source range that does not contain that declaration.
        if let Some(spec) = custom_operator_spec(&fixity, precedence) {
            custom.entry(spelling).or_insert(spec);
        }
    }

    for declaration in declarations {
        normalize_declaration(declaration, &custom, &mut issues);
    }
    issues
}

fn register_custom_fixity(
    custom: &mut BTreeMap<String, OperatorSpec>,
    spelling: &str,
    fixity: &str,
    precedence: u32,
    parameter_count: usize,
    origin: ByteSpan,
    issues: &mut Vec<ResolveIssue>,
) {
    if !seseragi_syntax::is_custom_operator_candidate(spelling) {
        issues.push(ResolveIssue {
            code: "SES-P0001".to_owned(),
            message_key: "operator.invalid-spelling".to_owned(),
            primary: origin,
        });
        return;
    }
    if is_reserved_operator(spelling) {
        issues.push(ResolveIssue {
            code: "SES-P0001".to_owned(),
            message_key: "operator.reserved-spelling".to_owned(),
            primary: origin,
        });
        return;
    }
    if parameter_count != 2 {
        issues.push(ResolveIssue {
            code: "SES-P0001".to_owned(),
            message_key: "operator.invalid-arity".to_owned(),
            primary: origin,
        });
        return;
    }
    let Some(spec) = custom_operator_spec(fixity, precedence) else {
        issues.push(ResolveIssue {
            code: "SES-P0102".to_owned(),
            message_key: "operator.invalid-fixity".to_owned(),
            primary: origin,
        });
        return;
    };
    custom.entry(spelling.to_owned()).or_insert(spec);
}

fn custom_operator_spec(fixity: &str, precedence: u32) -> Option<OperatorSpec> {
    if precedence > 8 {
        return None;
    }
    let associativity = match fixity {
        "infixl" => Associativity::Left,
        "infixr" => Associativity::Right,
        "infix" => Associativity::None,
        _ => return None,
    };
    Some(OperatorSpec {
        fixity: Fixity {
            rank: (precedence as i32) * 2,
            associativity,
        },
        meaning: OperatorMeaning::Custom,
    })
}

fn normalize_declaration(
    declaration: &mut SurfaceDecl,
    custom: &BTreeMap<String, OperatorSpec>,
    issues: &mut Vec<ResolveIssue>,
) {
    match declaration {
        SurfaceDecl::Let { body, .. }
        | SurfaceDecl::EffectFn { body, .. }
        | SurfaceDecl::Fn { body, .. }
        | SurfaceDecl::Operator { body, .. } => {
            if let Some(body) = body {
                normalize_expression(body, custom, issues);
            }
        }
        SurfaceDecl::Impl { members, .. } => {
            for member in members {
                match member {
                    SurfaceImplMember::Method { method, .. } => {
                        if let Some(body) = &mut method.body {
                            normalize_expression(body, custom, issues);
                        }
                    }
                    SurfaceImplMember::Operator { body, .. } => {
                        if let Some(body) = body {
                            normalize_expression(body, custom, issues);
                        }
                    }
                }
            }
        }
        SurfaceDecl::Instance { methods, .. } => {
            for method in methods {
                if let Some(body) = &mut method.body {
                    normalize_expression(body, custom, issues);
                }
            }
        }
        SurfaceDecl::Newtype { .. }
        | SurfaceDecl::Alias { .. }
        | SurfaceDecl::Type { .. }
        | SurfaceDecl::Struct { .. }
        | SurfaceDecl::Trait { .. } => {}
    }
}

fn normalize_expression(
    expression: &mut SurfaceExpr,
    custom: &BTreeMap<String, OperatorSpec>,
    issues: &mut Vec<ResolveIssue>,
) {
    match expression {
        SurfaceExpr::Template { parts, .. } => {
            for part in parts {
                if let SurfaceTemplatePart::Interpolation { value, .. } = part {
                    normalize_expression(value, custom, issues);
                }
            }
        }
        SurfaceExpr::Member { receiver, .. } => normalize_expression(receiver, custom, issues),
        SurfaceExpr::Application {
            function, argument, ..
        } => {
            normalize_expression(function, custom, issues);
            normalize_expression(argument, custom, issues);
        }
        SurfaceExpr::Prefix { operand, .. } => normalize_expression(operand, custom, issues),
        SurfaceExpr::Assignment { target, value, .. } => {
            normalize_expression(target, custom, issues);
            normalize_expression(value, custom, issues);
        }
        SurfaceExpr::Lambda { body, .. } => normalize_expression(body, custom, issues),
        SurfaceExpr::EffectfulFor { source, body, .. } => {
            normalize_expression(source, custom, issues);
            normalize_expression(body, custom, issues);
        }
        SurfaceExpr::Tuple { elements, .. }
        | SurfaceExpr::Array { elements, .. }
        | SurfaceExpr::List { elements, .. } => {
            for element in elements {
                normalize_expression(element, custom, issues);
            }
        }
        SurfaceExpr::Record { items, .. } | SurfaceExpr::Struct { items, .. } => {
            for item in items {
                match item {
                    SurfaceRecordItem::Field { value, .. }
                    | SurfaceRecordItem::Spread { value, .. } => {
                        normalize_expression(value, custom, issues);
                    }
                }
            }
        }
        SurfaceExpr::ArrayComprehension {
            element, clauses, ..
        }
        | SurfaceExpr::ListComprehension {
            element, clauses, ..
        } => {
            normalize_expression(element, custom, issues);
            for clause in clauses {
                match clause {
                    SurfaceComprehensionClause::Generator { source, .. } => {
                        normalize_expression(source, custom, issues);
                    }
                    SurfaceComprehensionClause::Guard { condition, .. } => {
                        normalize_expression(condition, custom, issues);
                    }
                }
            }
        }
        SurfaceExpr::Binary { left, right, .. } => {
            normalize_expression(left, custom, issues);
            normalize_expression(right, custom, issues);
        }
        SurfaceExpr::InfixChain { first, steps, .. } => {
            normalize_expression(first, custom, issues);
            for step in steps.iter_mut() {
                normalize_expression(&mut step.operand, custom, issues);
            }
        }
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            normalize_expression(condition, custom, issues);
            normalize_expression(then_branch, custom, issues);
            normalize_expression(else_branch, custom, issues);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            normalize_expression(scrutinee, custom, issues);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    normalize_expression(guard, custom, issues);
                }
                normalize_expression(&mut arm.body, custom, issues);
            }
        }
        SurfaceExpr::Do { items, result, .. } => {
            for item in items {
                match item {
                    SurfaceDoItem::Bind { value, .. }
                    | SurfaceDoItem::Let { value, .. }
                    | SurfaceDoItem::Expression { value, .. } => {
                        normalize_expression(value, custom, issues);
                    }
                }
            }
            if let Some(result) = result {
                normalize_expression(result, custom, issues);
            }
        }
        SurfaceExpr::Grouped { value, .. } => normalize_expression(value, custom, issues),
        SurfaceExpr::Unit { .. }
        | SurfaceExpr::Integer { .. }
        | SurfaceExpr::String { .. }
        | SurfaceExpr::Boolean { .. }
        | SurfaceExpr::Name { .. }
        | SurfaceExpr::Error { .. } => {}
    }

    let SurfaceExpr::InfixChain { .. } = expression else {
        return;
    };
    let span = expression.span();
    let chain = std::mem::replace(expression, SurfaceExpr::Error { span });
    let SurfaceExpr::InfixChain { first, steps, .. } = chain else {
        unreachable!("infix chain was matched before replacement")
    };
    *expression = associate_chain(*first, steps, span, custom, issues);
}

fn associate_chain(
    first: SurfaceExpr,
    steps: Vec<SurfaceInfixStep>,
    span: ByteSpan,
    custom: &BTreeMap<String, OperatorSpec>,
    issues: &mut Vec<ResolveIssue>,
) -> SurfaceExpr {
    let mut values = vec![first];
    let mut operators: Vec<PendingOperator> = Vec::new();

    for step in steps {
        let spec =
            builtin_operator_spec(&step.operator).or_else(|| custom.get(&step.operator).copied());
        let Some(spec) = spec else {
            issues.push(ResolveIssue {
                code: "SES-P0101".to_owned(),
                message_key: "operator.unknown".to_owned(),
                primary: step.operator_span,
            });
            return SurfaceExpr::Error { span };
        };
        while let Some(previous) = operators.last() {
            match reduction_at_boundary(previous.spec.fixity, spec.fixity) {
                Ok(true) => reduce_once(&mut values, &mut operators),
                Ok(false) => break,
                Err(()) => {
                    issues.push(ResolveIssue {
                        code: "SES-P0102".to_owned(),
                        message_key: "operator.fixity-conflict".to_owned(),
                        primary: step.operator_span,
                    });
                    return SurfaceExpr::Error { span };
                }
            }
        }
        operators.push(PendingOperator {
            spelling: step.operator,
            span: step.operator_span,
            spec,
        });
        values.push(step.operand);
    }
    while !operators.is_empty() {
        reduce_once(&mut values, &mut operators);
    }
    values
        .pop()
        .expect("a surface infix chain always contains its first operand")
}

fn reduction_at_boundary(previous: Fixity, incoming: Fixity) -> Result<bool, ()> {
    if previous.rank != incoming.rank {
        return Ok(previous.rank > incoming.rank);
    }
    match (previous.associativity, incoming.associativity) {
        (Associativity::Left, Associativity::Left) => Ok(true),
        (Associativity::Right, Associativity::Right) => Ok(false),
        _ => Err(()),
    }
}

fn reduce_once(values: &mut Vec<SurfaceExpr>, operators: &mut Vec<PendingOperator>) {
    let operator = operators
        .pop()
        .expect("operator reduction requires a pending operator");
    let right = values
        .pop()
        .expect("operator reduction requires a right operand");
    let left = values
        .pop()
        .expect("operator reduction requires a left operand");
    values.push(apply_operator(operator, left, right));
}

fn apply_operator(operator: PendingOperator, left: SurfaceExpr, right: SurfaceExpr) -> SurfaceExpr {
    let span = ByteSpan {
        start: left.span().start,
        end: right.span().end,
    };
    match operator.spec.meaning {
        OperatorMeaning::Binary => SurfaceExpr::Binary {
            operator: operator.spelling,
            operator_span: operator.span,
            left: Box::new(left),
            right: Box::new(right),
            span,
        },
        OperatorMeaning::Apply => SurfaceExpr::Application {
            function: Box::new(left),
            argument: Box::new(right),
            span,
        },
        OperatorMeaning::Pipeline => SurfaceExpr::Application {
            function: Box::new(right),
            argument: Box::new(left),
            span,
        },
        OperatorMeaning::TraitMethod {
            method,
            method_operand_sources,
        } => trait_method_application(
            method,
            operator.span,
            left,
            right,
            method_operand_sources,
            span,
        ),
        OperatorMeaning::Custom => {
            custom_operator_application(operator.spelling, operator.span, left, right, span)
        }
    }
}

fn custom_operator_application(
    spelling: String,
    operator_span: ByteSpan,
    left: SurfaceExpr,
    right: SurfaceExpr,
    span: ByteSpan,
) -> SurfaceExpr {
    let first_span = ByteSpan {
        start: left.span().start.min(operator_span.start),
        end: left.span().end.max(operator_span.end),
    };
    SurfaceExpr::Application {
        function: Box::new(SurfaceExpr::Application {
            function: Box::new(SurfaceExpr::Name {
                name: spelling,
                span: operator_span,
            }),
            argument: Box::new(left),
            span: first_span,
        }),
        argument: Box::new(right),
        span,
    }
}

fn trait_method_application(
    method: &str,
    operator_span: ByteSpan,
    left: SurfaceExpr,
    right: SurfaceExpr,
    method_operand_sources: [usize; 2],
    span: ByteSpan,
) -> SurfaceExpr {
    let mut source_operands = [Some(left), Some(right)];
    let [first_source, second_source] = method_operand_sources;
    let first = source_operands[first_source]
        .take()
        .expect("trait operator method order must be a permutation");
    let second = source_operands[second_source]
        .take()
        .expect("trait operator method order must be a permutation");
    let first_span = ByteSpan {
        start: operator_span.start.min(first.span().start),
        end: operator_span.end.max(first.span().end),
    };
    SurfaceExpr::Application {
        function: Box::new(SurfaceExpr::Application {
            function: Box::new(SurfaceExpr::Name {
                name: method.to_owned(),
                span: operator_span,
            }),
            argument: Box::new(first),
            span: first_span,
        }),
        argument: Box::new(second),
        span,
    }
}

fn builtin_operator_spec(spelling: &str) -> Option<OperatorSpec> {
    if let Some(operator) = seseragi_syntax::standard_trait_operator(spelling) {
        return Some(OperatorSpec {
            fixity: Fixity {
                rank: operator.fixity_rank,
                associativity: match operator.associativity {
                    seseragi_syntax::OperatorAssociativity::Left => Associativity::Left,
                    seseragi_syntax::OperatorAssociativity::Right => Associativity::Right,
                },
            },
            meaning: OperatorMeaning::TraitMethod {
                method: operator.method_name,
                method_operand_sources: operator.method_operand_sources,
            },
        });
    }
    let (rank, associativity, meaning) = match spelling {
        "$" => (-4, Associativity::Right, OperatorMeaning::Apply),
        "|>" => (-2, Associativity::Left, OperatorMeaning::Pipeline),
        "==" | "!=" | "<" | "<=" | ">" | ">=" => (6, Associativity::None, OperatorMeaning::Binary),
        ".." | "..=" => (7, Associativity::Left, OperatorMeaning::Binary),
        "+" | "-" => (8, Associativity::Left, OperatorMeaning::Binary),
        "*" | "/" | "%" => (10, Associativity::Left, OperatorMeaning::Binary),
        "**" => (12, Associativity::Right, OperatorMeaning::Binary),
        _ => return None,
    };
    Some(OperatorSpec {
        fixity: Fixity {
            rank,
            associativity,
        },
        meaning,
    })
}

fn is_reserved_operator(spelling: &str) -> bool {
    builtin_operator_spec(spelling).is_some()
        || matches!(
            spelling,
            "->" | "<-" | "..." | "//" | "&&" | "||" | "??" | ":="
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::{parse_surface_ast, SurfaceDecl};

    fn normalized(source: &str) -> (Vec<SurfaceDecl>, Vec<ResolveIssue>) {
        let mut surface = parse_surface_ast("artifact/operator/main.ssrg", source);
        let issues = normalize_operator_chains(&mut surface.declarations, []);
        (surface.declarations, issues)
    }

    #[test]
    fn associates_right_fixity_and_standard_precedence() {
        let (declarations, issues) = normalized(
            "operator infixr 6 <+> left: Int -> right: Int -> Int = left + right\n\
             let result: Int = 10 <+> 3 * 2 <+> 1\n",
        );
        assert!(issues.is_empty(), "{issues:#?}");
        let SurfaceDecl::Let {
            body:
                Some(SurfaceExpr::Binary {
                    operator,
                    left,
                    right,
                    ..
                }),
            ..
        } = &declarations[1]
        else {
            panic!("expected standard multiplication outside custom calls");
        };
        assert_eq!(operator, "*");
        assert!(is_custom_call(left, "<+>"));
        assert!(is_custom_call(right, "<+>"));
    }

    #[test]
    fn reports_unknown_and_conflicting_fixities() {
        let (_, unknown) = normalized("let result: Int = 1 <^> 2\n");
        assert_eq!(unknown[0].code, "SES-P0101");

        let (_, conflict) = normalized(
            "operator infix 4 <+> left: Int -> right: Int -> Int = left + right\n\
             let result: Int = 1 <+> 2 <+> 3\n",
        );
        assert_eq!(conflict[0].code, "SES-P0102");
    }

    #[test]
    fn rejects_invalid_spelling_and_non_binary_declarations() {
        let (_, single_character) =
            normalized("operator infixl 4 ^ left: Int -> right: Int -> Int = left\n");
        assert_eq!(single_character[0].code, "SES-P0001");
        assert_eq!(single_character[0].message_key, "operator.invalid-spelling");

        let (_, generic_delimiter) =
            normalized("operator infixl 4 << left: Int -> right: Int -> Int = left\n");
        assert_eq!(generic_delimiter[0].code, "SES-P0001");
        assert_eq!(
            generic_delimiter[0].message_key,
            "operator.invalid-spelling"
        );

        let (_, unary) = normalized("operator infixl 4 <^> value: Int -> Int = value\n");
        assert_eq!(unary[0].code, "SES-P0001");
        assert_eq!(unary[0].message_key, "operator.invalid-arity");
    }

    #[test]
    fn accepts_dot_inside_a_custom_operator() {
        let (declarations, issues) = normalized(
            "operator infixl 4 <.> left: Int -> right: Int -> Int = left\n\
             let result: Int = 1 <.> 2\n",
        );

        assert!(issues.is_empty(), "{issues:#?}");
        let SurfaceDecl::Let {
            body: Some(body), ..
        } = &declarations[1]
        else {
            panic!("expected result declaration");
        };
        assert!(is_custom_call(body, "<.>"));
    }

    fn is_custom_call(expression: &SurfaceExpr, spelling: &str) -> bool {
        matches!(
            expression,
            SurfaceExpr::Application { function, .. }
                if matches!(
                    function.as_ref(),
                    SurfaceExpr::Application { function, .. }
                        if matches!(
                            function.as_ref(),
                            SurfaceExpr::Name { name, .. } if name == spelling
                        )
                )
        )
    }
}
