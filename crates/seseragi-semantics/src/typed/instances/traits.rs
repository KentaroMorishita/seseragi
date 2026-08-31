use seseragi_syntax::SurfaceDecl;

use super::DerivedInstanceIssue;

pub(super) fn unknown_trait_issues(declarations: &[SurfaceDecl]) -> Vec<DerivedInstanceIssue> {
    let mut issues = Vec::new();
    for declaration in declarations {
        let (deriving, primary, declaration_span) = match declaration {
            SurfaceDecl::Type {
                deriving,
                name_span,
                span,
                ..
            }
            | SurfaceDecl::Newtype {
                deriving,
                name_span,
                span,
                ..
            }
            | SurfaceDecl::Struct {
                deriving,
                name_span,
                span,
                ..
            } => (deriving, *name_span, *span),
            _ => continue,
        };
        issues.extend(
            deriving
                .iter()
                .filter(|trait_name| {
                    !crate::prelude::trait_by_name(trait_name)
                        .is_some_and(|trait_spec| trait_spec.deriving)
                })
                .map(|trait_name| DerivedInstanceIssue::UnknownTrait {
                    trait_name: trait_name.clone(),
                    primary,
                    declaration: declaration_span,
                }),
        );
    }
    issues
}
