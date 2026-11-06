//! The linker and rustflags that Cargo uses for the host target, resolved
//! with Cargo's rules so native builds can add their own flags without
//! replacing the user's settings.
//! <https://doc.rust-lang.org/cargo/reference/config.html>

use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs,
    path::Path,
};

/// One `rustc --print cfg` entry: `unix` is `("unix", None)` and
/// `target_os="macos"` is `("target_os", Some("macos"))`.
pub(super) type CfgEntry = (String, Option<String>);

#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct LinkerSettings {
    /// `target.<host>.linker`, else the matching `target.'cfg(..)'.linker`.
    pub linker: Option<String>,
    /// The rustflags Cargo passes to rustc for the host target.
    pub rustflags: Vec<String>,
}

/// Settings collected for one `target.<key>` table across config sources.
#[derive(Default)]
struct TargetTable {
    linker: Option<String>,
    rustflags: MergedFlags,
}

/// A rustflags value merged across config sources as Cargo merges it: a
/// nearer string replaces a farther one, arrays are joined with the farther
/// values first, and a string and an array for the same key do not merge.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum MergedFlags {
    #[default]
    Unset,
    Text(Vec<String>),
    Array(Vec<String>),
    /// Cargo rejects the configuration, so no flags are taken from it.
    Mismatch,
}

impl MergedFlags {
    /// Merges a nearer config file's value.
    fn merge(&mut self, value: &toml::Value) {
        *self = match (std::mem::take(self), value) {
            (Self::Unset | Self::Text(_), toml::Value::String(flags)) => Self::Text(words(flags)),
            (previous @ (Self::Unset | Self::Array(_)), toml::Value::Array(values)) => {
                let flags = values
                    .iter()
                    .map(|flag| flag.as_str().map(String::from))
                    .collect::<Option<Vec<_>>>();
                match (previous, flags) {
                    (Self::Array(mut joined), Some(flags)) => {
                        joined.extend(flags);
                        Self::Array(joined)
                    }
                    (_, Some(flags)) => Self::Array(flags),
                    // A non-string element is a type error for Cargo.
                    (_, None) => Self::Mismatch,
                }
            }
            _ => Self::Mismatch,
        };
    }

    /// Appends an environment value, which Cargo adds after config values.
    fn append(&mut self, flags: &str) {
        match self {
            Self::Unset => *self = Self::Array(words(flags)),
            Self::Text(joined) | Self::Array(joined) => joined.extend(words(flags)),
            Self::Mismatch => {}
        }
    }

    fn flags(&self) -> &[String] {
        match self {
            Self::Text(flags) | Self::Array(flags) => flags,
            Self::Unset | Self::Mismatch => &[],
        }
    }
}

/// Cargo asks rustc for the target cfg with the selected rustflags, and
/// `target.'cfg(..)'` tables can change those rustflags, so the query repeats
/// until the cfg no longer changes.
pub(super) fn settle_with_cfg(
    query_cfg: &dyn Fn(&[String]) -> Vec<CfgEntry>,
    resolve: &dyn Fn(&[CfgEntry]) -> LinkerSettings,
) -> LinkerSettings {
    let mut cfg = query_cfg(&[]);
    let mut settings = resolve(&cfg);
    for _ in 0..4 {
        let next = query_cfg(&settings.rustflags);
        if next == cfg {
            break;
        }
        cfg = next;
        settings = resolve(&cfg);
    }
    settings
}

