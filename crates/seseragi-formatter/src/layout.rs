use std::collections::HashSet;

use seseragi_syntax::{CstArtifact, CstNode, Token, TokenKind, TokenStream};

const LINE_WIDTH: usize = 88;

pub(super) fn format_valid_module(tokens: &TokenStream, cst: &CstArtifact) -> String {
    let source_lines = source_lines(&tokens.tokens);
    let token_lines = token_line_map(&source_lines, tokens.tokens.len());
    let angles = angle_tokens(&tokens.tokens);
    let delimiters = Delimiters::new(&tokens.tokens, &angles);
    let lines = logical_lines(&source_lines, &tokens.tokens, &delimiters, &angles);
    let mut output = Vec::new();
    let mut delimiter_depth = 0usize;

    for line in lines {
        let LogicalLine::Content(indices) = line else {
            push_blank_line(&mut output);
            continue;
        };
        let Some(first) = indices.first().copied() else {
            continue;
        };
        let leading_closers = leading_closers(&indices, &tokens.tokens, &angles);
        let structural_depth = delimiter_depth.saturating_sub(leading_closers);
        let continuation = declaration_continuation(cst, first, &tokens.tokens);
        let do_item_continuation = do_item_continuation(cst, &token_lines, first, &tokens.tokens);
        let branch_continuation = delimiters.branch_depth(first);
        let indent = structural_depth + continuation + do_item_continuation + branch_continuation;
        output.extend(format_logical_line(
            &indices,
            &tokens.tokens,
            &angles,
            indent,
        ));
        delimiter_depth =
            updated_delimiter_depth(delimiter_depth, &indices, &tokens.tokens, &angles);
    }

    while output.last().is_some_and(|line| line.is_empty()) {
        output.pop();
    }
    output.push(String::new());
    output.join("\n")
}

#[derive(Clone, Debug)]
enum LogicalLine {
    Blank,
    Content(Vec<usize>),
}

fn logical_lines(
    source_lines: &[SourceLine],
    tokens: &[Token],
    delimiters: &Delimiters,
    angles: &HashSet<usize>,
) -> Vec<LogicalLine> {
    let mut result = Vec::new();
    let mut pending: Option<Vec<usize>> = None;

    for line in source_lines {
        let indices = significant_indices(tokens, line.start, line.end);
        if indices.is_empty() {
            if let Some(content) = pending.take() {
                result.push(LogicalLine::Content(content));
            }
            if !matches!(result.last(), Some(LogicalLine::Blank)) {
                result.push(LogicalLine::Blank);
            }
            continue;
        }

        if let Some(current) = pending.as_mut() {
            if should_join(current, &indices, tokens, delimiters, angles) {
                current.extend(indices);
                continue;
            }
            let complete = pending.replace(indices).expect("pending logical line");
            result.push(LogicalLine::Content(complete));
        } else {
            pending = Some(indices);
        }
    }

    if let Some(content) = pending {
        result.push(LogicalLine::Content(content));
    }
    result
}

