use seseragi_syntax::SurfaceDecl;

use super::DerivedInstanceIssue;

const KNOWN_DERIVING_TRAITS: &[&str] = &[
    "Eq",
    "Ord",
    "Show",
    "Debug",
    "Hash",
    "JsonEncode",
    "JsonDecode",
];

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
                .filter(|trait_name| !KNOWN_DERIVING_TRAITS.contains(&trait_name.as_str()))
                .map(|trait_name| DerivedInstanceIssue::UnknownTrait {
                    trait_name: trait_name.clone(),
                    primary,
                    declaration: declaration_span,
                }),
        );
    }
    issues
}
