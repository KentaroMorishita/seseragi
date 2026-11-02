enum NativeRegexInstruction {
    Consume(NativeRegexMatcher, usize),
    Jump(usize),
    Split(usize, usize),
    Save(usize, usize),
    Assert(NativeRegexAssertion, usize),
    Reset(usize, usize),
    CounterSplit {
        counter: usize,
        minimum: i64,
        maximum: Option<i64>,
        body: usize,
        exit: usize,
        slots: Vec<usize>,
    },
    Increment(usize, usize),
    Accept,
}

struct NativeRegexFragment {
    start: usize,
    exits: Vec<(usize, bool)>,
}

#[derive(Clone)]
struct NativeRegexThread {
    instruction: usize,
    start: usize,
    captures: Vec<Option<usize>>,
    counts: Vec<i64>,
    iteration_starts: Vec<Option<usize>>,
    iteration_captures: Vec<Option<Vec<Option<usize>>>>,
}

struct NativeRegexMatch {
    start: usize,
    end: usize,
    captures: Vec<Option<usize>>,
}

impl NativeRegexNode {
    fn capture_slots(&self, slots: &mut Vec<usize>) {
        match self {
            Self::Capture(index, child) => {
                slots.extend([index * 2, index * 2 + 1]);
                child.capture_slots(slots);
            }
            Self::Concat(children) | Self::Alternate(children) => {
                for child in children {
                    child.capture_slots(slots);
                }
            }
            Self::Repeat(child, _, _) => child.capture_slots(slots),
            _ => {}
        }
    }
}

impl NativeRegex {
    fn emit(&mut self, instruction: NativeRegexInstruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }

    fn patch(&mut self, exits: Vec<(usize, bool)>, target: usize) {
        for (index, exit) in exits {
            let next = match &mut self.instructions[index] {
                NativeRegexInstruction::CounterSplit { exit: next, .. } if exit => next,
                NativeRegexInstruction::Consume(_, next)
                | NativeRegexInstruction::Jump(next)
                | NativeRegexInstruction::Save(_, next)
                | NativeRegexInstruction::Assert(_, next)
                | NativeRegexInstruction::Reset(_, next)
                    if !exit =>
                {
                    next
                }
                _ => panic!("invalid Regex compiler patch"),
            };
            *next = target;
        }
    }

    fn compile_node(&mut self, node: &NativeRegexNode) -> NativeRegexFragment {
        use NativeRegexInstruction as I;
        use NativeRegexNode as N;
        let single = match node {
            N::Empty => Some(I::Jump(usize::MAX)),
            N::Consume(matcher) => Some(I::Consume(matcher.clone(), usize::MAX)),
            N::Assert(assertion) => Some(I::Assert(*assertion, usize::MAX)),
            _ => None,
        };
        if let Some(instruction) = single {
            let start = self.emit(instruction);
            return NativeRegexFragment {
                start,
                exits: vec![(start, false)],
            };
        }
        match node {
            N::Concat(parts) => {
                let mut result = self.compile_node(&parts[0]);
                for part in &parts[1..] {
                    let next = self.compile_node(part);
                    self.patch(result.exits, next.start);
                    result.exits = next.exits;
                }
                result
            }
            N::Alternate(branches) => {
                let mut fragments: Vec<_> = branches
                    .iter()
                    .map(|branch| self.compile_node(branch))
                    .collect();
                let mut result = fragments.pop().unwrap();
                for mut first in fragments.into_iter().rev() {
                    let start = self.emit(I::Split(first.start, result.start));
                    first.exits.extend(result.exits);
                    result = NativeRegexFragment {
                        start,
                        exits: first.exits,
                    };
                }
                result
            }
            N::Capture(index, child) => {
                let open = self.emit(I::Save(index * 2, usize::MAX));
                let body = self.compile_node(child);
                let close = self.emit(I::Save(index * 2 + 1, usize::MAX));
                self.patch(vec![(open, false)], body.start);
                self.patch(body.exits, close);
                NativeRegexFragment {
                    start: open,
                    exits: vec![(close, false)],
                }
            }
            N::Repeat(child, minimum, maximum) => {
                let counter = self.counter_count;
                self.counter_count += 1;
                let reset = self.emit(I::Reset(counter, usize::MAX));
                let mut slots = vec![];
                child.capture_slots(&mut slots);
                let split = self.emit(I::CounterSplit {
                    counter,
                    minimum: *minimum,
                    maximum: *maximum,
                    body: usize::MAX,
                    exit: usize::MAX,
                    slots,
                });
                let body = self.compile_node(child);
                let increment = self.emit(I::Increment(counter, split));
                self.patch(vec![(reset, false)], split);
                let I::CounterSplit { body: target, .. } = &mut self.instructions[split] else {
                    unreachable!()
                };
                *target = body.start;
                self.patch(body.exits, increment);
                NativeRegexFragment {
                    start: reset,
                    exits: vec![(split, true)],
                }
            }
            _ => unreachable!(),
        }
    }

    fn assertion_matches(
        &self,
        assertion: NativeRegexAssertion,
        position: usize,
        input: &[char],
    ) -> bool {
        use NativeRegexAssertion::*;
        match assertion {
            AbsoluteStart => position == 0,
            AbsoluteEnd => position == input.len(),
            Start if !self.options.multiline => position == 0,
            End if !self.options.multiline => position == input.len(),
            Start => {
                position == 0
                    || (native_regex_line_terminator(input[position - 1])
                        && !(input[position - 1] == '\r' && input.get(position) == Some(&'\n')))
            }
            End => {
                position == input.len()
                    || (native_regex_line_terminator(input[position])
                        && !(input[position] == '\n'
                            && position.checked_sub(1).map(|i| input[i]) == Some('\r')))
            }
        }
    }