fn should_join(
    current: &[usize],
    next: &[usize],
    tokens: &[Token],
    delimiters: &Delimiters,
    angles: &HashSet<usize>,
) -> bool {
    if current
        .iter()
        .chain(next)
        .any(|index| tokens[*index].kind == TokenKind::TriviaComment)
    {
        return false;
    }
    let previous = *current.last().expect("non-empty logical line");
    let following = next[0];
    let previous_token = &tokens[previous];
    let following_token = &tokens[following];

    if let Some(open) = current.iter().rev().find(|index| {
        is_open_delimiter(tokens[**index].kind)
            && delimiters
                .matching(**index)
                .is_some_and(|close| close > previous)
    }) {
        return delimiters.joinable_open(*open);
    }

    if following_token.raw == "|" {
        return false;
    }

    let declaration_header_continues = !current
        .iter()
        .any(|index| tokens[*index].kind == TokenKind::OperatorEquals)
        && (current
            .iter()
            .any(|index| tokens[*index].kind == TokenKind::KeywordFn)
            || matches!(
                (current.first(), current.get(1)),
                (Some(first), Some(second))
                    if tokens[*first].raw == "operator"
                        && matches!(tokens[*second].raw.as_str(), "infixl" | "infixr")
            ));
    if declaration_header_continues {
        return true;
    }

    if previous_token.kind == TokenKind::PunctuationComma
        || (!angles.contains(&previous) && is_trailing_operator(previous_token))
        || (!angles.contains(&following) && is_leading_operator(following_token))
        || matches!(
            following_token.kind,
            TokenKind::OperatorArrow
                | TokenKind::KeywordWith
                | TokenKind::KeywordFails
                | TokenKind::KeywordThen
                | TokenKind::KeywordElse
        )
        || matches!(
            previous_token.kind,
            TokenKind::KeywordThen | TokenKind::KeywordElse
        )
        || following_token.raw == "where"
    {
        return true;
    }

    if is_open_delimiter(previous_token.kind) && delimiters.joinable_open(previous) {
        return true;
    }
    if is_close_delimiter(following_token.kind)
        && delimiters
            .matching(following)
            .is_some_and(|open| delimiters.joinable_open(open))
    {
        return true;
    }
    false
}

fn format_logical_line(
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    base_indent: usize,
) -> Vec<String> {
    if let Some(operator) = structural_rhs_break(indices, tokens, angles) {
        let mut lines = format_logical_line(&indices[..=operator], tokens, angles, base_indent);
        lines.extend(format_logical_line(
            &indices[operator + 1..],
            tokens,
            angles,
            base_indent + 1,
        ));
        return lines;
    }

    let flat = render_flat(indices, tokens, angles);
    let local = LocalDelimiters::new(indices, tokens, angles);
    let force_struct = starts_struct_declaration(indices, tokens);
    let mut expanded = HashSet::new();
    if display_width(&flat) + base_indent * 2 > LINE_WIDTH || force_struct {
        mark_expanded_groups(
            indices,
            tokens,
            angles,
            &local,
            base_indent,
            force_struct,
            &mut expanded,
        );
    }

    let needs_breaking = display_width(&flat) + base_indent * 2 > LINE_WIDTH;
    if expanded.is_empty() && !needs_breaking {
        return vec![format!("{}{}", "  ".repeat(base_indent), flat)];
    }

    let equals = top_level_equals(indices, tokens, angles);
    let signature_needs_breaking = equals.is_some_and(|position| {
        display_width(&render_flat(&indices[..=position], tokens, angles)) + base_indent * 2
            > LINE_WIDTH
    });
    let mut writer = LineWriter::new(base_indent, tokens, angles);
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut before_equals = true;

    for (position, index) in indices.iter().copied().enumerate() {
        let token = &tokens[index];
        let is_expanded_close = is_close_delimiter(token.kind)
            && local
                .matching_position(position)
                .is_some_and(|open| expanded.contains(&open));
        if is_expanded_close {
            let open_indent = stack
                .last()
                .map(|(_, child_indent)| child_indent.saturating_sub(1))
                .unwrap_or(base_indent);
            writer.break_line(open_indent);
        } else if needs_breaking && stack.is_empty() {
            let break_indent = if signature_needs_breaking
                && (matches!(token.kind, TokenKind::KeywordWith | TokenKind::KeywordFails)
                    || token.raw == "where")
            {
                Some(base_indent)
            } else if signature_needs_breaking
                && token.kind == TokenKind::OperatorArrow
                && before_equals
            {
                Some(base_indent + 1)
            } else if matches!(token.kind, TokenKind::KeywordThen | TokenKind::KeywordElse) {
                Some(base_indent)
            } else if position > 0
                && equals == Some(position - 1)
                && !matches!(token.kind, TokenKind::PunctuationBraceLeft)
            {
                Some(base_indent + 1)
            } else if position > 0
                && !angles.contains(&index)
                && is_breakable_operator(token)
                && token.kind != TokenKind::OperatorApply
            {
                Some(base_indent + 1)
            } else {
                None
            };
            if let Some(indent) = break_indent {
                writer.break_line(indent);
            }
        }

        writer.push(index);
        if Some(position) == equals {
            before_equals = false;
        }

        if token.kind == TokenKind::OperatorApply && needs_breaking && stack.is_empty() {
            writer.break_line(base_indent + 1);
        } else if matches!(token.kind, TokenKind::KeywordThen | TokenKind::KeywordElse)
            && needs_breaking
            && stack.is_empty()
            && !(token.kind == TokenKind::KeywordElse
                && indices
                    .get(position + 1)
                    .is_some_and(|next| tokens[*next].kind == TokenKind::KeywordIf))
        {
            writer.break_line(base_indent + 1);
        }

        if is_open_delimiter(token.kind) && !angles.contains(&index) {
            let child_indent = writer.current_indent + 1;
            stack.push((position, child_indent));
            if expanded.contains(&position) {
                writer.break_line(child_indent);
            }
        } else if is_close_delimiter(token.kind) && !angles.contains(&index) {
            stack.pop();
        } else if token.kind == TokenKind::PunctuationComma {
            if let Some((open, child_indent)) = stack.last() {
                if expanded.contains(open) {
                    writer.break_line(*child_indent);
                }
            }
        } else if token.kind == TokenKind::PunctuationSemicolon {
            writer.break_line(
                stack
                    .last()
                    .map(|(_, child_indent)| *child_indent)
                    .unwrap_or(base_indent),
            );
        }

        let next = indices.get(position + 1).copied();
        if needs_breaking
            && writer.current_width() > LINE_WIDTH
            && next.is_some()
            && !matches!(
                token.kind,
                TokenKind::LiteralString | TokenKind::LiteralTemplate
            )
            && next.is_some_and(|next| safe_emergency_break_after(index, next, tokens, angles))
        {
            writer.break_line(
                stack
                    .last()
                    .map(|(_, child_indent)| *child_indent)
                    .unwrap_or(base_indent + 1),
            );
        }
    }
    writer.finish()
}

