use std::fmt::Write;

pub const IMPORTS: &str = r#"import * as arrays from "std/array"
import * as lists from "std/list"
import * as maps from "std/map"
import * as sets from "std/set"
import * as text from "std/text"
import * as chars from "std/char"
import * as floats from "std/float"
import * as effects from "std/effect"
"#;
const MAX: i128 = 9_007_199_254_740_991;
const WORDS: &[&str] = &[
    "", "🙂", "𐀀", "\u{ffff}", "e\u{301}", "é", "雪", "\0", "🌊", "\u{301}",
];

#[derive(Clone, Debug)]
pub struct Case {
    pub seed: u64,
    pub area: &'static str,
    pub body: String,
    pub expected: Option<Result<String, &'static str>>,
}
impl Case {
    pub fn standalone(&self) -> String {
        format!(
            "{IMPORTS}\nfn probe -> String = {}\npub effect fn main = println (probe ())\n",
            self.body
        )
    }
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut n = self.0;
        n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
        n ^ (n >> 31)
    }
    fn below(&mut self, limit: usize) -> usize {
        (self.next() % limit as u64) as usize
    }
    fn word(&mut self) -> &'static str {
        WORDS[self.below(WORDS.len())]
    }
}

fn quoted(value: &str, quote: char) -> String {
    let mut result = quote.to_string();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == ' ' {
            result.push(ch);
        } else {
            write!(result, "\\u{{{:x}}}", ch as u32).unwrap();
        }
    }
    result.push(quote);
    result
}
fn lines(expressions: &[String]) -> String {
    format!(
        "{{\n  text.join \"\\n\" [\n    {}\n  ]\n}}",
        expressions.join(",\n    ")
    )
}
fn int(seed: u64, random: &mut Random) -> Case {
    let edge = MAX - random.below(8) as i128;
    let small = (random.below(15) + 1) as i128;
    let sign = if random.below(2) == 0 { 1 } else { -1 };
    let (a, op, b) = match (seed / 5) % 8 {
        0 => (edge * sign, '+', 16 * sign),
        1 => (edge * sign, '-', -16 * sign),
        2 => (edge * sign, '*', small + 1),
        3 => (edge * sign, '+', -small * sign),
        4 => (edge, '/', if random.below(16) == 0 { 0 } else { small }),
        5 => (
            -edge,
            '/',
            if random.below(2) == 0 { small } else { -small },
        ),
        6 => (edge, '%', if random.below(16) == 0 { 0 } else { -small }),
        _ => (
            -edge,
            '%',
            if random.below(2) == 0 { small } else { -small },
        ),
    };
    // i128 computes exact results independently of either target's Int runtime.
    let expected = if b == 0 {
        Err(if op == '/' {
            "division-by-zero"
        } else {
            "remainder-by-zero"
        })
    } else {
        let value = match op {
            '+' => a + b,
            '-' => a - b,
            '*' => a * b,
            '/' => a / b,
            '%' => a % b,
            _ => unreachable!(),
        };
        if !(-MAX..=MAX).contains(&value) {
            Err("int-overflow")
        } else {
            Ok(format!("{value}\n"))
        }
    };
    let expression = match random.below(3) {
        0 => format!("({a}) {op} ({b})"),
        1 => format!("{{ let apply:Int -> Int -> Int = ({op}); apply ({a}) ({b}) }}"),
        _ => format!("{{ let left:Int = ({a}); let right:Int = ({b}); (left {op} right) + 0 }}"),
    };
    Case {
        seed,
        area: "int",
        body: format!("{{ show ({expression}) }}"),
        expected: Some(expected),
    }
}
fn float(seed: u64, random: &mut Random) -> Case {
    let base: f64 = [1e-6, 1e21, -1e-6, -1e21][random.below(4)];
    let offset = random.below(9) as i64 - 4;
    let near = f64::from_bits(base.to_bits().wrapping_add_signed(offset));
    let wide = f64::from_bits(random.next());
    let values = [
        "-0.0".to_owned(),
        "floats.nan ()".into(),
        "0.000001".into(),
        "0.0000001".into(),
        "1e20".into(),
        "1e21".into(),
        format!("{near:.17e}"),
        if wide.is_finite() {
            format!("{wide:.17e}")
        } else if wide.is_nan() {
            "floats.nan ()".into()
        } else if wide.is_sign_negative() {
            "floats.negativeInfinity ()".into()
        } else {
            "floats.positiveInfinity ()".into()
        },
    ];
    let mut expressions = Vec::new();
    for value in values {
        expressions.push(format!("show ({value})"));
        expressions.push(format!("debug ({value})"));
        expressions.push(format!("floats.format ({value})"));
    }
    Case {
        seed,
        area: "float",
        body: lines(&expressions),
        expected: None,
    }
}
fn unicode(seed: u64, random: &mut Random) -> Case {
    let value = format!("🙂e\u{301}{}{}", random.word(), random.word());
    let pattern = random.word();
    let replacement = random.word();
    let point = [0, 0x301, 0xd7ff, 0xe000, 0xffff, 0x10000, 0x1f642, 0x10ffff][random.below(8)];
    let ch = quoted(&char::from_u32(point).unwrap().to_string(), '\'');
    let value = quoted(&value, '"');
    let pattern = quoted(pattern, '"');
    let replacement = quoted(replacement, '"');
    let expressions = vec![
        format!("show (text.split \"\" {value})"),
        format!("show (text.split {pattern} {value})"),
        format!("text.replace {pattern} {replacement} {value}"),
        format!("show (text.startsWith {pattern} {value})"),
        format!("show (text.endsWith {pattern} {value})"),
        format!("show ({value} == ({value} + \"\"))"),
        format!("show ({value} == {pattern})"),
        format!("show {ch}"),
        format!("debug {ch}"),
        format!("show (chars.codePoint {ch})"),
        format!("chars.toString {ch}"),
        format!("show ({ch} < '🙂')"),
        format!("show (chars.fromCodePoint {point})"),
    ];
    Case {
        seed,
        area: "unicode",
        body: lines(&expressions),
        expected: None,
    }
}
fn ordering(seed: u64, random: &mut Random) -> Case {
    let count = if random.below(10) == 0 {
        0
    } else {
        random.below(8) + 3
    };
    let mut values: Vec<_> = (0..count)
        .map(|index| (index * 10 + random.below(3)).to_string())
        .collect();
    for index in (1..count).rev() {
        let other = random.below(index + 1);
        values.swap(index, other);
    }
    let array = format!("[{}]", values.join(","));
    let expressions = vec![
        "show (arrays.sort input)".into(),
        "show (lists.sort linked)".into(),
        "show (arrays.sortBy (\\n:Int -> n % 10) input)".into(),
        "show (lists.sortBy (\\n:Int -> n % 10) linked)".into(),
        "show (maps.entries (arrays.groupBy (\\n:Int -> n % 10) input))".into(),
        "show (maps.entries (lists.groupBy (\\n:Int -> n % 10) linked))".into(),
        "show input".into(),
        "show linked".into(),
    ];
    Case {
        seed,
        area: "ordering",
        body: format!(
            "{{\n  let input:Array<Int> = {array}\n  let linked:List<Int> = `{array}\n  {}\n}}",
            lines(&expressions)
        ),
        expected: None,
    }
}
fn insertion(seed: u64, random: &mut Random) -> Case {
    let count = random.below(6) + 3;
    let keys: Vec<_> = (0..3).map(|_| random.word()).collect();
    let mut body = String::from("{\n  let m0:maps.Map<String,Int> = maps.empty ()\n  let s0:sets.Set<String> = sets.empty ()\n");
    let mut expressions = Vec::new();
    for index in 1..=count {
        let key = quoted(keys[random.below(keys.len())], '"');
        let prev = index - 1;
        if random.below(4) == 0 {
            writeln!(body, "  let m{index} = maps.remove {key} m{prev}\n  let s{index} = sets.remove {key} s{prev}").unwrap();
        } else {
            writeln!(body, "  let m{index} = maps.insert {key} {index} m{prev}\n  let s{index} = sets.insert {key} s{prev}").unwrap();
        }
        expressions.push(format!("show (maps.entries m{index})"));
        expressions.push(format!("show (sets.toArray s{index})"));
    }
    expressions.extend([
        "show (maps.entries m0)".into(),
        "show (sets.toArray s0)".into(),
    ]);
    writeln!(body, "  {}\n}}", lines(&expressions)).unwrap();
    Case {
        seed,
        area: "insertion",
        body,
        expected: None,
    }
}

pub fn generate(seed: u64) -> Case {
    let mut random = Random(seed);
    match seed % 5 {
        0 => int(seed, &mut random),
        1 => float(seed, &mut random),
        2 => unicode(seed, &mut random),
        3 => ordering(seed, &mut random),
        _ => insertion(seed, &mut random),
    }
}

pub fn batch(cases: &[Case]) -> String {
    let mut source = IMPORTS.to_owned();
    for (index, case) in cases.iter().enumerate() {
        writeln!(source, "fn case{index} -> String = {}", case.body).unwrap();
    }
    source.push_str("pub effect fn main = do {\n  selected <- readLine () |> effects.mapError (\\_error:StdinError -> \"input failed\")\n  println (match selected {\n");
    for index in 0..cases.len() {
        writeln!(source, "    Just \"{index}\" -> case{index} ()").unwrap();
    }
    source.push_str("    _ -> \"invalid case selector\"\n  }) |> effects.mapError (\\_error:ConsoleError -> \"output failed\")\n}\n");
    source
}
