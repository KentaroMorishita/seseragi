use super::{CstMissing, CstNode, CstParser};
use crate::{CstError, TokenKind};

pub(super) fn parse_type_decl(parser: &mut CstParser<'_>, start: usize, end: usize) -> CstNode {
    let mut children = Vec::new();
    let Some(name) = parser.next_significant_token(start + 1, end) else {
        reject_missing(parser, end, "type name");
        return CstNode::new("type-decl", start, end, children);
    };
    if parser.kind_at(name) != Some(TokenKind::IdentifierUpper) {
        reject_token(parser, name);
    }

    let Some(equals) =
        parser.find_significant_token(name + 1, end, |kind| kind == TokenKind::OperatorEquals)
    else {
        reject_missing(parser, end, "=");
        return CstNode::new("type-decl", start, end, children);
    };

    let pipes = (equals + 1..end)
        .filter(|index| parser.raw_at(*index) == Some("|"))
        .collect::<Vec<_>>();
    if pipes.is_empty() {
        reject_missing(parser, equals + 1, "variant");
    }
    for (position, pipe) in pipes.iter().enumerate() {
        let variant_end = pipes.get(position + 1).copied().unwrap_or(end);
        children.push(parse_variant(parser, *pipe, variant_end));
    }

    CstNode::new("type-decl", start, end, children)
}

fn parse_variant(parser: &mut CstParser<'_>, pipe: usize, end: usize) -> CstNode {
    let mut children = Vec::new();
    let Some(name) = parser.next_significant_token(pipe + 1, end) else {
        reject_missing(parser, pipe + 1, "constructor name");
        return CstNode::new("variant-decl", pipe, end, children);
    };
    if parser.kind_at(name) != Some(TokenKind::IdentifierUpper) {
        reject_token(parser, name);
    }
    if let Some(payload) = parser.next_significant_token(name + 1, end) {
        children.push(CstNode::new("variant-payload", payload, end, vec![]));
    }
    CstNode::new("variant-decl", pipe, end, children)
}

fn reject_missing(parser: &mut CstParser<'_>, at_token: usize, expected: &str) {
    let at_byte = parser
        .tokens
        .get(at_token)
        .map(|token| token.start)
        .unwrap_or_else(|| parser.tokens.last().map(|token| token.end).unwrap_or(0));
    parser.missing.push(CstMissing {
        expected: expected.to_owned(),
        at_token,
        at_byte,
    });
    parser.errors.push(CstError {
        code: "SES-P0001".to_owned(),
        start_token: at_token,
        end_token: at_token,
    });
}

fn reject_token(parser: &mut CstParser<'_>, token: usize) {
    parser.errors.push(CstError {
        code: "SES-P0001".to_owned(),
        start_token: token,
        end_token: token + 1,
    });
}

#[cfg(test)]
mod tests {
    use crate::parse_cst;

    #[test]
    fn preserves_public_and_private_adt_variants() {
        let cst = parse_cst(
            "main.ssrg",
            "pub type Maybe<A> =\n  | Nothing\n  | Just A\n\ntype Hand =\n  | Rock\n  | Paper\n",
        );

        assert!(cst.errors.is_empty());
        assert_eq!(cst.root.children.len(), 2);
        let public_type = &cst.root.children[0].children[1];
        assert_eq!(public_type.kind, "type-decl");
        assert_eq!(public_type.children.len(), 2);
        assert_eq!(public_type.children[0].kind, "variant-decl");
        assert!(public_type.children[0].children.is_empty());
        assert_eq!(public_type.children[1].children[0].kind, "variant-payload");
        assert_eq!(cst.root.children[1].children[0].kind, "type-decl");
    }

    #[test]
    fn rejects_empty_and_lowercase_adt_declarations() {
        for source in [
            "type Empty =\n",
            "type bad = | Rock\n",
            "type Bad = | rock\n",
        ] {
            let cst = parse_cst("main.ssrg", source);

            assert_eq!(cst.errors.len(), 1, "{source:?}");
            assert_eq!(cst.errors[0].code, "SES-P0001");
        }
    }
}
