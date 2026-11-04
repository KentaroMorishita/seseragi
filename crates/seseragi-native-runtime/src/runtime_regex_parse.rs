#[derive(Clone)]
enum NativeRegexNode {
    Empty,
    Consume(NativeRegexMatcher),
    Assert(NativeRegexAssertion),
    Concat(Vec<NativeRegexNode>),
    Alternate(Vec<NativeRegexNode>),
    Capture(usize, Box<NativeRegexNode>),
    Repeat(Box<NativeRegexNode>, i64, Option<i64>),
}

struct NativeRegexError {
    kind: &'static str,
    value: Option<String>,
    offset: usize,
}

impl NativeRegexError {
    fn value(self) -> V {
        let payload = match self.value {
            Some(value) if self.kind == "std/regex::UnexpectedRegexToken" => {
                V::Char(value.chars().next().unwrap())
            }
            Some(value) => V::text(&value),
            None => V::Unit,
        };
        native_regex_record(vec![
            ("kind", V::adt(self.kind, payload)),
            ("offset", V::Int(self.offset as i64)),
        ])
    }
}

struct NativeRegexParser<'a> {
    pattern: &'a str,
    scalars: Vec<(usize, char)>,
    cursor: usize,
    captures: usize,
    named: Vec<(String, usize)>,
}

impl<'a> NativeRegexParser<'a> {
    fn new(pattern: &'a str) -> Self {
        Self {
            pattern,
            scalars: pattern.char_indices().collect(),
            cursor: 0,
            captures: 0,
            named: vec![],
        }
    }

    fn error(&self, kind: &'static str, offset: usize) -> NativeRegexError {
        NativeRegexError {
            kind,
            offset,
            value: None,
        }
    }

    fn unsupported(&self, value: &str, offset: usize) -> NativeRegexError {
        NativeRegexError {
            kind: "std/regex::UnsupportedRegexFeature",
            value: Some(value.into()),
            offset,
        }
    }

    fn unexpected(&self, value: char, offset: usize) -> NativeRegexError {
        NativeRegexError {
            kind: "std/regex::UnexpectedRegexToken",
            value: Some(value.to_string()),
            offset,
        }
    }

    fn offset(&self) -> usize {
        self.scalars
            .get(self.cursor)
            .map_or(self.pattern.len(), |s| s.0)
    }

    fn peek(&self) -> Option<char> {
        self.scalars.get(self.cursor).map(|s| s.1)
    }

    fn advance(&mut self) -> Result<(usize, char), NativeRegexError> {
        let scalar = self
            .scalars
            .get(self.cursor)
            .copied()
            .ok_or_else(|| self.error("std/regex::UnexpectedRegexEnd", self.offset()))?;
        self.cursor += 1;
        Ok(scalar)
    }