fn mark_expanded_groups(
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    local: &LocalDelimiters,
    base_indent: usize,
    force_struct: bool,
    expanded: &mut HashSet<usize>,
) {
    for position in 0..indices.len() {
        let Some(close) = local.matching_position(position) else {
            continue;
        };
        let record_like = tokens[indices[position]].kind == TokenKind::PunctuationBraceLeft
            && is_record_like_brace(indices[position], tokens);
        if close <= position
            || (!record_like && !local.has_direct_comma(position, close, indices, tokens, angles))
        {
            continue;
        }
        let group = &indices[position..=close];
        let group_is_long = display_width(&render_flat(group, tokens, angles))
            + (base_indent + local.depth(position)) * 2
            > LINE_WIDTH;
        let is_struct_fields = force_struct
            && tokens[indices[position]].kind == TokenKind::PunctuationBraceLeft
            && local.depth(position) == 0;
        if is_struct_fields || group_is_long {
            expanded.insert(position);
        }
    }
}

fn is_structural_expression_start(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::KeywordIf
            | TokenKind::KeywordMatch
            | TokenKind::KeywordDo
            | TokenKind::PunctuationBraceLeft
    )
}

fn safe_emergency_break_after(
    current: usize,
    next: usize,
    tokens: &[Token],
    angles: &HashSet<usize>,
) -> bool {
    let left = &tokens[current];
    let right = &tokens[next];
    if angles.contains(&current) || angles.contains(&next) {
        return false;
    }
    !is_operator(left.kind)
        && !is_operator(right.kind)
        && !is_open_delimiter(left.kind)
        && !is_close_delimiter(right.kind)
        && !matches!(
            left.kind,
            TokenKind::PunctuationDot | TokenKind::PunctuationColon
        )
        && !matches!(
            right.kind,
            TokenKind::PunctuationDot
                | TokenKind::PunctuationColon
                | TokenKind::PunctuationComma
                | TokenKind::PunctuationSemicolon
        )
}

