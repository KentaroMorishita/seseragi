use std::collections::HashSet;

use seseragi_syntax::{CstArtifact, CstNode, Token, TokenKind, TokenStream};

pub(super) fn format_valid_module(
    tokens: &TokenStream,
    cst: &CstArtifact,
    line_width: usize,
) -> String {
    let source_lines = source_lines(&tokens.tokens);
    let token_lines = token_line_map(&source_lines, tokens.tokens.len());
    let angles = angle_tokens(&tokens.tokens);
    let delimiters = Delimiters::new(&tokens.tokens, &angles);
    let member_bodies = member_body_map(&tokens.tokens);
    let lines = logical_lines(
        &source_lines,
        &tokens.tokens,
        &delimiters,
        &angles,
        &member_bodies,
    );
    let mut output = Vec::new();
    let mut delimiter_depth = 0usize;
    let mut implementation_member_seen = false;

    for line in lines {
        let LogicalLine::Content(indices) = line else {
            push_blank_line(&mut output);
            continue;
        };
        let Some(first) = indices.first().copied() else {
            continue;
        };
        let starts_implementation_member = member_bodies[first] == Some(MemberBody::Implementation)
            && starts_bodyless_member(&indices, &tokens.tokens, Some(MemberBody::Implementation));
        if starts_implementation_member {
            if implementation_member_seen {
                push_blank_line(&mut output);
            }
            implementation_member_seen = true;
        }
        let leading_closers = leading_closers(&indices, &tokens.tokens, &angles);
        let structural_depth = delimiter_depth.saturating_sub(leading_closers);
        let continuation = declaration_continuation(cst, first, &tokens.tokens);
        let structural_rhs_continuation = structural_rhs_body_continuation(
            cst,
            first,
            &tokens.tokens,
            &angles,
            &delimiters,
            line_width,
        );
        let do_item_continuation = do_item_continuation(cst, &token_lines, first, &tokens.tokens);
        let branch_continuation = delimiters.branch_depth(first);
        let indent = structural_depth
            + continuation
            + structural_rhs_continuation
            + do_item_continuation
            + if leading_closers > 1 {
                0
            } else {
                branch_continuation
            };
        output.extend(format_logical_line(
            &indices,
            &tokens.tokens,
            &angles,
            &delimiters,
            &member_bodies,
            indent,
            line_width,
        ));
        for (position, index) in indices.iter().copied().enumerate() {
            if tokens.tokens[index].kind == TokenKind::PunctuationBraceLeft
                && member_body_for_open(index, &tokens.tokens) == Some(MemberBody::Implementation)
            {
                implementation_member_seen = starts_bodyless_member(
                    &indices[position + 1..],
                    &tokens.tokens,
                    Some(MemberBody::Implementation),
                );
            } else if tokens.tokens[index].kind == TokenKind::PunctuationBraceRight
                && member_bodies[index] == Some(MemberBody::Implementation)
            {
                implementation_member_seen = false;
            }
        }
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
    member_bodies: &[Option<MemberBody>],
) -> Vec<LogicalLine> {
    let mut result = Vec::new();
    let mut pending: Option<(Vec<usize>, usize)> = None;

    for line in source_lines {
        let indices = significant_indices(tokens, line.start, line.end);
        if indices.is_empty() {
            if let Some((content, _)) = pending.take() {
                result.push(LogicalLine::Content(content));
            }
            if !matches!(result.last(), Some(LogicalLine::Blank)) {
                result.push(LogicalLine::Blank);
            }
            continue;
        }

        if let Some((current, current_indent)) = pending.as_mut() {
            if should_join(
                current,
                &indices,
                *current_indent,
                line.indent,
                tokens,
                delimiters,
                angles,
                member_bodies,
            ) {
                current.extend(indices);
                continue;
            }
            let (complete, _) = pending
                .replace((indices, line.indent))
                .expect("pending logical line");
            result.push(LogicalLine::Content(complete));
        } else {
            pending = Some((indices, line.indent));
        }
    }

    if let Some((content, _)) = pending {
        result.push(LogicalLine::Content(content));
    }
    result
}

fn should_join(
    current: &[usize],
    next: &[usize],
    current_indent: usize,
    next_indent: usize,
    tokens: &[Token],
    delimiters: &Delimiters,
    angles: &HashSet<usize>,
    member_bodies: &[Option<MemberBody>],
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

    if is_close_delimiter(following_token.kind) {
        return false;
    }

    if next_indent > current_indent
        && ends_application_atom(previous_token.kind)
        && starts_application_atom(following_token)
    {
        return true;
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
        return !starts_bodyless_member(
            next,
            tokens,
            member_bodies.get(following).copied().flatten(),
        );
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

fn starts_bodyless_member(
    indices: &[usize],
    tokens: &[Token],
    member_body: Option<MemberBody>,
) -> bool {
    let raw = |position: usize| {
        indices
            .get(position)
            .and_then(|index| tokens.get(*index))
            .map(|token| token.raw.as_str())
    };

    let deprecated_clause = raw(0) == Some("deprecated")
        && indices
            .get(1)
            .is_some_and(|index| tokens[*index].kind == TokenKind::LiteralString);
    match member_body {
        Some(MemberBody::Trait) => raw(0) == Some("fn") || deprecated_clause,
        Some(MemberBody::Foreign) => {
            if deprecated_clause
                || raw(0) == Some("namespace")
                    && indices
                        .get(1)
                        .is_some_and(|index| tokens[*index].kind == TokenKind::IdentifierLower)
                || raw(0) == Some("opaque") && raw(1) == Some("type")
            {
                return true;
            }
            if !matches!(raw(0), Some("pure" | "task")) {
                return false;
            }
            match raw(1) {
                Some("fn" | "value") => true,
                Some("constructor" | "method" | "property") => raw(2) == Some("fn"),
                _ => false,
            }
        }
        Some(MemberBody::Implementation) => {
            matches!(raw(0), Some("fn" | "effect" | "operator" | "pub"))
        }
        None => false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MemberBody {
    Trait,
    Foreign,
    Implementation,
}

fn member_body_map(tokens: &[Token]) -> Vec<Option<MemberBody>> {
    let mut result = vec![None; tokens.len()];
    let mut bodies = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        result[index] = bodies.last().copied().flatten();
        match token.kind {
            TokenKind::PunctuationBraceLeft => bodies.push(member_body_for_open(index, tokens)),
            TokenKind::PunctuationBraceRight => {
                bodies.pop();
            }
            _ => {}
        }
    }
    result
}

fn member_body_for_open(open: usize, tokens: &[Token]) -> Option<MemberBody> {
    for token in tokens[..open]
        .iter()
        .rev()
        .filter(|token| !is_trivia(token.kind))
    {
        if matches!(
            token.kind,
            TokenKind::PunctuationBraceLeft | TokenKind::PunctuationBraceRight
        ) {
            break;
        }
        match token.raw.as_str() {
            "trait" => return Some(MemberBody::Trait),
            "foreign" | "namespace" => return Some(MemberBody::Foreign),
            "impl" | "instance" => return Some(MemberBody::Implementation),
            _ => {}
        }
    }
    None
}

fn format_logical_line(
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    delimiters: &Delimiters,
    member_bodies: &[Option<MemberBody>],
    base_indent: usize,
    line_width: usize,
) -> Vec<String> {
    let leading = leading_closers(indices, tokens, angles);
    if leading > 1 && leading == indices.len() {
        return indices
            .iter()
            .enumerate()
            .map(|(position, index)| {
                let indent = if position == 0 {
                    base_indent + leading
                } else {
                    base_indent + leading - position - 1
                };
                format!("{}{}", "  ".repeat(indent), tokens[*index].raw.trim_end())
            })
            .collect();
    }
    let flat = render_flat(indices, tokens, angles);
    let needs_breaking = display_width(&flat) + base_indent * 2 > line_width;
    if needs_breaking {
        if let Some(operator) = structural_rhs_break(indices, tokens, angles) {
            let mut lines = format_logical_line(
                &indices[..=operator],
                tokens,
                angles,
                delimiters,
                member_bodies,
                base_indent,
                line_width,
            );
            lines.extend(format_logical_line(
                &indices[operator + 1..],
                tokens,
                angles,
                delimiters,
                member_bodies,
                base_indent + 1,
                line_width,
            ));
            return lines;
        }
    }

    let local = LocalDelimiters::new(indices, tokens, angles);
    let mut expanded = HashSet::new();
    if needs_breaking {
        mark_expanded_groups(
            indices,
            tokens,
            angles,
            delimiters,
            &local,
            base_indent,
            line_width,
            &mut expanded,
        );
    }

    let has_member_boundary = indices.iter().copied().any(|index| {
        tokens[index].kind == TokenKind::PunctuationBraceLeft
            && member_body_for_open(index, tokens).is_some()
            || tokens[index].kind == TokenKind::PunctuationBraceRight
                && member_bodies.get(index).copied().flatten().is_some()
    });
    if expanded.is_empty() && !needs_breaking && !has_member_boundary {
        return vec![format!("{}{}", "  ".repeat(base_indent), flat)];
    }

    let equals = top_level_equals(indices, tokens, angles);
    let signature_needs_breaking = equals.is_some_and(|position| {
        is_callable_header(&indices[..position], tokens)
            && display_width(&render_flat(&indices[..=position], tokens, angles)) + base_indent * 2
                > line_width
    });
    let mut writer = LineWriter::new(base_indent, tokens, angles);
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut before_equals = equals.is_some();
    let mut application_indent = None;
    let mut conditional_indents = Vec::new();
    let mut else_indent = base_indent;

    for (position, index) in indices.iter().copied().enumerate() {
        let token = &tokens[index];
        let next = indices.get(position + 1).copied();
        let is_expanded_close = is_close_delimiter(token.kind)
            && local
                .matching_position(position)
                .is_some_and(|open| expanded.contains(&open));
        let is_member_body_close = token.kind == TokenKind::PunctuationBraceRight
            && member_bodies.get(index).copied().flatten().is_some();
        if token.kind == TokenKind::KeywordElse && needs_breaking {
            else_indent = conditional_indents.pop().unwrap_or(base_indent);
            writer.break_line(else_indent);
        } else if is_member_body_close {
            writer.break_line(if position == 0 {
                base_indent
            } else {
                base_indent.saturating_sub(1)
            });
        } else if is_expanded_close {
            let open_indent = stack
                .last()
                .map(|(_, child_indent)| child_indent.saturating_sub(1))
                .unwrap_or(base_indent);
            writer.break_line(open_indent);
        } else if position > 0
            && tokens[indices[position - 1]].kind == TokenKind::PunctuationColon
            && is_structural_expression_start(token.kind)
            && !(token.kind == TokenKind::PunctuationBraceLeft
                && brace_has_direct_field(index, tokens))
            && stack
                .last()
                .is_some_and(|(open, _)| expanded.contains(open))
        {
            let indent = stack
                .last()
                .map(|(_, child_indent)| child_indent + 1)
                .unwrap_or(base_indent + 1);
            writer.break_line(indent);
        } else if needs_breaking && stack.is_empty() {
            let rhs_head_does_not_fit = position > 0
                && equals == Some(position - 1)
                && (display_width(&render_flat(&indices[..position], tokens, angles))
                    + base_indent * 2
                    + 1
                    + application_atom_width(
                        position, index, indices, tokens, angles, &local, &expanded,
                    )
                    > line_width
                    || expanded
                        .iter()
                        .copied()
                        .filter(|open| *open >= position)
                        .min()
                        .is_some_and(|open| {
                            let start = rhs_segment_start(
                                position,
                                indices,
                                tokens,
                                angles,
                                signature_needs_breaking,
                            );
                            display_width(&render_flat(&indices[start..=open], tokens, angles))
                                + base_indent * 2
                                > line_width
                        })
                    || expanded.is_empty()
                        && rhs_segment_width(
                            position,
                            indices,
                            tokens,
                            angles,
                            signature_needs_breaking,
                        ) + base_indent * 2
                            > line_width);
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
            } else if token.kind == TokenKind::KeywordElse {
                Some(base_indent)
            } else if rhs_head_does_not_fit
                && !matches!(token.kind, TokenKind::PunctuationBraceLeft)
            {
                Some(base_indent + 1)
            } else if position > 0
                && !before_equals
                && (!angles.contains(&index) && is_breakable_operator(token)
                    || starts_composite_operator(position, indices, tokens))
                && !writer
                    .previous
                    .is_some_and(|previous| continues_operator_spelling(previous, index, tokens))
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

        let application_width =
            application_atom_width(position, index, indices, tokens, angles, &local, &expanded);
        let inside_parenthesized_atom = stack.iter().any(|(open, _)| {
            tokens[indices[*open]].kind == TokenKind::PunctuationParenLeft
                && !expanded.contains(open)
        });
        if needs_breaking
            && !stack.is_empty()
            && !inside_parenthesized_atom
            && writer.projected_width_by(application_width, index) > line_width
            && writer.previous.is_some_and(|previous| {
                is_application_boundary(previous, index, next, tokens, angles)
            })
        {
            let candidate = stack
                .last()
                .map(|(_, child_indent)| child_indent + 1)
                .unwrap_or(writer.current_indent + 1);
            let indent = *application_indent.get_or_insert(candidate);
            writer.break_line(indent);
        }

        if token.kind == TokenKind::KeywordIf {
            conditional_indents.push(writer.current_indent);
        }
        writer.push(index);
        if Some(position) == equals {
            before_equals = false;
        }

        if token.kind == TokenKind::OperatorApply && needs_breaking && stack.is_empty() {
            writer.break_line(base_indent + 1);
        } else if matches!(token.kind, TokenKind::KeywordThen | TokenKind::KeywordElse)
            && needs_breaking
            && !(token.kind == TokenKind::KeywordElse
                && indices
                    .get(position + 1)
                    .is_some_and(|next| tokens[*next].kind == TokenKind::KeywordIf))
        {
            let head_indent = if token.kind == TokenKind::KeywordThen {
                conditional_indents.last().copied().unwrap_or(base_indent)
            } else {
                else_indent
            };
            writer.break_line(head_indent + 1);
        }

        if is_open_delimiter(token.kind) && !angles.contains(&index) {
            let child_indent = writer.current_indent + 1;
            stack.push((position, child_indent));
            if expanded.contains(&position)
                || token.kind == TokenKind::PunctuationBraceLeft
                    && member_body_for_open(index, tokens).is_some()
            {
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
    }
    writer.finish()
}

fn mark_expanded_groups(
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    delimiters: &Delimiters,
    local: &LocalDelimiters,
    base_indent: usize,
    line_width: usize,
    expanded: &mut HashSet<usize>,
) {
    for position in 0..indices.len() {
        let local_close = local.matching_position(position);
        let global_open = indices[position];
        let global_close = delimiters.matching(global_open);
        let record_like = tokens[indices[position]].kind == TokenKind::PunctuationBraceLeft
            && is_record_like_brace(indices[position], tokens);
        let member_body = tokens[indices[position]].kind == TokenKind::PunctuationBraceLeft
            && member_body_for_open(indices[position], tokens).is_some();
        let application_group = tokens[indices[position]].kind == TokenKind::PunctuationParenLeft
            && local_close.is_some_and(|close| {
                (position + 1..close).any(|current| {
                    let previous = indices[current - 1];
                    let current_index = indices[current];
                    let next = indices.get(current + 1).copied();
                    is_application_boundary(previous, current_index, next, tokens, angles)
                })
            });
        if local_close.is_none() && record_like {
            if let Some(close) = global_close {
                let group = significant_indices(tokens, global_open, close + 1);
                let prefix = render_flat(&indices[..position], tokens, angles);
                let width = display_width(&prefix)
                    + usize::from(!prefix.is_empty())
                    + display_width(&render_flat(&group, tokens, angles))
                    + base_indent * 2;
                if width > line_width {
                    expanded.insert(position);
                }
            }
            continue;
        }
        let Some(close) = local_close else {
            continue;
        };
        if close <= position
            || (!record_like
                && !member_body
                && !application_group
                && !local.has_direct_comma(position, close, indices, tokens, angles))
        {
            continue;
        }
        let open_kind = tokens[indices[position]].kind;
        let group_width = if application_group
            || local.depth(position) == 0 && open_kind == TokenKind::PunctuationParenLeft
        {
            display_width(&render_flat(&indices[position..=close], tokens, angles))
        } else {
            group_line_width(position, close, indices, tokens, angles, local)
        };
        let rhs_continuation = top_level_equals(indices, tokens, angles).is_some_and(|equals| {
            equals < position
                && display_width(&render_flat(&indices[..=equals], tokens, angles))
                    + base_indent * 2
                    > line_width
        });
        let group_is_long = group_width
            + (base_indent + local.depth(position) + usize::from(rhs_continuation)) * 2
            > line_width;
        if member_body || group_is_long {
            expanded.insert(position);
        }
    }
}

fn group_line_width(
    open: usize,
    close: usize,
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    local: &LocalDelimiters,
) -> usize {
    if tokens[indices[open]].kind == TokenKind::PunctuationBraceLeft
        && is_record_like_brace(indices[open], tokens)
    {
        let mut start = open;
        while start > 0
            && matches!(
                tokens[indices[start - 1]].kind,
                TokenKind::IdentifierLower | TokenKind::IdentifierUpper | TokenKind::PunctuationDot
            )
        {
            start -= 1;
        }
        let record_start = start;
        let start = match start
            .checked_sub(1)
            .map(|position| &tokens[indices[position]])
        {
            Some(token)
                if matches!(
                    token.kind,
                    TokenKind::OperatorEquals | TokenKind::OperatorArrow
                ) =>
            {
                (0..start)
                    .rev()
                    .find(|position| {
                        tokens[indices[*position]].kind == TokenKind::PunctuationBraceLeft
                            && member_body_for_open(indices[*position], tokens).is_some()
                    })
                    .map_or(0, |position| position + 1)
            }
            Some(token) if token.kind == TokenKind::PunctuationColon => (0..start - 1)
                .rev()
                .find(|position| {
                    let kind = tokens[indices[*position]].kind;
                    kind == TokenKind::PunctuationComma
                        && local.depth(*position) == local.depth(open)
                        || local.depth(*position) + 1 == local.depth(open)
                            && is_open_delimiter(kind)
                })
                .map_or(0, |position| position + 1),
            _ => record_start,
        };
        return display_width(&render_flat(&indices[start..=close], tokens, angles));
    }

    if local.depth(open) == 0 {
        return display_width(&render_flat(indices, tokens, angles));
    }

    let depth = local.depth(open);
    let start = (0..open)
        .rev()
        .find(|position| {
            let kind = tokens[indices[*position]].kind;
            kind == TokenKind::PunctuationComma && local.depth(*position) == depth
                || local.depth(*position) + 1 == depth
                    && matches!(
                        kind,
                        TokenKind::PunctuationBraceLeft
                            | TokenKind::PunctuationSquareLeft
                            | TokenKind::PunctuationListLeft
                    )
        })
        .map_or(0, |position| position + 1);
    display_width(&render_flat(&indices[start..=close], tokens, angles))
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

fn is_application_boundary(
    previous: usize,
    current: usize,
    next: Option<usize>,
    tokens: &[Token],
    angles: &HashSet<usize>,
) -> bool {
    let left = &tokens[previous];
    let right = &tokens[current];
    if angles.contains(&previous) || angles.contains(&current) {
        return false;
    }
    let empty_unit = right.kind == TokenKind::PunctuationParenLeft
        && next.is_some_and(|next| tokens[next].kind == TokenKind::PunctuationParenRight);
    let record_constructor = right.kind == TokenKind::PunctuationBraceLeft
        && matches!(
            left.kind,
            TokenKind::IdentifierLower | TokenKind::IdentifierUpper
        )
        && brace_has_direct_field(current, tokens);
    !empty_unit
        && !record_constructor
        && ends_application_atom(left.kind)
        && starts_application_atom(right)
}

fn continues_operator_spelling(previous: usize, current: usize, tokens: &[Token]) -> bool {
    is_operator(tokens[previous].kind)
        && is_operator(tokens[current].kind)
        && tokens[previous].end == tokens[current].start
}

fn starts_composite_operator(position: usize, indices: &[usize], tokens: &[Token]) -> bool {
    let Some(next) = indices.get(position + 1).copied() else {
        return false;
    };
    let current = indices[position];
    tokens[current].end == tokens[next].start
        && (is_operator(tokens[current].kind) || tokens[current].raw == "<")
        && is_operator(tokens[next].kind)
}

fn application_atom_width(
    position: usize,
    index: usize,
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    local: &LocalDelimiters,
    expanded: &HashSet<usize>,
) -> usize {
    if is_open_delimiter(tokens[index].kind)
        && !angles.contains(&index)
        && !expanded.contains(&position)
    {
        if let Some(close) = local.matching_position(position) {
            return display_width(&render_flat(&indices[position..=close], tokens, angles));
        }
    }
    display_width(tokens[index].raw.trim_end())
}

fn ends_application_atom(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::IdentifierLower
            | TokenKind::IdentifierUpper
            | TokenKind::LiteralBoolean
            | TokenKind::LiteralFloat
            | TokenKind::LiteralInteger
            | TokenKind::LiteralString
            | TokenKind::LiteralTemplate
            | TokenKind::PunctuationBraceRight
            | TokenKind::PunctuationParenRight
            | TokenKind::PunctuationSquareRight
    )
}

fn starts_application_atom(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::IdentifierLower
            | TokenKind::IdentifierUpper
            | TokenKind::LiteralBoolean
            | TokenKind::LiteralFloat
            | TokenKind::LiteralInteger
            | TokenKind::LiteralString
            | TokenKind::LiteralTemplate
            | TokenKind::OperatorLambda
            | TokenKind::PunctuationBraceLeft
            | TokenKind::PunctuationListLeft
            | TokenKind::PunctuationParenLeft
            | TokenKind::PunctuationSquareLeft
    ) || is_prefix_operator_token(token)
}

fn is_prefix_operator_token(token: &Token) -> bool {
    matches!(
        (token.kind, token.raw.as_str()),
        (TokenKind::OperatorArithmetic, "-" | "*") | (TokenKind::OperatorCustom, "!")
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

fn is_callable_header(indices: &[usize], tokens: &[Token]) -> bool {
    indices
        .iter()
        .any(|index| tokens[*index].kind == TokenKind::KeywordFn)
        || indices
            .first()
            .is_some_and(|index| tokens[*index].raw == "operator")
}

fn rhs_segment_width(
    rhs: usize,
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    signature_needs_breaking: bool,
) -> usize {
    let start = rhs_segment_start(rhs, indices, tokens, angles, signature_needs_breaking);
    display_width(&render_flat(&indices[start..], tokens, angles))
}

fn rhs_segment_start(
    rhs: usize,
    indices: &[usize],
    tokens: &[Token],
    angles: &HashSet<usize>,
    signature_needs_breaking: bool,
) -> usize {
    let mut start = 0;
    if signature_needs_breaking {
        let mut depth = 0usize;
        for position in 0..rhs.saturating_sub(1) {
            let index = indices[position];
            if angles.contains(&index) {
                continue;
            }
            let kind = tokens[index].kind;
            if is_open_delimiter(kind) {
                depth += 1;
            } else if is_close_delimiter(kind) {
                depth = depth.saturating_sub(1);
            } else if depth == 0
                && (kind == TokenKind::OperatorArrow
                    || matches!(kind, TokenKind::KeywordWith | TokenKind::KeywordFails)
                    || tokens[index].raw == "where")
            {
                start = position;
            }
        }
    }
    start
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

    fn projected_width_by(&self, token_width: usize, index: usize) -> usize {
        self.current_width()
            + usize::from(
                self.previous
                    .is_some_and(|previous| needs_space(previous, index, self.tokens, self.angles)),
            )
            + token_width
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
            | TokenKind::OperatorLogical
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
        TokenKind::OperatorPipeline
            | TokenKind::OperatorApply
            | TokenKind::OperatorCustom
            | TokenKind::OperatorLogical
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
            | TokenKind::OperatorLogical
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
            if tokens[index].kind != TokenKind::PunctuationColon {
                continue;
            }
            let Some(rhs) = significant.get(position + 1).copied() else {
                continue;
            };
            if !matches!(
                tokens[rhs].kind,
                TokenKind::KeywordMatch | TokenKind::KeywordDo | TokenKind::PunctuationBraceLeft
            ) {
                continue;
            }
            let Some(open) = significant[position + 1..]
                .iter()
                .copied()
                .find(|candidate| tokens[*candidate].kind == TokenKind::PunctuationBraceLeft)
            else {
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
    if member_body_for_open(open, tokens).is_some() {
        return false;
    }
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
    indent: usize,
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
            lines.push(SourceLine {
                start,
                end: index,
                indent: source_indent(&tokens[start..index]),
            });
            start = index + 1;
        }
    }
    if start < eof {
        lines.push(SourceLine {
            start,
            end: eof,
            indent: source_indent(&tokens[start..eof]),
        });
    }
    lines
}

fn source_indent(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .take_while(|token| token.kind == TokenKind::TriviaSpace)
        .map(|token| display_width(&token.raw))
        .sum()
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
    // CST declaration ranges include trailing trivia. Standalone comments
    // after the last syntax token belong to the declaration boundary.
    let last_syntax = (declaration.start_token..declaration.end_token)
        .rev()
        .find(|index| {
            !is_trivia(tokens[*index].kind) && tokens[*index].kind != TokenKind::TriviaComment
        });
    if last_syntax.is_none_or(|last| first > last) {
        return 0;
    }
    let mut depth = 0usize;
    let mut follows_equals = false;
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
            follows_equals = first > index;
        }
    }
    usize::from(follows_equals && depth == 0)
}

fn structural_rhs_body_continuation(
    cst: &CstArtifact,
    first: usize,
    tokens: &[Token],
    angles: &HashSet<usize>,
    delimiters: &Delimiters,
    line_width: usize,
) -> usize {
    let Some(declaration) = cst
        .root
        .children
        .iter()
        .find(|node| node.start_token <= first && first < node.end_token)
    else {
        return 0;
    };
    let significant = significant_indices(tokens, declaration.start_token, declaration.end_token);
    let mut depth = 0usize;
    let mut equals = None;
    for (position, index) in significant.iter().copied().enumerate() {
        if angles.contains(&index) {
            continue;
        }
        let kind = tokens[index].kind;
        if depth == 0 && kind == TokenKind::OperatorEquals {
            equals = Some(position);
            break;
        }
        if is_open_delimiter(kind) {
            depth += 1;
        } else if is_close_delimiter(kind) {
            depth = depth.saturating_sub(1);
        }
    }
    let Some(equals) = equals else {
        return 0;
    };
    let Some(rhs_position) = (equals + 1..significant.len())
        .find(|position| tokens[significant[*position]].kind != TokenKind::TriviaComment)
    else {
        return 0;
    };
    let rhs = significant[rhs_position];
    if !matches!(
        tokens[rhs].kind,
        TokenKind::KeywordDo | TokenKind::KeywordMatch | TokenKind::PunctuationBraceLeft
    ) {
        return 0;
    }
    let Some(open_position) = significant[rhs_position..]
        .iter()
        .position(|index| tokens[*index].kind == TokenKind::PunctuationBraceLeft)
        .map(|position| position + rhs_position)
    else {
        return 0;
    };
    let open = significant[open_position];
    let Some(close) = delimiters.matching(open) else {
        return 0;
    };
    if first <= open || first > close {
        return 0;
    }
    let header = render_flat(&significant[..=open_position], tokens, angles);
    let separated_by_comment = significant[equals + 1..rhs_position]
        .iter()
        .any(|index| tokens[*index].kind == TokenKind::TriviaComment);
    usize::from(separated_by_comment || display_width(&header) > line_width)
}

fn leading_closers(indices: &[usize], tokens: &[Token], angles: &HashSet<usize>) -> usize {
    indices
        .iter()
        .take_while(|index| is_close_delimiter(tokens[**index].kind) && !angles.contains(index))
        .count()
}

#[cfg(test)]
mod tests {
    use seseragi_syntax::{lex, TokenKind};

    use super::{member_body_for_open, member_body_map, MemberBody};

    #[test]
    fn recognizes_implementation_body_boundaries() {
        let tokens = lex(
            "main.ssrg",
            "impl Score {\n  operator + self -> bonus: Int -> Score = Score { value: bonus }\n}\n",
        );
        let open = tokens
            .tokens
            .iter()
            .position(|token| token.kind == TokenKind::PunctuationBraceLeft)
            .expect("impl body open");
        let close = tokens
            .tokens
            .iter()
            .rposition(|token| token.kind == TokenKind::PunctuationBraceRight)
            .expect("impl body close");

        assert_eq!(
            member_body_for_open(open, &tokens.tokens),
            Some(MemberBody::Implementation)
        );
        assert_eq!(
            member_body_map(&tokens.tokens)[close],
            Some(MemberBody::Implementation)
        );
    }
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
