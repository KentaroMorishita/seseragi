//! Canonical source layout over the shared lossless syntax artifacts.
//!
//! This crate never reparses expressions or assigns operator precedence. It
//! renders the shared lossless token/CST artifacts into the fixed canonical
//! layout: two-space indentation, normalized token spacing, compact groups
//! that fit 88 source columns, and stable structural breaks for signatures,
//! operator chains, collections, and nested blocks. Token order and
//! non-trivia spelling are preserved.

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
    fn formats_the_golden_layout_corpus_and_converges() {
        let input = include_str!("../tests/fixtures/canonical-layout.input.ssrg");
        let expected = include_str!("../tests/fixtures/canonical-layout.expected.ssrg");

        let first = format(input);
        assert!(first.changed);
        assert_eq!(first.text, expected);

        let second = format(expected);
        assert!(!second.changed, "{}", second.text);
        assert_eq!(second.text, expected);
    }

    #[test]
    fn preserves_foreign_member_boundaries() {
        let source = concat!(
            "foreign \"typescript\" from \"../host/logical.mjs\" {\n",
            "  pure fn check label: String -> value: Bool -> Bool\n",
            "  pure fn explode label: String -> Bool\n",
            "}\n",
            "\n",
            "pub let result = False && explode \"unreachable\"\n",
        );

        let formatted = format(source);
        assert!(!formatted.changed, "{}", formatted.text);
        assert_eq!(formatted.text, source);
    }

    #[test]
    fn canonicalizes_logical_operator_spacing() {
        let first = format("pub let result=True&&False||True\n");
        assert_eq!(first.text, "pub let result = True && False || True\n");

        let second = format(&first.text);
        assert!(!second.changed);
        assert_eq!(second.text, first.text);
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

        assert!(!formatted.changed, "{}", formatted.text);
        assert_eq!(formatted.text, source);
    }

    #[test]
    fn phase_one_goal_program_is_already_canonical() {
        let source = include_str!(
            "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg"
        );
        let formatted = format(source);

        assert!(!formatted.changed, "{}", formatted.text);
        assert_eq!(formatted.text, source);
    }

    #[test]
    fn compacts_short_do_items_without_losing_block_rhythm() {
        let source = concat!(
            "pub effect fn main =\n",
            "  do {\n",
            "    input <-\n",
            "      readLine ()\n",
            "      |> mapError StdinFailure\n",
            "    let parsed =\n",
            "      input\n",
            "      |> parseInput\n",
            "    parsed\n",
            "    |> println\n",
            "  }\n",
        );

        let expected = concat!(
            "pub effect fn main =\n",
            "  do {\n",
            "    input <- readLine () |> mapError StdinFailure\n",
            "    let parsed = input |> parseInput\n",
            "    parsed |> println\n",
            "  }\n",
        );
        let formatted = format(source);

        assert!(formatted.changed);
        assert_eq!(formatted.text, expected);
        assert!(!format(expected).changed);
    }

    #[test]
    fn compacts_an_operator_chain_that_fits_the_width() {
        let source = concat!(
            "fn transform value: Maybe<Int> -> Maybe<Int> =\n",
            "  increment <$> value\n",
            "  >>= validate\n",
        );

        let expected =
            "fn transform value: Maybe<Int> -> Maybe<Int> = increment <$> value >>= validate\n";
        let formatted = format(source);

        assert!(formatted.changed);
        assert_eq!(formatted.text, expected);
        assert!(!format(expected).changed);
    }

    #[test]
    fn compacts_a_small_collection_and_preserves_declaration_spacing() {
        let source = concat!(
            "fn cx classes: Array<String> -> String =\n",
            "  join \" \" classes\n",
            "\n",
            "let cardClass =\n",
            "  cx [\n",
            "  \"rounded-2xl\",\n",
            "  \"bg-white\",\n",
            "  \"p-6\",\n",
            "  \"shadow-lg\"\n",
            "  ]\n",
        );

        let expected = concat!(
            "fn cx classes: Array<String> -> String = join \" \" classes\n",
            "\n",
            "let cardClass = cx [\"rounded-2xl\", \"bg-white\", \"p-6\", \"shadow-lg\"]\n",
        );
        let formatted = format(source);

        assert!(formatted.changed);
        assert_eq!(formatted.text, expected);
        assert!(!format(expected).changed);
    }

    #[test]
    fn preserves_pattern_bindings_inside_struct_block_and_nested_do_braces() {
        let source = concat!(
            "struct User {\n",
            "  id: Int,\n",
            "  name: String,\n",
            "}\n",
            "\n",
            "fn identifier -> Int = {\n",
            "  let { value } = { value: 1 }\n",
            "  let User { id, name } = User { id: value, name: \"Aki\" }\n",
            "  id\n",
            "}\n",
            "\n",
            "pub effect fn main =\n",
            "  do {\n",
            "    let (left, right) = (1, 2)\n",
            "    for (value, label) <- [(left + right, \"sum\")] {\n",
            "      println $ `${label}: ${value}`\n",
            "    }\n",
            "    println \"done\"\n",
            "  }\n",
        );

        let expected = concat!(
            "struct User {\n",
            "  id: Int,\n",
            "  name: String,\n",
            "}\n",
            "\n",
            "fn identifier -> Int =\n",
            "  {\n",
            "    let { value } = { value: 1 }\n",
            "    let User { id, name } = User { id: value, name: \"Aki\" }\n",
            "    id\n",
            "  }\n",
            "\n",
            "pub effect fn main =\n",
            "  do {\n",
            "    let (left, right) = (1, 2)\n",
            "    for (value, label) <- [(left + right, \"sum\")] {\n",
            "      println $ `${label}: ${value}`\n",
            "    }\n",
            "    println \"done\"\n",
            "  }\n",
        );

        let formatted = format(source);

        assert!(formatted.changed, "{}", formatted.text);
        assert_eq!(formatted.text, expected);
        assert!(!format(&expected).changed);
    }

    #[test]
    fn canonicalizes_alias_layout_without_changing_its_type_structure() {
        let source = concat!(
            "pub alias Pair<A> = { left: A, right: A }  \r\n",
            "\r\n",
            "alias TaskResult<A> =\r\n",
            "      Effect<{}, Never, A>\r\n",
            "alias StateT<S, M<_>, A> = S -> M<(A, S)>\r\n",
        );
        let expected = concat!(
            "pub alias Pair<A> = { left: A, right: A }\n",
            "\n",
            "alias TaskResult<A> = Effect<{}, Never, A>\n",
            "alias StateT<S, M<_>, A> = S -> M<(A, S)>\n",
        );

        let first = format(source);
        assert!(first.changed);
        assert_eq!(first.text, expected);
        assert!(!format(&first.text).changed);
    }
}