fn structural_rhs_break(
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
) -> Option<usize> {
    let mut depth = 0usize;
    for (position, pair) in indices.windows(2).enumerate() {
        let index = pair[0];
        if !angles.contains(&index) {
            let kind = tokens[index].kind;
            if is_open_delimiter(kind) {
                depth += 1;
            } else if is_close_delimiter(kind) {
                depth = depth.saturating_sub(1);
            }
        }
        if depth == 0
            && matches!(
                tokens[index].kind,
                TokenKind::OperatorEquals | TokenKind::OperatorArrow
            )
            && is_structural_expression_start(tokens[pair[1]].kind)
            && !(tokens[pair[1]].kind == TokenKind::PunctuationBraceLeft
                && brace_has_direct_field(pair[1], tokens))
        {
            return Some(position);
        }
    }
    None
}

fn brace_has_direct_field(open: usize, tokens: &[Token]) -> bool {
    let mut depth = 0usize;
    for token in tokens.iter().skip(open) {
        if token.kind == TokenKind::PunctuationBraceLeft {
            depth += 1;
            continue;
        }
        if token.kind == TokenKind::PunctuationBraceRight {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return false;
            }
            continue;
        }
        if depth == 1
            && matches!(
                token.kind,
                TokenKind::PunctuationColon | TokenKind::PunctuationComma
            )
        {
            return true;
        }
    }
    false
}

struct LineWriter<'a> {
    lines: Vec<String>,
    current: String,
    current_indent: usize,
    previous: Option<usize>,
    tokens: &'a [Token],
    angles: &'a HashSet<usize>,
}

impl<'a> LineWriter<'a> {
    fn new(indent: usize, tokens: &'a [Token], angles: &'a HashSet<usize>) -> Self {
        Self {
            lines: Vec::new(),
            current: "  ".repeat(indent),
            current_indent: indent,
            previous: None,
            tokens,
            angles,
        }
    }

    fn push(&mut self, index: usize) {
        if let Some(previous) = self.previous {
            if needs_space(previous, index, self.tokens, self.angles) {
                if self.tokens[index].kind == TokenKind::TriviaComment {
                    self.current.push_str("  ");
                } else {
                    self.current.push(' ');
                }
            }
        }
        self.current.push_str(self.tokens[index].raw.trim_end());
        self.previous = Some(index);
    }

    fn break_line(&mut self, indent: usize) {
        if self.previous.is_none()
            || indent == self.current_indent && self.current.trim().is_empty()
        {
            self.current_indent = indent;
            self.current = "  ".repeat(indent);
            return;
        }
        let line = std::mem::replace(&mut self.current, "  ".repeat(indent));
        self.lines.push(line.trim_end().to_owned());
        self.current_indent = indent;
        self.previous = None;
    }

    fn current_width(&self) -> usize {
        display_width(&self.current)
    }

    fn finish(mut self) -> Vec<String> {
        if self.previous.is_some() {
            self.lines.push(self.current.trim_end().to_owned());
        }
        self.lines
    }
}

fn render_flat(indices: &[usize], tokens: &[Token], angles: &HashSet<usize>) -> String {
    let mut result = String::new();
    let mut previous = None;
    for index in indices.iter().copied() {
        if let Some(previous) = previous {
            if needs_space(previous, index, tokens, angles) {
                result.push(' ');
            }
        }
        result.push_str(tokens[index].raw.trim_end());
        previous = Some(index);
    }
    result
}

