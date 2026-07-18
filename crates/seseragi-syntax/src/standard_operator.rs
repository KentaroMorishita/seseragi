use crate::{SurfaceDecl, SurfaceImplMember, SurfaceMethod, SurfaceParameter, TypeRef};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardOperatorKind {
    Arithmetic,
    Equality,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardOperator {
    pub spelling: &'static str,
    pub trait_name: &'static str,
    pub method_name: &'static str,
    pub kind: StandardOperatorKind,
    pub declarable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorAssociativity {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardTraitOperator {
    pub spelling: &'static str,
    pub trait_name: &'static str,
    pub method_name: &'static str,
    /// For each method argument, identifies the corresponding source operand.
    pub method_operand_sources: [usize; 2],
    pub parser_precedence: u8,
    pub fixity_rank: i32,
    pub associativity: OperatorAssociativity,
}

const STANDARD_OPERATORS: &[StandardOperator] = &[
    arithmetic("+", "Add", "add"),
    arithmetic("-", "Sub", "sub"),
    arithmetic("*", "Mul", "mul"),
    arithmetic("/", "Div", "div"),
    arithmetic("%", "Rem", "rem"),
    arithmetic("**", "Pow", "pow"),
    StandardOperator {
        spelling: "==",
        trait_name: "Eq",
        method_name: "eq",
        kind: StandardOperatorKind::Equality,
        declarable: true,
    },
    StandardOperator {
        spelling: "!=",
        trait_name: "Eq",
        method_name: "eq",
        kind: StandardOperatorKind::Equality,
        declarable: false,
    },
];

const STANDARD_TRAIT_OPERATORS: &[StandardTraitOperator] = &[
    StandardTraitOperator {
        spelling: "<$>",
        trait_name: "Functor",
        method_name: "map",
        method_operand_sources: [0, 1],
        parser_precedence: 10,
        fixity_rank: -2,
        associativity: OperatorAssociativity::Left,
    },
    StandardTraitOperator {
        spelling: "<*>",
        trait_name: "Applicative",
        method_name: "apply",
        method_operand_sources: [0, 1],
        parser_precedence: 10,
        fixity_rank: -2,
        associativity: OperatorAssociativity::Left,
    },
    StandardTraitOperator {
        spelling: ">>=",
        trait_name: "Monad",
        method_name: "flatMap",
        method_operand_sources: [1, 0],
        parser_precedence: 10,
        fixity_rank: -2,
        associativity: OperatorAssociativity::Left,
    },
];

const fn arithmetic(
    spelling: &'static str,
    trait_name: &'static str,
    method_name: &'static str,
) -> StandardOperator {
    StandardOperator {
        spelling,
        trait_name,
        method_name,
        kind: StandardOperatorKind::Arithmetic,
        declarable: true,
    }
}

pub fn standard_operator(spelling: &str) -> Option<&'static StandardOperator> {
    STANDARD_OPERATORS
        .iter()
        .find(|operator| operator.spelling == spelling)
}

pub fn standard_trait_operator(spelling: &str) -> Option<&'static StandardTraitOperator> {
    STANDARD_TRAIT_OPERATORS
        .iter()
        .find(|operator| operator.spelling == spelling)
}

pub fn declarable_standard_operator(spelling: &str) -> Option<&'static StandardOperator> {
    standard_operator(spelling).filter(|operator| operator.declarable)
}

pub fn impl_operator_instances(declaration: &SurfaceDecl) -> Vec<SurfaceDecl> {
    let SurfaceDecl::Impl {
        type_parameters,
        target,
        constraints,
        members,
        ..
    } = declaration
    else {
        return Vec::new();
    };

    members
        .iter()
        .filter_map(|member| {
            let SurfaceImplMember::Operator {
                spelling,
                spelling_span,
                self_span,
                parameters,
                return_type,
                body,
                span,
                ..
            } = member
            else {
                return None;
            };
            let operator = declarable_standard_operator(spelling)?;
            let mut method_parameters = vec![SurfaceParameter {
                name: "self".to_owned(),
                name_span: *self_span,
                type_ref: target.clone(),
            }];
            method_parameters.extend(parameters.iter().cloned());
            let arguments = match operator.kind {
                StandardOperatorKind::Arithmetic => vec![
                    target.clone(),
                    parameters
                        .first()
                        .map(|parameter| parameter.type_ref.clone())
                        .unwrap_or(TypeRef::Hole { span: *self_span }),
                    return_type.clone(),
                ],
                StandardOperatorKind::Equality => vec![target.clone()],
            };

            Some(SurfaceDecl::Instance {
                type_parameters: type_parameters.clone(),
                trait_name: operator.trait_name.to_owned(),
                trait_name_span: *spelling_span,
                arguments,
                constraints: constraints.clone(),
                methods: vec![SurfaceMethod {
                    name: operator.method_name.to_owned(),
                    name_span: *spelling_span,
                    type_parameters: Vec::new(),
                    parameters: method_parameters,
                    return_type: return_type.clone(),
                    constraints: Vec::new(),
                    body: body.clone(),
                    span: *span,
                }],
                span: *span,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_surface_ast;

    #[test]
    fn expands_an_impl_operator_into_a_standard_trait_instance() {
        let surface = parse_surface_ast(
            "artifact/operator-sugar/main.ssrg",
            "struct Score { value: Int }\nimpl Score { operator + self -> other: Int -> Score = self }\n",
        );
        let instances = impl_operator_instances(&surface.declarations[1]);
        let [SurfaceDecl::Instance {
            trait_name,
            arguments,
            methods,
            ..
        }] = instances.as_slice()
        else {
            panic!("expected one synthesized instance");
        };

        assert_eq!(trait_name, "Add");
        assert_eq!(arguments.len(), 3);
        assert_eq!(methods[0].name, "add");
        assert_eq!(methods[0].parameters[0].name, "self");
        assert_eq!(methods[0].parameters.len(), 2);
    }

    #[test]
    fn does_not_offer_inequality_as_an_overload_declaration() {
        assert!(standard_operator("!=").is_some());
        assert!(declarable_standard_operator("!=").is_none());
    }

    #[test]
    fn records_trait_operator_source_to_method_order() {
        assert_eq!(
            standard_trait_operator("<$>")
                .unwrap()
                .method_operand_sources,
            [0, 1]
        );
        assert_eq!(
            standard_trait_operator(">>=")
                .unwrap()
                .method_operand_sources,
            [1, 0]
        );
    }

    #[test]
    fn exposes_operator_sugar_as_a_module_instance() {
        let interface = crate::parse_module_interface(
            "artifact/operator-interface/main.ssrg",
            "pub struct Score { value: Int }\nimpl Score { operator + self -> other: Int -> Score = self }\n",
        );

        assert_eq!(interface.instances.len(), 1);
        assert_eq!(interface.instances[0].trait_name, "Add");
        let crate::InterfaceType::Apply { arguments, .. } = &interface.instances[0].head else {
            panic!("expected an applied Add head");
        };
        assert_eq!(arguments.len(), 3);
    }
}
