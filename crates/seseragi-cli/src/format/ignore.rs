//! Directory walking for `seseragi format`.
//!
//! The walk skips folders that never hold user sources (`.git`, `.seseragi`,
//! `dist`, `node_modules`) and paths listed in `.gitignore` files. The
//! `.gitignore` support covers the common forms: comments, `!` negation,
//! a trailing `/` for directories, a leading or inner `/` to anchor a pattern
//! to its file's directory, and the `*`, `?` and `**` wildcards. The last
//! matching pattern wins, and a skipped directory is not entered.

use std::path::{Path, PathBuf};

const SKIPPED_DIRECTORIES: [&str; 4] = [".git", ".seseragi", "dist", "node_modules"];

struct Rule {
    /// The directory holding the `.gitignore` file.
    base: PathBuf,
    segments: Vec<String>,
    anchored: bool,
    directory_only: bool,
    negated: bool,
}

pub(super) fn walk(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut rules = ancestor_rules(root);
    visit(root, &mut rules, files)
}

fn visit(directory: &Path, rules: &mut Vec<Rule>, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let inherited = rules.len();
    read_rules(directory, rules);
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    let mut entries = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    entries.sort();
    for path in entries {
        // Links are not followed: they can point outside the package, and
        // format must only rewrite files that the paths given name.
        let Ok(file_type) = std::fs::symlink_metadata(&path).map(|metadata| metadata.file_type())
        else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let is_directory = file_type.is_dir();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if is_directory && SKIPPED_DIRECTORIES.contains(&name) {
            continue;
        }
        if ignored(rules, &path, is_directory) {
            continue;
        }
        if is_directory {
            visit(&path, rules, files)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("ssrg") {
            files.push(path);
        }
    }
    rules.truncate(inherited);
    Ok(())
}

/// Rules from `.gitignore` files above the walk root, up to the repository
/// root (the nearest directory holding `.git`). Outside a repository only the
/// walk root and its descendants contribute rules.
fn ancestor_rules(root: &Path) -> Vec<Rule> {
    let Ok(root) = root.canonicalize() else {
        return Vec::new();
    };
    if root.join(".git").exists() {
        return Vec::new();
    }
    let mut ancestors = Vec::new();
    for directory in root.ancestors().skip(1) {
        ancestors.push(directory);
        if directory.join(".git").exists() {
            let mut rules = Vec::new();
            for directory in ancestors.iter().rev() {
                read_rules(directory, &mut rules);
            }
            return rules;
        }
    }
    Vec::new()
}

fn read_rules(directory: &Path, rules: &mut Vec<Rule>) {
    let Ok(text) = std::fs::read_to_string(directory.join(".gitignore")) else {
        return;
    };
    let base = directory
        .canonicalize()
        .unwrap_or_else(|_| directory.to_owned());
    for line in text.lines() {
        if let Some(rule) = parse_rule(&base, line) {
            rules.push(rule);
        }
    }
}

fn parse_rule(base: &Path, line: &str) -> Option<Rule> {
    let line = line.trim_end();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (negated, line) = match line.strip_prefix('!') {
        Some(rest) => (true, rest),
        None => (false, line.strip_prefix('\\').unwrap_or(line)),
    };
    let (directory_only, line) = match line.strip_suffix('/') {
        Some(rest) => (true, rest),
        None => (false, line),
    };
    let anchored = line.contains('/');
    let line = line.strip_prefix('/').unwrap_or(line);
    if line.is_empty() {
        return None;
    }
    Some(Rule {
        base: base.to_owned(),
        segments: line.split('/').map(str::to_owned).collect(),
        anchored,
        directory_only,
        negated,
    })
}

fn ignored(rules: &[Rule], path: &Path, is_directory: bool) -> bool {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_owned());
    let mut ignored = false;
    for rule in rules {
        if rule.directory_only && !is_directory {
            continue;
        }
        let Ok(relative) = canonical.strip_prefix(&rule.base) else {
            continue;
        };
        let segments = relative
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .collect::<Vec<_>>();
        let matched = if rule.anchored {
            match_segments(&rule.segments, &segments)
        } else {
            segments
                .last()
                .is_some_and(|name| match_glob(&rule.segments[0], name))
        };
        if matched {
            ignored = !rule.negated;
        }
    }
    ignored
}

fn match_segments(pattern: &[String], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        // A trailing `**` matches what is inside a directory, not the
        // directory itself; an inner `**` may match no segments.
        Some((first, rest)) if first == "**" && rest.is_empty() => !path.is_empty(),
        Some((first, rest)) if first == "**" => {
            (0..=path.len()).any(|skip| match_segments(rest, &path[skip..]))
        }
        Some((first, rest)) => path
            .split_first()
            .is_some_and(|(name, tail)| match_glob(first, name) && match_segments(rest, tail)),
    }
}

/// Matches one path segment against `*` and `?` wildcards.
fn match_glob(pattern: &str, name: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let name = name.chars().collect::<Vec<_>>();
    let (mut p, mut n) = (0, 0);
    let mut star = None;
    while n < name.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == name[n]) {
            p += 1;
            n += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some((p, n));
            p += 1;
        } else if let Some((star_p, star_n)) = star {
            p = star_p + 1;
            n = star_n + 1;
            star = Some((star_p, star_n + 1));
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|character| *character == '*')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(line: &str) -> Rule {
        parse_rule(Path::new("/repo"), line).unwrap()
    }

    #[test]
    fn matches_common_gitignore_forms() {
        let rules = vec![
            rule("*.tmp"),
            rule("build/"),
            rule("/root-only.ssrg"),
            rule("docs/**/draft.ssrg"),
            rule("keep*"),
            rule("!keep-me.ssrg"),
        ];
        let check = |path: &str, directory: bool| {
            ignored(&rules, &Path::new("/repo").join(path), directory)
        };
        assert!(check("a/b.tmp", false));
        assert!(check("a/build", true));
        assert!(!check("a/build", false));
        assert!(check("root-only.ssrg", false));
        assert!(!check("a/root-only.ssrg", false));
        assert!(check("docs/draft.ssrg", false));
        assert!(check("docs/a/b/draft.ssrg", false));
        assert!(check("keep-out.ssrg", false));
        assert!(!check("keep-me.ssrg", false));
        assert!(!check("src/main.ssrg", false));

        let rules = vec![rule("generated/**"), rule("!generated/keep.ssrg")];
        let check = |path: &str, directory: bool| {
            ignored(&rules, &Path::new("/repo").join(path), directory)
        };
        assert!(!check("generated", true));
        assert!(check("generated/out.ssrg", false));
        assert!(!check("generated/keep.ssrg", false));
    }

    #[test]
    fn matches_wildcards_inside_a_segment() {
        assert!(match_glob("*.ssrg", "main.ssrg"));
        assert!(match_glob("m?in.*", "main.ssrg"));
        assert!(!match_glob("*.ssrg", "main.ts"));
        assert!(match_glob("a*b*c", "aXXbYYc"));
        assert!(!match_glob("a*b*c", "aXXbYY"));
    }
}