fn needs_space(previous: usize, current: usize, tokens: &[Token], angles: &HashSet<usize>) -> bool {
    let left = &tokens[previous];
    let right = &tokens[current];
    if is_operator(left.kind) && is_operator(right.kind) && left.end == right.start {
        return false;
    }
    if right.kind == TokenKind::TriviaComment {
        return true;
    }
    if angles.contains(&current) && right.raw == "<" {
        return false;
    }
    if angles.contains(&current) && right.raw == ">" {
        return false;
    }
    if angles.contains(&previous) && left.raw == "<" {
        return false;
    }
    if right.kind == TokenKind::PunctuationBraceRight {
        return left.kind != TokenKind::PunctuationBraceLeft;
    }
    if matches!(
        right.kind,
        TokenKind::PunctuationParenRight
            | TokenKind::PunctuationSquareRight
            | TokenKind::PunctuationComma
            | TokenKind::PunctuationColon
            | TokenKind::PunctuationDot
            | TokenKind::PunctuationSemicolon
    ) {
        return false;
    }
    if left.kind == TokenKind::PunctuationBraceLeft {
        return right.kind != TokenKind::PunctuationBraceRight;
    }
    if matches!(
        left.kind,
        TokenKind::PunctuationParenLeft
            | TokenKind::PunctuationSquareLeft
            | TokenKind::PunctuationListLeft
            | TokenKind::PunctuationDot
            | TokenKind::OperatorLambda
            | TokenKind::PunctuationEllipsis
    ) {
        return false;
    }
    if matches!(
        left.kind,
        TokenKind::PunctuationComma | TokenKind::PunctuationColon | TokenKind::PunctuationSemicolon
    ) {
        return true;
    }
    if is_prefix_operator(left, previous, tokens, angles) {
        return false;
    }
    if is_operator(left.kind) || is_operator(right.kind) {
        return true;
    }
    true
}

fn is_prefix_operator(
    token: &Token,
    index: usize,
    tokens: &[Token],
    angles: &HashSet<usize>,
) -> bool {
    if angles.contains(&index) {
        return false;
    }
    if !matches!(
        (token.kind, token.raw.as_str()),
        (TokenKind::OperatorArithmetic, "-" | "*") | (TokenKind::OperatorCustom, "!")
    ) {
        return false;
    }
    let previous = tokens[..index]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, token)| !is_trivia(token.kind));
    previous.is_none_or(|(_, previous)| {
        is_open_delimiter(previous.kind)
            || previous.kind == TokenKind::PunctuationComma
            || is_operator(previous.kind)
            || matches!(
                previous.kind,
                TokenKind::KeywordThen | TokenKind::KeywordElse | TokenKind::KeywordLet
            )
    })
}

fn is_operator(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::OperatorArrow
            | TokenKind::OperatorArithmetic
            | TokenKind::OperatorApply
            | TokenKind::OperatorAssignment
            | TokenKind::OperatorBind
            | TokenKind::OperatorComparison
            | TokenKind::OperatorEquals
            | TokenKind::OperatorPipeline
            | TokenKind::OperatorRangeExclusive
            | TokenKind::OperatorRangeInclusive
            | TokenKind::OperatorCustom
    )
}

fn is_breakable_operator(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::OperatorPipeline | TokenKind::OperatorApply | TokenKind::OperatorCustom
    )
}

fn is_trailing_operator(token: &Token) -> bool {
    is_operator(token.kind)
        && !matches!(
            (token.kind, token.raw.as_str()),
            (TokenKind::OperatorArithmetic, "-" | "*") | (TokenKind::OperatorCustom, "!")
        )
}

fn is_leading_operator(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::OperatorPipeline
            | TokenKind::OperatorApply
            | TokenKind::OperatorCustom
            | TokenKind::OperatorArithmetic
            | TokenKind::OperatorComparison
    ) && !matches!(token.raw.as_str(), "-" | "*" | "!")
}

