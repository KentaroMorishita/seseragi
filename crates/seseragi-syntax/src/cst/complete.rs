//! Complete syntax boundaries projected onto lossless tokens by the shared
//! source parser. Layout clients never parse source fragments themselves.
use super::CstNode;
use crate::surface_model::*;
use crate::{parse_surface_ast, TokenStream};

pub(super) fn enrich(root: &mut CstNode, tokens: &TokenStream) {
    let module = parse_surface_ast(&tokens.source, &tokens.reconstructed_text());
    let builder = Builder(tokens);
    for decl in &module.declarations {
        let node = builder.declaration(decl);
        if let Some(parent) = root.children.iter_mut().find(|parent| {
            parent.start_token <= node.start_token && parent.end_token >= node.end_token
        }) {
            parent.children.push(node);
        }
    }
}

struct Builder<'a>(&'a TokenStream);
impl Builder<'_> {
    fn node(&self, kind: &str, span: ByteSpan, children: Vec<CstNode>) -> CstNode {
        let start = self
            .0
            .tokens
            .partition_point(|token| token.end <= span.start);
        let end = self
            .0
            .tokens
            .partition_point(|token| token.start < span.end);
        CstNode::new(format!("complete-{kind}"), start, end.max(start), children)
    }
    fn declaration(&self, decl: &SurfaceDecl) -> CstNode {
        let span = decl.span();
        let children = match decl {
            SurfaceDecl::Let { body, .. }
            | SurfaceDecl::Fn { body, .. }
            | SurfaceDecl::EffectFn { body, .. }
            | SurfaceDecl::Operator { body, .. } => self.callable(span, body.as_ref()),
            SurfaceDecl::Impl { members, .. } => members
                .iter()
                .map(|member| {
                    let body = match member {
                        SurfaceImplMember::Method { method, .. } => method.body.as_ref(),
                        SurfaceImplMember::Operator { body, .. } => body.as_ref(),
                    };
                    self.node("member", member.span(), self.callable(member.span(), body))
                })
                .collect(),
            SurfaceDecl::Trait { methods, .. } | SurfaceDecl::Instance { methods, .. } => methods
                .iter()
                .map(|method| {
                    self.node(
                        "member",
                        method.span,
                        self.callable(method.span, method.body.as_ref()),
                    )
                })
                .collect(),
            _ => Vec::new(),
        };
        self.node("declaration", span, children)
    }
    fn callable(&self, span: ByteSpan, body: Option<&SurfaceExpr>) -> Vec<CstNode> {
        body.map(|body| {
            vec![
                self.node(
                    "signature",
                    ByteSpan {
                        start: span.start,
                        end: body.span().start,
                    },
                    Vec::new(),
                ),
                self.expression(body),
            ]
        })
        .unwrap_or_default()
    }
    fn expression(&self, expr: &SurfaceExpr) -> CstNode {
        let mut children = Vec::new();
        match expr {
            SurfaceExpr::Index {
                receiver, index, ..
            } => children.extend([self.expression(receiver), self.expression(index)]),
            SurfaceExpr::Member { receiver, .. } => children.push(self.expression(receiver)),
            SurfaceExpr::Application {
                function, argument, ..
            } => children.extend([self.expression(function), self.expression(argument)]),
            SurfaceExpr::Prefix { operand, .. } => children.push(self.expression(operand)),
            SurfaceExpr::Assignment { target, value, .. } => {
                children.extend([self.expression(target), self.expression(value)])
            }
            SurfaceExpr::Lambda { body, .. } => children.push(self.expression(body)),
            SurfaceExpr::EffectfulFor { source, body, .. } => {
                children.extend([self.expression(source), self.expression(body)])
            }
            SurfaceExpr::Tuple { elements, .. }
            | SurfaceExpr::Array { elements, .. }
            | SurfaceExpr::List { elements, .. } => children.extend(
                elements
                    .iter()
                    .map(|item| self.node("item", item.span(), vec![self.expression(item)])),
            ),
            SurfaceExpr::Record { items, .. } | SurfaceExpr::Struct { items, .. } => children
                .extend(items.iter().map(|item| {
                    self.node("field", item.span(), vec![self.expression(item.value())])
                })),
            SurfaceExpr::ArrayComprehension {
                element, clauses, ..
            }
            | SurfaceExpr::ListComprehension {
                element, clauses, ..
            } => {
                children.push(self.expression(element));
                for clause in clauses {
                    let (span, value) = match clause {
                        SurfaceComprehensionClause::Generator { span, source, .. } => {
                            (*span, source)
                        }
                        SurfaceComprehensionClause::Guard { span, condition } => (*span, condition),
                    };
                    children.push(self.node("clause", span, vec![self.expression(value)]));
                }
            }
            SurfaceExpr::Binary { left, right, .. } => {
                children.extend([self.expression(left), self.expression(right)])
            }
            SurfaceExpr::InfixChain { first, steps, .. } => {
                children.push(self.expression(first));
                children.extend(steps.iter().map(|step| self.expression(&step.operand)));
            }
            SurfaceExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => children.extend([
                self.expression(condition),
                self.expression(then_branch),
                self.expression(else_branch),
            ]),
            SurfaceExpr::Match {
                scrutinee, arms, ..
            } => {
                children.push(self.expression(scrutinee));
                for arm in arms {
                    let mut parts = arm
                        .guard
                        .iter()
                        .map(|guard| self.expression(guard))
                        .collect::<Vec<_>>();
                    parts.push(self.expression(&arm.body));
                    children.push(self.node("arm", arm.span, parts));
                }
            }
            SurfaceExpr::Block { items, result, .. } => {
                for item in items {
                    let (span, value) = match item {
                        SurfaceBlockItem::Let { span, value, .. }
                        | SurfaceBlockItem::Function { span, value, .. } => (*span, value),
                    };
                    children.push(self.node("block-item", span, vec![self.expression(value)]));
                }
                children.push(self.expression(result));
            }
            SurfaceExpr::Do { items, result, .. } => {
                for item in items {
                    let (span, value) = match item {
                        SurfaceDoItem::Let { span, value, .. }
                        | SurfaceDoItem::Bind { span, value, .. }
                        | SurfaceDoItem::Expression { span, value } => (*span, value),
                    };
                    children.push(self.node("do-item", span, vec![self.expression(value)]));
                }
                children.extend(result.iter().map(|result| self.expression(result)));
            }
            SurfaceExpr::Grouped { value, .. } => children.push(self.expression(value)),
            _ => {}
        }
        self.node(
            if matches!(expr, SurfaceExpr::Error { .. }) {
                "error"
            } else {
                "expression"
            },
            expr.span(),
            children,
        )
    }
}