    fn take(&mut self, value: char) -> bool {
        if self.peek() == Some(value) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn parse(&mut self) -> Result<NativeRegexNode, NativeRegexError> {
        let node = self.alternation()?;
        if let Some(value) = self.peek() {
            return Err(self.unexpected(value, self.offset()));
        }
        Ok(node)
    }

    fn alternation(&mut self) -> Result<NativeRegexNode, NativeRegexError> {
        let mut branches = vec![self.concatenation()?];
        while self.take('|') {
            branches.push(self.concatenation()?);
        }
        Ok(if branches.len() == 1 {
            branches.pop().unwrap()
        } else {
            NativeRegexNode::Alternate(branches)
        })
    }

    fn concatenation(&mut self) -> Result<NativeRegexNode, NativeRegexError> {
        let mut parts = vec![];
        while !matches!(self.peek(), None | Some(')' | '|')) {
            parts.push(self.repetition()?);
        }
        Ok(match parts.len() {
            0 => NativeRegexNode::Empty,
            1 => parts.pop().unwrap(),
            _ => NativeRegexNode::Concat(parts),
        })
    }

    fn repetition(&mut self) -> Result<NativeRegexNode, NativeRegexError> {
        let (node, quantifiable) = self.atom()?;
        if !matches!(self.peek(), Some('*' | '+' | '?' | '{')) {
            return Ok(node);
        }
        let offset = self.offset();
        if !quantifiable {
            return Err(self.error("std/regex::InvalidRegexQuantifier", offset));
        }
        let (minimum, maximum) = if self.take('*') {
            (0, None)
        } else if self.take('+') {
            (1, None)
        } else if self.take('?') {
            (0, Some(1))
        } else {
            self.take('{');
            let minimum = self.quantifier_number(offset)?;
            let maximum = if self.take('}') {
                Some(minimum)
            } else {
                if !self.take(',') {
                    return Err(self.error("std/regex::InvalidRegexQuantifier", offset));
                }
                let maximum = if self.peek() == Some('}') {
                    None
                } else {
                    Some(self.quantifier_number(offset)?)
                };
                if !self.take('}') || maximum.is_some_and(|maximum| maximum < minimum) {
                    return Err(self.error("std/regex::InvalidRegexQuantifier", offset));
                }
                maximum
            };
            (minimum, maximum)
        };
        match self.peek() {
            Some('?') => return Err(self.unsupported("lazy quantifier", self.offset())),
            Some('+') => return Err(self.unsupported("possessive quantifier", self.offset())),
            Some('*' | '{') => {
                return Err(self.error("std/regex::InvalidRegexQuantifier", self.offset()))
            }
            _ => {}
        }
        Ok(NativeRegexNode::Repeat(Box::new(node), minimum, maximum))
    }

    fn quantifier_number(&mut self, offset: usize) -> Result<i64, NativeRegexError> {
        if !self.peek().is_some_and(|c| c.is_ascii_digit()) {
            return Err(self.error("std/regex::InvalidRegexQuantifier", offset));
        }
        let mut value = 0;
        while let Some(c) = self.peek().filter(|c| c.is_ascii_digit()) {
            self.cursor += 1;
            value = value * 10 + i64::from(c as u32 - '0' as u32);
            if value > 9_007_199_254_740_991 {
                return Err(self.error("std/regex::InvalidRegexQuantifier", offset));
            }
        }
        Ok(value)
    }

    fn atom(&mut self) -> Result<(NativeRegexNode, bool), NativeRegexError> {
        let (offset, value) = self.advance()?;
        Ok(match value {
            '(' => (self.group()?, true),
            '[' => (self.character_class()?, true),
            '.' => (
                NativeRegexNode::Consume(NativeRegexMatcher {
                    terms: vec![],
                    negated: false,
                    dot: true,
                }),
                true,
            ),
            '^' => (NativeRegexNode::Assert(NativeRegexAssertion::Start), false),
            '$' => (NativeRegexNode::Assert(NativeRegexAssertion::End), false),
            '\\' => self.escape(false, offset)?,
            ')' | ']' => return Err(self.unexpected(value, offset)),
            '*' | '+' | '?' | '{' => {
                return Err(self.error("std/regex::InvalidRegexQuantifier", offset))
            }
            _ => (Self::literal(value), true),
        })
    }

    fn literal(value: char) -> NativeRegexNode {
        Self::term(NativeRegexTerm::Range(value as u32, value as u32))
    }

    fn term(term: NativeRegexTerm) -> NativeRegexNode {
        NativeRegexNode::Consume(NativeRegexMatcher {
            terms: vec![term],
            negated: false,
            dot: false,
        })
    }

    fn group(&mut self) -> Result<NativeRegexNode, NativeRegexError> {
        let mut capturing = true;
        let mut name = None;
        if self.take('?') {
            let offset = self.scalars[self.cursor - 1].0;
            if self.take(':') {
                capturing = false;
            } else if self.take('<') {
                if matches!(self.peek(), Some('=' | '!')) {
                    return Err(self.unsupported("look-around", offset));
                }
                name = Some(self.capture_name(offset)?);
            } else {
                let feature = match self.peek() {
                    Some('=' | '!') => "look-around",
                    Some('>') => "atomic group",
                    Some('(') => "conditional",
                    Some('R' | '0'..='9') => "recursion",
                    _ => "inline flag",
                };
                return Err(self.unsupported(feature, offset));
            }
        }
        let index = self.captures;
        self.captures += usize::from(capturing);
        if let Some((name, offset)) = name {
            if self.named.iter().any(|(previous, _)| previous == &name) {
                return Err(NativeRegexError {
                    kind: "std/regex::DuplicateCaptureName",
                    value: Some(name),
                    offset,
                });
            }
            self.named.push((name, index));
        }
        let child = self.alternation()?;
        if !self.take(')') {
            return Err(match self.peek() {
                None => self.error("std/regex::UnexpectedRegexEnd", self.offset()),
                Some(value) => self.unexpected(value, self.offset()),
            });
        }
        Ok(if capturing {
            NativeRegexNode::Capture(index, Box::new(child))
        } else {
            child
        })
    }

    fn capture_name(&mut self, question: usize) -> Result<(String, usize), NativeRegexError> {
        let offset = self.offset();
        let start = self.cursor;
        while !matches!(self.peek(), None | Some('>')) {
            self.cursor += 1;
        }
        if self.peek().is_none() {
            return Err(self.error("std/regex::UnexpectedRegexEnd", self.offset()));
        }
        let values = &self.scalars[start..self.cursor];
        let valid_start = |c| c == '_' || native_regex_alphabetic(c);
        if values.first().is_none_or(|s| !valid_start(s.1))
            || values
                .iter()
                .skip(1)
                .any(|s| !valid_start(s.1) && !(5..=8).contains(&native_regex_category(s.1)))
        {
            return Err(self.error("std/regex::InvalidRegexEscape", question));
        }
        let name = values.iter().map(|s| s.1).collect();
        self.cursor += 1;
        Ok((name, offset))
    }

    fn character_class(&mut self) -> Result<NativeRegexNode, NativeRegexError> {
        let negated = self.take('^');
        let mut terms = vec![];
        loop {
            match self.peek() {
                None => return Err(self.error("std/regex::UnexpectedRegexEnd", self.offset())),
                Some(']') => {
                    if terms.is_empty() {
                        return Err(self.unexpected(']', self.offset()));
                    }
                    self.cursor += 1;
                    break;
                }
                _ => {}
            }
            let (left, left_point) = self.class_atom()?;
            if self.peek() == Some('-')
                && self.scalars.get(self.cursor + 1).map(|s| s.1) != Some(']')
            {
                let offset = self.advance()?.0;
                let (_, right_point) = self.class_atom()?;
                match (left_point, right_point) {
                    (Some(left), Some(right)) if left <= right => {
                        terms.push(NativeRegexTerm::Range(left, right))
                    }
                    _ => return Err(self.error("std/regex::InvalidRegexRange", offset)),
                }
            } else {
                terms.push(left);
            }
        }
        Ok(NativeRegexNode::Consume(NativeRegexMatcher {
            terms,
            negated,
            dot: false,
        }))
    }

    fn class_atom(&mut self) -> Result<(NativeRegexTerm, Option<u32>), NativeRegexError> {
        let (offset, value) = self.advance()?;
        let term = if value == '\\' {
            let (NativeRegexNode::Consume(mut matcher), _) = self.escape(true, offset)? else {
                return Err(self.error("std/regex::InvalidRegexEscape", offset));
            };
            assert_eq!(matcher.terms.len(), 1);
            matcher.terms.pop().unwrap()
        } else {
            NativeRegexTerm::Range(value as u32, value as u32)
        };
        let point = match term {
            NativeRegexTerm::Range(start, end) if start == end => Some(start),
            _ => None,
        };
        Ok((term, point))
    }

    fn escape(
        &mut self,
        in_class: bool,
        offset: usize,
    ) -> Result<(NativeRegexNode, bool), NativeRegexError> {
        let (_, value) = self.advance()?;
        let node = match value {
            'd' | 'D' => Self::term(NativeRegexTerm::Digit(value == 'D')),
            's' | 'S' => Self::term(NativeRegexTerm::Space(value == 'S')),
            'w' | 'W' => Self::term(NativeRegexTerm::Word(value == 'W')),
            'p' | 'P' => {
                if !self.take('{') {
                    return Err(self.error("std/regex::InvalidRegexEscape", offset));
                }
                let start = self.cursor;
                while !matches!(self.peek(), None | Some('}')) {
                    self.cursor += 1;
                }
                if self.peek().is_none() {
                    return Err(self.error("std/regex::UnexpectedRegexEnd", self.offset()));
                }
                let name: String = self.scalars[start..self.cursor]
                    .iter()
                    .map(|s| s.1)
                    .collect();
                self.cursor += 1;
                let property = native_regex_property(&name)
                    .ok_or_else(|| self.error("std/regex::InvalidRegexEscape", offset))?;
                Self::term(NativeRegexTerm::Property(property, value == 'P'))
            }
            'A' | 'z' => {
                if in_class {
                    return Err(self.error("std/regex::InvalidRegexEscape", offset));
                }
                return Ok((
                    NativeRegexNode::Assert(if value == 'A' {
                        NativeRegexAssertion::AbsoluteStart
                    } else {
                        NativeRegexAssertion::AbsoluteEnd
                    }),
                    false,
                ));
            }
            'n' => Self::literal('\n'),
            'r' => Self::literal('\r'),
            't' => Self::literal('\t'),
            'f' => Self::literal('\u{c}'),
            'v' => Self::literal('\u{b}'),
            '0' => {
                if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    return Err(self.unsupported("backreference", offset));
                }
                Self::literal('\0')
            }
            'u' => Self::literal(self.unicode_escape(offset)?),
            'b' if in_class => Self::literal('\u{8}'),
            'b' | 'B' => return Err(self.unsupported("word boundary", offset)),
            'k' | '1'..='9' => return Err(self.unsupported("backreference", offset)),
            c if c.is_ascii_alphabetic() => {
                return Err(self.error("std/regex::InvalidRegexEscape", offset))
            }
            _ => Self::literal(value),
        };
        Ok((node, true))
    }

    fn unicode_escape(&mut self, offset: usize) -> Result<char, NativeRegexError> {
        if !self.take('{') {
            return Err(self.error("std/regex::InvalidRegexEscape", offset));
        }
        let mut value = 0;
        let mut digits = 0;
        while !matches!(self.peek(), None | Some('}')) {
            let (_, scalar) = self.advance()?;
            let Some(digit) = scalar.to_digit(16).filter(|_| scalar.is_ascii()) else {
                return Err(self.error("std/regex::InvalidRegexEscape", offset));
            };
            if digits == 6 {
                return Err(self.error("std/regex::InvalidRegexEscape", offset));
            }
            value = value * 16 + digit;
            digits += 1;
        }
        if !self.take('}') || digits == 0 {
            return Err(self.error("std/regex::InvalidRegexEscape", offset));
        }
        char::from_u32(value).ok_or_else(|| self.error("std/regex::InvalidRegexEscape", offset))
    }
}