fn top_level_equals(indices: &[usize], tokens: &[Token], angles: &HashSet<usize>) -> Option<usize> {
    let mut depth = 0usize;
    for (position, index) in indices.iter().copied().enumerate() {
        if angles.contains(&index) {
            continue;
        }
        let kind = tokens[index].kind;
        if is_open_delimiter(kind) {
            depth += 1;
        } else if is_close_delimiter(kind) {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && kind == TokenKind::OperatorEquals {
            return Some(position);
        }
    }
    None
}

fn starts_struct_declaration(indices: &[usize], tokens: &[Token]) -> bool {
    indices
        .iter()
        .take(3)
        .any(|index| tokens[*index].raw == "struct")
}

fn angle_tokens(tokens: &[Token]) -> HashSet<usize> {
    let significant = significant_indices(tokens, 0, tokens.len());
    let mut stack = Vec::new();
    let mut angles = HashSet::new();
    for (position, index) in significant.iter().copied().enumerate() {
        let token = &tokens[index];
        if token.raw == "<" {
            let previous = position
                .checked_sub(1)
                .and_then(|position| significant.get(position))
                .map(|index| &tokens[*index]);
            let next = significant.get(position + 1).map(|index| &tokens[*index]);
            let type_like = previous
                .is_some_and(|token| token.kind == TokenKind::IdentifierUpper || token.raw == ">")
                || next.is_some_and(|token| {
                    matches!(
                        token.kind,
                        TokenKind::IdentifierUpper
                            | TokenKind::Wildcard
                            | TokenKind::PunctuationParenLeft
                    )
                });
            if type_like {
                stack.push(index);
                angles.insert(index);
            }
        } else if token.raw == ">" && stack.pop().is_some() {
            angles.insert(index);
        }
    }
    angles
}

#[derive(Default)]
struct Delimiters {
    matching: Vec<Option<usize>>,
    joinable: HashSet<usize>,
    branch_depth: Vec<usize>,
}

impl Delimiters {
    fn new(tokens: &[Token], angles: &HashSet<usize>) -> Self {
        let mut matching = vec![None; tokens.len()];
        let mut stack: Vec<usize> = Vec::new();
        for (index, token) in tokens.iter().enumerate() {
            if is_open_delimiter(token.kind) {
                stack.push(index);
            } else if is_close_delimiter(token.kind) {
                let Some(open) = stack.pop() else {
                    continue;
                };
                if delimiters_match(tokens[open].kind, token.kind) {
                    matching[open] = Some(index);
                    matching[index] = Some(open);
                }
            }
        }

        let mut joinable = HashSet::new();
        for (open, close) in matching.iter().enumerate().filter_map(|(open, close)| {
            close
                .filter(|close| *close > open)
                .map(|close| (open, close))
        }) {
            let kind = tokens[open].kind;
            if !matches!(kind, TokenKind::PunctuationBraceLeft)
                || has_direct_comma_in_range(open, close, tokens, &matching, angles)
                || is_record_like_brace(open, tokens)
            {
                joinable.insert(open);
            }
        }
        let significant = significant_indices(tokens, 0, tokens.len());
        let mut branch_depth = vec![0; tokens.len()];
        for (position, index) in significant.iter().copied().enumerate() {
            if tokens[index].kind != TokenKind::OperatorArrow {
                continue;
            }
            let Some(next) = significant.get(position + 1).copied() else {
                continue;
            };
            let open = if tokens[next].kind == TokenKind::PunctuationBraceLeft {
                Some(next)
            } else if matches!(
                tokens[next].kind,
                TokenKind::KeywordMatch | TokenKind::KeywordDo
            ) {
                significant[position + 2..]
                    .iter()
                    .copied()
                    .find(|candidate| tokens[*candidate].kind == TokenKind::PunctuationBraceLeft)
            } else {
                None
            };
            let Some(open) = open else {
                continue;
            };
            let Some(close) = matching[open] else {
                continue;
            };
            for depth in branch_depth.iter_mut().take(close + 1).skip(open + 1) {
                *depth += 1;
            }
        }

        Self {
            matching,
            joinable,
            branch_depth,
        }
    }

    fn matching(&self, index: usize) -> Option<usize> {
        self.matching.get(index).copied().flatten()
    }

    fn joinable_open(&self, index: usize) -> bool {
        self.joinable.contains(&index)
    }

    fn branch_depth(&self, index: usize) -> usize {
        self.branch_depth.get(index).copied().unwrap_or_default()
    }
}

fn is_record_like_brace(open: usize, tokens: &[Token]) -> bool {
    for token in tokens[..open]
        .iter()
        .rev()
        .filter(|token| !is_trivia(token.kind))
    {
        if matches!(
            token.kind,
            TokenKind::OperatorEquals
                | TokenKind::OperatorArrow
                | TokenKind::PunctuationComma
                | TokenKind::PunctuationBraceLeft
                | TokenKind::PunctuationBraceRight
        ) {
            break;
        }
        if matches!(token.kind, TokenKind::KeywordMatch | TokenKind::KeywordDo) {
            return false;
        }
    }

    let mut previous = tokens[..open]
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, token)| !is_trivia(token.kind));
    let Some((_, immediate)) = previous.next() else {
        return false;
    };
    if immediate.kind == TokenKind::IdentifierUpper {
        return true;
    }
    matches!(
        (previous.next().map(|(_, token)| token.kind), immediate.kind),
        (Some(TokenKind::PunctuationDot), TokenKind::IdentifierLower)
    )
}