/// Resolves the settings with Cargo's rules.
///
/// - Config files are read from each `.cargo/` from the build directory up to
///   the file system root, then `CARGO_HOME`. A nearer file wins. In one
///   directory, `config` wins over `config.toml`. See [`MergedFlags`] for
///   how rustflags values merge.
/// - Environment variables override config files.
/// - The linker is `target.<host>.linker`, else the matching
///   `target.'cfg(..)'.linker`.
/// - Rustflags are the first of `CARGO_ENCODED_RUSTFLAGS` or `RUSTFLAGS`
///   when set, else the joined `target.<host>.rustflags` and matching
///   `target.'cfg(..)'.rustflags` when not empty, else `build.rustflags`.
///
/// A file Cargo cannot parse is skipped here; Cargo reports it when the build
/// starts.
pub(super) fn linker_settings(
    directory: &Path,
    cargo_home: &Path,
    host: &str,
    cfg: &[CfgEntry],
    environment: &dyn Fn(&str) -> Option<OsString>,
) -> LinkerSettings {
    let mut directories = directory
        .ancestors()
        .map(|ancestor| ancestor.join(".cargo"))
        .collect::<Vec<_>>();
    if !directories.iter().any(|candidate| candidate == cargo_home) {
        directories.push(cargo_home.to_owned());
    }
    let mut targets = BTreeMap::<String, TargetTable>::new();
    let mut build_flags = MergedFlags::Unset;
    // Lowest precedence first, so later values override or append.
    for directory in directories.iter().rev() {
        let Some(config) = [directory.join("config"), directory.join("config.toml")]
            .into_iter()
            .find(|file| file.is_file())
            .and_then(|file| fs::read_to_string(file).ok())
            .and_then(|source| source.parse::<toml::Table>().ok())
        else {
            continue;
        };
        if let Some(tables) = config.get("target").and_then(toml::Value::as_table) {
            for (key, table) in tables {
                let entry = targets.entry(key.clone()).or_default();
                if let Some(linker) = table.get("linker").and_then(toml::Value::as_str) {
                    entry.linker = Some(linker.to_owned());
                }
                if let Some(flags) = table.get("rustflags") {
                    entry.rustflags.merge(flags);
                }
            }
        }
        if let Some(flags) = config.get("build").and_then(|build| build.get("rustflags")) {
            build_flags.merge(flags);
        }
    }
    let text = |name: &str| environment(name).map(|value| value.to_string_lossy().into_owned());
    let key = host.to_uppercase().replace(['-', '.'], "_");
    let host_table = targets.entry(host.to_owned()).or_default();
    if let Some(linker) = text(&format!("CARGO_TARGET_{key}_LINKER")) {
        host_table.linker = Some(linker);
    }
    if let Some(flags) = text(&format!("CARGO_TARGET_{key}_RUSTFLAGS")) {
        host_table.rustflags.append(&flags);
    }
    if let Some(flags) = text("CARGO_BUILD_RUSTFLAGS") {
        build_flags.append(&flags);
    }

    let matching = targets
        .iter()
        .filter(|(key, _)| cfg_matches(key, cfg))
        .map(|(_, table)| table)
        .collect::<Vec<_>>();
    let linker = targets[host]
        .linker
        .clone()
        .or_else(|| matching.iter().rev().find_map(|table| table.linker.clone()));

    let encoded = environment("CARGO_ENCODED_RUSTFLAGS");
    let plain = environment("RUSTFLAGS");
    let rustflags = if encoded.is_some() || plain.is_some() {
        environment_flags(encoded.as_deref(), plain.as_deref())
    } else {
        let mut target_flags = targets[host].rustflags.flags().to_vec();
        for table in &matching {
            target_flags.extend(table.rustflags.flags().iter().cloned());
        }
        // Cargo falls back to build.rustflags when the target flags are empty.
        if target_flags.is_empty() {
            build_flags.flags().to_vec()
        } else {
            target_flags
        }
    };
    LinkerSettings { linker, rustflags }
}

/// Splits the rustflags environment variables with Cargo's precedence.
pub(super) fn environment_flags(encoded: Option<&OsStr>, plain: Option<&OsStr>) -> Vec<String> {
    match (encoded, plain) {
        (Some(flags), _) => flags
            .to_string_lossy()
            .split('\x1f')
            .filter(|flag| !flag.is_empty())
            .map(Into::into)
            .collect(),
        (None, Some(flags)) => words(&flags.to_string_lossy()),
        (None, None) => Vec::new(),
    }
}

