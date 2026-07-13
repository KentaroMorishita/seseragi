//! Canonical source layout over the shared lossless syntax artifacts.
//!
//! This crate never reparses expressions or assigns operator precedence. The
//! first formatter gate only rewrites line endings, indentation, trailing
//! whitespace, and excess blank lines. Token order and non-trivia spelling are
//! preserved, leaving fixity-sensitive line wrapping to a later shared-fixity
//! slice.

mod layout;

use seseragi_syntax::{CstArtifact, TokenStream};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattedSource {
    pub text: String,
    pub changed: bool,
}

/// Formats artifacts produced from the same source snapshot.
///
/// A recovery tree is returned byte-for-byte. Product adapters can choose to
/// report the shared parser diagnostics; the core formatter itself guarantees
/// that an error node never causes tokens to be inserted, deleted, or moved.
pub fn format_cst(tokens: &TokenStream, cst: &CstArtifact) -> FormattedSource {
    let original = tokens.reconstructed_text();
    if !cst.errors.is_empty() || !cst.missing.is_empty() {
        return FormattedSource {
            text: original,
            changed: false,
        };
    }

    let text = layout::format_valid_module(tokens, cst);
    FormattedSource {
        changed: text != original,
        text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::{lex, parse_cst_from_tokens};

    fn format(source: &str) -> FormattedSource {
        let tokens = lex("main.ssrg", source);
        let cst = parse_cst_from_tokens(tokens.clone());
        format_cst(&tokens, &cst)
    }

    #[test]
    fn canonicalizes_phase_one_layout_and_is_idempotent() {
        let source = concat!(
            "pub type Hand =   \r\n",
            "\t\t| Rock  \r\n",
            "      | Paper\r\n",
            "\r\n",
            "fn decide first: Hand -> second: Hand -> Hand =   \r\n",
            "      match (first, second) {\r\n",
            "          (Rock, Paper) -> Paper   \r\n",
            "            _ -> first\r\n",
            "      }\r\n",
            "\r\n",
            "\r\n",
        );
        let expected = concat!(
            "pub type Hand =\n",
            "  | Rock\n",
            "  | Paper\n",
            "\n",
            "fn decide first: Hand -> second: Hand -> Hand =\n",
            "  match (first, second) {\n",
            "    (Rock, Paper) -> Paper\n",
            "    _ -> first\n",
            "  }\n",
        );

        let first = format(source);
        assert!(first.changed);
        assert_eq!(first.text, expected);
        let second = format(&first.text);
        assert!(!second.changed);
        assert_eq!(second.text, expected);
    }

    #[test]
    fn preserves_recovery_source_byte_for_byte() {
        let source = "pub let answer: Int =   \r\n";
        let formatted = format(source);

        assert!(!formatted.changed);
        assert_eq!(formatted.text, source);
    }

    #[test]
    fn phase_one_goal_program_is_already_canonical() {
        let source = include_str!(
            "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg"
        );
        let formatted = format(source);

        assert!(!formatted.changed);
        assert_eq!(formatted.text, source);
    }
}