struct LocalDelimiters {
    matching: Vec<Option<usize>>,
    depths: Vec<usize>,
}

impl LocalDelimiters {
    fn new(indices: &[usize], tokens: &[Token], angles: &HashSet<usize>) -> Self {
        let mut matching = vec![None; indices.len()];
        let mut depths = vec![0; indices.len()];
        let mut stack: Vec<usize> = Vec::new();
        for (position, index) in indices.iter().copied().enumerate() {
            depths[position] = stack.len();
            if angles.contains(&index) {
                continue;
            }
            let kind = tokens[index].kind;
            if is_open_delimiter(kind) {
                stack.push(position);
            } else if is_close_delimiter(kind) {
                let Some(open) = stack.pop() else {
                    continue;
                };
                if delimiters_match(tokens[indices[open]].kind, kind) {
                    matching[open] = Some(position);
                    matching[position] = Some(open);
                }
            }
        }
        Self { matching, depths }
    }

    fn matching_position(&self, position: usize) -> Option<usize> {
        self.matching.get(position).copied().flatten()
    }

    fn depth(&self, position: usize) -> usize {
        self.depths.get(position).copied().unwrap_or_default()
    }

    fn has_direct_comma(
        &self,
        open: usize,
        close: usize,
        indices: &[usize],
        tokens: &[Token],
        angles: &HashSet<usize>,
    ) -> bool {
        let mut angle_depth = 0usize;
        (open + 1..close).any(|position| {
            let index = indices[position];
            if angles.contains(&index) {
                if tokens[index].raw == "<" {
                    angle_depth += 1;
                } else if tokens[index].raw == ">" {
                    angle_depth = angle_depth.saturating_sub(1);
                }
                return false;
            }
            self.depth(position) == self.depth(open) + 1
                && angle_depth == 0
                && tokens
                    .get(index)
                    .is_some_and(|token| token.kind == TokenKind::PunctuationComma)
        })
    }
}

fn has_direct_comma_in_range(
    open: usize,
    close: usize,
    tokens: &[Token],
    matching: &[Option<usize>],
    angles: &HashSet<usize>,
) -> bool {
    let mut cursor = open + 1;
    let mut angle_depth = 0usize;
    while cursor < close {
        if angles.contains(&cursor) {
            if tokens[cursor].raw == "<" {
                angle_depth += 1;
            } else if tokens[cursor].raw == ">" {
                angle_depth = angle_depth.saturating_sub(1);
            }
            cursor += 1;
            continue;
        }
        if angle_depth == 0 && tokens[cursor].kind == TokenKind::PunctuationComma {
            return true;
        }
        if angle_depth == 0 && is_open_delimiter(tokens[cursor].kind) {
            if let Some(nested_close) = matching[cursor] {
                cursor = nested_close + 1;
                continue;
            }
        }
        cursor += 1;
    }
    false
}

fn is_open_delimiter(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::PunctuationBraceLeft
            | TokenKind::PunctuationParenLeft
            | TokenKind::PunctuationSquareLeft
            | TokenKind::PunctuationListLeft
    )
}

fn is_close_delimiter(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::PunctuationBraceRight
            | TokenKind::PunctuationParenRight
            | TokenKind::PunctuationSquareRight
    )
}