    fn closure(
        &self,
        input: &[char],
        position: usize,
        seeds: Vec<NativeRegexThread>,
        mut matched: Option<NativeRegexMatch>,
    ) -> (Vec<NativeRegexThread>, Option<NativeRegexMatch>) {
        use NativeRegexInstruction as I;
        let mut stack: Vec<_> = seeds.into_iter().rev().collect();
        let mut seen = std::collections::HashSet::new();
        let mut consuming = vec![];
        let mut cutoff = None;
        while let Some(mut thread) = stack.pop() {
            if matched.as_ref().is_some_and(|m| thread.start > m.start)
                || cutoff.is_some_and(|c| thread.start >= c)
            {
                continue;
            }
            if !seen.insert((
                thread.instruction,
                thread.counts.clone(),
                thread.iteration_starts.clone(),
            )) {
                continue;
            }
            match &self.instructions[thread.instruction] {
                I::Consume(_, _) => consuming.push(thread),
                I::Jump(next) => {
                    thread.instruction = *next;
                    stack.push(thread);
                }
                I::Split(first, second) => {
                    let mut rest = thread.clone();
                    rest.instruction = *second;
                    thread.instruction = *first;
                    stack.push(rest);
                    stack.push(thread);
                }
                I::Save(slot, next) => {
                    thread.instruction = *next;
                    thread.captures[*slot] = Some(position);
                    stack.push(thread);
                }
                I::Assert(assertion, next) => {
                    if self.assertion_matches(*assertion, position, input) {
                        thread.instruction = *next;
                        stack.push(thread);
                    }
                }
                I::Reset(counter, next) => {
                    thread.instruction = *next;
                    thread.counts[*counter] = 0;
                    thread.iteration_starts[*counter] = None;
                    thread.iteration_captures[*counter] = None;
                    stack.push(thread);
                }
                I::CounterSplit {
                    counter,
                    minimum,
                    maximum,
                    body,
                    exit,
                    slots,
                } => {
                    let count = thread.counts[*counter];
                    let enter_body = count < *minimum || maximum.is_none_or(|m| count < m);
                    if count >= *minimum {
                        let mut exiting = thread.clone();
                        exiting.exit_counter(*counter, *exit);
                        stack.push(exiting);
                    }
                    if enter_body {
                        thread.instruction = *body;
                        thread.iteration_captures[*counter] = Some(thread.captures.clone());
                        for &slot in slots {
                            thread.captures[slot] = None;
                        }
                        thread.iteration_starts[*counter] = Some(position);
                        stack.push(thread);
                    }
                }
                I::Increment(counter, split) => {
                    let previous = thread.counts[*counter];
                    let I::CounterSplit {
                        minimum,
                        maximum,
                        exit,
                        ..
                    } = &self.instructions[*split]
                    else {
                        unreachable!()
                    };
                    let count = (previous + 1).min(maximum.unwrap_or(*minimum));
                    if thread.iteration_starts[*counter] == Some(position) {
                        if previous >= *minimum {
                            if let Some(captures) = thread.iteration_captures[*counter].take() {
                                thread.captures = captures;
                            }
                        }
                        thread.exit_counter(*counter, *exit);
                    } else {
                        thread.instruction = *split;
                        thread.counts[*counter] = count;
                    }
                    stack.push(thread);
                }
                I::Accept => {
                    cutoff = Some(thread.start);
                    matched = Some(NativeRegexMatch {
                        start: thread.start,
                        end: position,
                        captures: thread.captures,
                    });
                }
            }
        }
        (consuming, matched)
    }

    fn find(&self, input: &[char]) -> Option<NativeRegexMatch> {
        self.find_from(input, 0)
    }

    fn find_from(&self, input: &[char], first_start: usize) -> Option<NativeRegexMatch> {
        let mut seeds = vec![];
        let mut matched = None;
        for position in first_start..=input.len() {
            if matched.is_none() {
                seeds.push(NativeRegexThread {
                    instruction: self.start,
                    start: position,
                    captures: vec![None; self.capture_count * 2],
                    counts: vec![-1; self.counter_count],
                    iteration_starts: vec![None; self.counter_count],
                    iteration_captures: vec![None; self.counter_count],
                });
            }
            let (consuming, found) = self.closure(input, position, seeds, matched);
            matched = found;
            if position == input.len() {
                return matched;
            }
            seeds = vec![];
            for mut thread in consuming {
                if matched.as_ref().is_some_and(|m| thread.start > m.start) {
                    continue;
                }
                let NativeRegexInstruction::Consume(matcher, next) =
                    &self.instructions[thread.instruction]
                else {
                    unreachable!()
                };
                if matcher.matches(input[position], self.options) {
                    thread.instruction = *next;
                    seeds.push(thread);
                }
            }
            if seeds.is_empty() && matched.is_some() {
                return matched;
            }
        }
        matched
    }

    fn find_all(&self, input: &[char]) -> Vec<NativeRegexMatch> {
        let mut matches = Vec::new();
        let mut next = 0;
        while next <= input.len() {
            let Some(matched) = self.find_from(input, next) else {
                break;
            };
            let (start, end) = (matched.start, matched.end);
            matches.push(matched);
            if end > start {
                next = end;
            } else if end < input.len() {
                next = end + 1;
            } else {
                break;
            }
        }
        matches
    }
}

impl NativeRegexThread {
    fn exit_counter(&mut self, counter: usize, exit: usize) {
        self.instruction = exit;
        self.counts[counter] = -1;
        self.iteration_starts[counter] = None;
        self.iteration_captures[counter] = None;
    }
}