/// Parses `rustc --print cfg` output.
pub(super) fn parse_cfg(output: &str) -> Vec<CfgEntry> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| match line.split_once('=') {
            Some((name, value)) => (name.to_owned(), Some(value.trim_matches('"').to_owned())),
            None => (line.to_owned(), None),
        })
        .collect()
}

fn words(flags: &str) -> Vec<String> {
    flags.split_whitespace().map(Into::into).collect()
}

/// Whether a `target` key is a `cfg(..)` expression that holds for `cfg`.
/// A target triple or a malformed expression does not match; Cargo itself
/// rejects a malformed expression.
fn cfg_matches(key: &str, cfg: &[CfgEntry]) -> bool {
    let Some(expression) = key
        .trim()
        .strip_prefix("cfg(")
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        return false;
    };
    let mut parser = CfgParser {
        tokens: tokenize(expression).unwrap_or_default(),
        position: 0,
    };
    if parser.tokens.is_empty() {
        return false;
    }
    match parser.expression(cfg) {
        Some(value) if parser.position == parser.tokens.len() => value,
        _ => false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    Identifier(String),
    Text(String),
    Open,
    Close,
    Comma,
    Equals,
}

fn tokenize(source: &str) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut characters = source.chars().peekable();
    while let Some(&character) = characters.peek() {
        match character {
            ' ' | '\t' | '\n' | '\r' => {
                characters.next();
            }
            '(' | ')' | ',' | '=' => {
                characters.next();
                tokens.push(match character {
                    '(' => Token::Open,
                    ')' => Token::Close,
                    ',' => Token::Comma,
                    _ => Token::Equals,
                });
            }
            '"' => {
                characters.next();
                let mut text = String::new();
                loop {
                    match characters.next()? {
                        '"' => break,
                        character => text.push(character),
                    }
                }
                tokens.push(Token::Text(text));
            }
            character if character == '_' || character.is_alphanumeric() => {
                let mut name = String::new();
                while let Some(&character) = characters.peek() {
                    if character == '_' || character.is_alphanumeric() {
                        name.push(character);
                        characters.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(name));
            }
            _ => return None,
        }
    }
    Some(tokens)
}

/// `cfg` expressions: `all(..)`, `any(..)`, `not(..)`, `name` and
/// `name = "value"`, with optional trailing commas in lists.
struct CfgParser {
    tokens: Vec<Token>,
    position: usize,
}

impl CfgParser {
    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += 1;
        token
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn expression(&mut self, cfg: &[CfgEntry]) -> Option<bool> {
        let Some(Token::Identifier(name)) = self.next() else {
            return None;
        };
        match (name.as_str(), self.peek()) {
            ("all" | "any" | "not", Some(Token::Open)) => {
                self.next();
                let mut values = Vec::new();
                loop {
                    if self.peek() == Some(&Token::Close) {
                        self.next();
                        break;
                    }
                    values.push(self.expression(cfg)?);
                    match self.next()? {
                        Token::Comma => {}
                        Token::Close => break,
                        _ => return None,
                    }
                }
                match name.as_str() {
                    "all" => Some(values.iter().all(|value| *value)),
                    "any" => Some(values.iter().any(|value| *value)),
                    _ if values.len() == 1 => Some(!values[0]),
                    _ => None,
                }
            }
            (_, Some(Token::Equals)) => {
                self.next();
                let Some(Token::Text(value)) = self.next() else {
                    return None;
                };
                Some(cfg.iter().any(|(entry, entry_value)| {
                    *entry == name && entry_value.as_deref() == Some(value.as_str())
                }))
            }
            _ => Some(
                cfg.iter()
                    .any(|(entry, entry_value)| *entry == name && entry_value.is_none()),
            ),
        }
    }
}