fn delimiters_match(open: TokenKind, close: TokenKind) -> bool {
    matches!(
        (open, close),
        (
            TokenKind::PunctuationBraceLeft,
            TokenKind::PunctuationBraceRight
        ) | (
            TokenKind::PunctuationParenLeft,
            TokenKind::PunctuationParenRight
        ) | (
            TokenKind::PunctuationSquareLeft | TokenKind::PunctuationListLeft,
            TokenKind::PunctuationSquareRight
        )
    )
}

fn significant_indices(tokens: &[Token], start: usize, end: usize) -> Vec<usize> {
    (start..end)
        .filter(|index| !is_trivia(tokens[*index].kind))
        .collect()
}

fn is_trivia(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::TriviaSpace | TokenKind::TriviaNewline | TokenKind::Eof
    )
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

fn do_item_continuation(
    cst: &CstArtifact,
    token_lines: &[usize],
    first: usize,
    tokens: &[Token],
) -> usize {
    let Some(item) = enclosing_do_item(&cst.root, first) else {
        return 0;
    };
    if token_lines.get(item.start_token) == token_lines.get(first) {
        return 0;
    }
    if item.kind == "do-bind-item" {
        return 1;
    }
    let starts_with_let = (item.start_token..item.end_token)
        .find(|index| !is_trivia(tokens[*index].kind))
        .is_some_and(|index| tokens[index].kind == TokenKind::KeywordLet);
    usize::from(starts_with_let)
}

fn enclosing_do_item(node: &CstNode, token: usize) -> Option<&CstNode> {
    if !node.children.is_empty() {
        if let Some(item) = node
            .children
            .iter()
            .find_map(|child| enclosing_do_item(child, token))
        {
            return Some(item);
        }
    }
    matches!(node.kind.as_str(), "do-bind-item" | "do-expression-item")
        .then_some(node)
        .filter(|node| node.start_token <= token && token < node.end_token)
}

#[derive(Clone, Copy)]
struct SourceLine {
    start: usize,
    end: usize,
}

fn source_lines(tokens: &[Token]) -> Vec<SourceLine> {
    let eof = tokens
        .iter()
        .position(|token| token.kind == TokenKind::Eof)
        .unwrap_or(tokens.len());
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (index, token) in tokens.iter().enumerate().take(eof) {
        if token.kind == TokenKind::TriviaNewline {
            lines.push(SourceLine { start, end: index });
            start = index + 1;
        }
    }
    if start < eof {
        lines.push(SourceLine { start, end: eof });
    }
    lines
}

fn token_line_map(lines: &[SourceLine], token_count: usize) -> Vec<usize> {
    let mut result = vec![0; token_count];
    for (line_number, line) in lines.iter().enumerate() {
        for token_line in result.iter_mut().take(line.end).skip(line.start) {
            *token_line = line_number;
        }
    }
    result
}

fn declaration_continuation(cst: &CstArtifact, first: usize, tokens: &[Token]) -> usize {
    let Some(declaration) = cst
        .root
        .children
        .iter()
        .find(|node| node.start_token <= first && first < node.end_token)
    else {
        return 0;
    };
    let mut depth = 0usize;
    for (index, token) in tokens
        .iter()
        .enumerate()
        .take(first)
        .skip(declaration.start_token)
    {
        if is_open_delimiter(token.kind) {
            depth += 1;
        } else if is_close_delimiter(token.kind) {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && token.kind == TokenKind::OperatorEquals {
            return usize::from(first > index);
        }
    }
    0
}

fn leading_closers(indices: &[usize], tokens: &[Token], angles: &HashSet<usize>) -> usize {
    indices
        .iter()
        .take_while(|index| is_close_delimiter(tokens[**index].kind) && !angles.contains(index))
        .count()
}

fn updated_delimiter_depth(
    mut depth: usize,
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
) -> usize {
    for index in indices {
        if angles.contains(index) {
            continue;
        }
        let kind = tokens[*index].kind;
        if is_open_delimiter(kind) {
            depth += 1;
        } else if is_close_delimiter(kind) {
            depth = depth.saturating_sub(1);
        }
    }
    depth
}

fn push_blank_line(output: &mut Vec<String>) {
    if output.last().is_some_and(|line| !line.is_empty()) {
        output.push(String::new());
    }
}
