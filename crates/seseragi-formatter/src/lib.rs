//! Canonical source layout over the shared lossless syntax artifacts.
//!
//! This crate never reparses expressions or assigns operator precedence. It
//! renders the shared lossless token/CST artifacts into a deterministic layout:
//! two-space indentation, normalized token spacing, compact groups that fit
//! the requested source width, and stable structural breaks for signatures,
//! operator chains, collections, and nested blocks. Token order and
//! non-trivia spelling are preserved.

mod layout;

use seseragi_syntax::{CstArtifact, TokenStream};

pub const DEFAULT_LINE_WIDTH: usize = 88;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatOptions {
    pub line_width: usize,
}

impl FormatOptions {
    pub const fn new(line_width: usize) -> Self {
        Self { line_width }
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self::new(DEFAULT_LINE_WIDTH)
    }
}

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
    format_cst_with_options(tokens, cst, FormatOptions::default())
}

pub fn format_cst_with_options(
    tokens: &TokenStream,
    cst: &CstArtifact,
    options: FormatOptions,
) -> FormattedSource {
    let original = tokens.reconstructed_text();
    if !cst.errors.is_empty() || !cst.missing.is_empty() {
        return FormattedSource {
            text: original,
            changed: false,
        };
    }

    let text = layout::format_valid_module(tokens, cst, options.line_width);
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

    fn format_with_width(source: &str, line_width: usize) -> FormattedSource {
        let tokens = lex("main.ssrg", source);
        let cst = parse_cst_from_tokens(tokens.clone());
        format_cst_with_options(&tokens, &cst, FormatOptions::new(line_width))
    }

    #[test]
    fn preserves_right_associative_maybe_fallback_layout() {
        let source = "pub let display = cached??requested??\"anonymous\"\n";
        for width in [20, 88] {
            let formatted = format_with_width(source, width);
            assert_eq!(
                format_with_width(&formatted.text, width).text,
                formatted.text
            );
            assert_eq!(formatted.text.matches("??").count(), 2);
            assert!(
                seseragi_syntax::parse_diagnostics("fallback.ssrg", &formatted.text)
                    .diagnostics
                    .is_empty()
            );
        }
    }

    #[test]
    fn preserves_char_literal_spelling_and_apostrophe_application() {
        let source = r"let account'='瀬'
pub let result=(account','\u{03BB}','\'','\\')
";
        for width in [20, 88] {
            let formatted = format_with_width(source, width);
            assert_eq!(
                format_with_width(&formatted.text, width).text,
                formatted.text
            );
            for spelling in ["account'", "'瀬'", r"'\u{03BB}'", r"'\''", r"'\\'"] {
                assert!(formatted.text.contains(spelling), "{}", formatted.text);
            }
            assert!(
                seseragi_syntax::parse_diagnostics("char.ssrg", &formatted.text)
                    .diagnostics
                    .is_empty()
            );
        }
    }

    #[test]
    fn keeps_index_adjacency_distinct_from_array_application() {
        let source = "let values=[1,2,3]\nlet selected=values[ 1 ]\nlet applied=read [1,2]\nlet nested=([values])[0]\n";
        for width in [20, 88] {
            let formatted = format_with_width(source, width);
            assert!(formatted.text.contains("values[1]"), "{}", formatted.text);
            assert!(formatted.text.contains("read ["), "{}", formatted.text);
            assert_eq!(
                format_with_width(&formatted.text, width).text,
                formatted.text
            );
        }
    }

    #[test]
    fn default_options_are_byte_identical_to_the_legacy_entrypoint() {
        let input = include_str!("../tests/fixtures/canonical-layout.input.ssrg");

        assert_eq!(format(input), format_with_width(input, DEFAULT_LINE_WIDTH));
    }

    #[test]
    fn applies_narrow_width_at_structural_boundaries_and_converges() {
        let source = concat!(
            "pub effect fn main userName: String -> Unit with Console fails AppError = do {\n",
            "  println userName\n",
            "}\n",
            "\n",
            "let labels = [\"formatter\", \"playground\", \"curriculum\", \"diagnostics\"]\n",
        );

        let wide = format_with_width(source, 88);
        let narrow = format_with_width(source, 48);

        assert_ne!(narrow.text, wide.text);
        assert!(narrow.text.contains("\nwith Console\n"), "{}", narrow.text);
        assert!(narrow.text.contains("[\n"), "{}", narrow.text);
        assert!(narrow.text.contains("  \"formatter\","), "{}", narrow.text);

        let converged = format_with_width(&narrow.text, 48);
        assert!(!converged.changed, "{}", converged.text);
        assert_eq!(converged.text, narrow.text);
    }

    #[test]
    fn narrow_options_preserve_unsplittable_tokens_and_complex_surface_idempotence() {
        let source = concat!(
            "let endpoint = \"https://example.test/a-single-token-that-is-longer-than-forty-eight-columns\"\n",
            "\n",
            include_str!("../tests/fixtures/style-contract.expected.ssrg"),
        );

        let narrow = format_with_width(source, 48);
        assert!(narrow.text.contains(
            "https://example.test/a-single-token-that-is-longer-than-forty-eight-columns"
        ));
        let converged = format_with_width(&narrow.text, 48);
        assert!(!converged.changed, "{}", converged.text);
        assert_eq!(converged.text, narrow.text);
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
    fn enforces_the_full_style_contract_and_converges() {
        let input = include_str!("../tests/fixtures/style-contract.input.ssrg");
        let expected = include_str!("../tests/fixtures/style-contract.expected.ssrg");

        let first = format(input);
        assert!(first.changed);
        assert_eq!(first.text, expected);

        let formatted_tokens = lex("formatted.ssrg", &first.text);
        let formatted_cst = parse_cst_from_tokens(formatted_tokens);
        assert!(
            formatted_cst.errors.is_empty(),
            "{:#?}",
            formatted_cst.errors
        );
        assert!(
            formatted_cst.missing.is_empty(),
            "{:#?}",
            formatted_cst.missing
        );

        let second = format(expected);
        assert!(!second.changed, "{}", second.text);
        assert_eq!(second.text, expected);
    }

    #[test]
    fn dogfood_indentation_preserves_declaration_and_branch_boundaries() {
        let input = include_str!("../tests/fixtures/dogfood-indentation.input.ssrg");
        let expected = include_str!("../tests/fixtures/dogfood-indentation.expected.ssrg");
        for source in [input.to_owned(), input.replace('\n', "\r\n")] {
            let formatted = format(&source);
            assert_eq!(formatted.text, expected);
            assert_eq!(format(&formatted.text).text, expected);
            assert_eq!(format(&format(&formatted.text).text).text, expected);
        }
        let tight = input.replace("\n\n//", "\n//");
        assert_eq!(format(&tight).text, expected.replace("\n\n//", "\n//"));
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
    fn canonicalizes_implementation_member_boundaries() {
        let source = concat!(
            "struct Score { value: Int }\n",
            "\n",
            "impl Score { operator + self -> bonus: Int -> Score = Score { value: bonus }\n",
            "\n",
            "operator == self -> other: Score -> Bool = self.value == other.value }\n",
        );
        let expected = concat!(
            "struct Score { value: Int }\n",
            "\n",
            "impl Score {\n",
            "  operator + self -> bonus: Int -> Score = Score { value: bonus }\n",
            "\n",
            "  operator == self -> other: Score -> Bool = self.value == other.value\n",
            "}\n",
        );

        let first = format(source);
        assert_eq!(first.text, expected);
        assert!(!format(expected).changed);
    }

    #[test]
    fn preserves_bodyless_declaration_boundaries_and_converges() {
        let input = include_str!("../tests/fixtures/declaration-boundaries.input.ssrg");
        let expected = include_str!("../tests/fixtures/declaration-boundaries.expected.ssrg");

        let first = format(input);
        assert!(first.changed);
        assert_eq!(first.text, expected);

        let tokens = lex("formatted.ssrg", &first.text);
        let cst = parse_cst_from_tokens(tokens);
        assert!(cst.errors.is_empty(), "{:#?}", cst.errors);
        assert!(cst.missing.is_empty(), "{:#?}", cst.missing);

        let second = format(expected);
        assert!(!second.changed, "{}", second.text);
        assert_eq!(second.text, expected);
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
            "fn decide first: Hand -> second: Hand -> Hand = match (first, second) {\n",
            "  (Rock, Paper) -> Paper\n",
            "  _ -> first\n",
            "}\n",
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
            "pub effect fn main = do {\n",
            "  input <- readLine () |> mapError StdinFailure\n",
            "  let parsed = input |> parseInput\n",
            "  parsed |> println\n",
            "}\n",
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
    fn wraps_a_long_pipeline_at_operator_boundaries() {
        let source = concat!(
            "pub effect fn main = do {\n",
            "  values |> map firstLongTransformation |> filter secondLongPredicate |> collect thirdLongValue |> println\n",
            "}\n",
        );
        let expected = concat!(
            "pub effect fn main = do {\n",
            "  values\n",
            "    |> map firstLongTransformation\n",
            "    |> filter secondLongPredicate\n",
            "    |> collect thirdLongValue\n",
            "    |> println\n",
            "}\n",
        );

        let first = format(source);
        assert_eq!(first.text, expected);
        assert!(!format(expected).changed);
    }

    #[test]
    fn keeps_a_short_parenthesized_argument_compact_in_a_long_application() {
        let source = concat!(
            "let view = render firstLongArgument secondLongArgument thirdLongArgument ",
            "(dispatch state Submitted) fourthLongArgument fifthLongArgument\n",
        );

        let expected = concat!(
            "let view =\n",
            "  render firstLongArgument secondLongArgument thirdLongArgument ",
            "(dispatch state Submitted) fourthLongArgument fifthLongArgument\n",
        );

        let formatted = format(source);
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
            "struct User { id: Int, name: String, }\n",
            "\n",
            "fn identifier -> Int = {\n",
            "  let { value } = { value: 1 }\n",
            "  let User { id, name } = User { id: value, name: \"Aki\" }\n",
            "  id\n",
            "}\n",
            "\n",
            "pub effect fn main = do {\n",
            "  let (left, right) = (1, 2)\n",
            "  for (value, label) <- [(left + right, \"sum\")] {\n",
            "    println $ `${label}: ${value}`\n",
            "  }\n",
            "  println \"done\"\n",
            "}\n",
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
