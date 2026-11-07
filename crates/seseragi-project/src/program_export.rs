/// One selected program WASM export, as written after `--export` or in a
/// `web.programs.<name>.exports` entry.
///
/// - `label` exports the public function `label`.
/// - `label<Int>` exports the generic function `label` at the type
///   arguments `Int`, under the name `label`.
/// - `labelInt=label<Int>` exports the same instantiation under `labelInt`,
///   so one function can be exported at several type arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramExport {
    /// The name the host sees.
    pub name: String,
    /// The selected public function.
    pub function: String,
    /// The source text of the type arguments, without the angle brackets.
    pub type_arguments: Option<String>,
}

impl ProgramExport {
    pub fn parse(text: &str) -> Result<Self, String> {
        let (alias, selection) = match text.split_once('=') {
            Some((alias, selection)) => (Some(alias.trim()), selection.trim()),
            None => (None, text.trim()),
        };
        let (function, type_arguments) = match selection.split_once('<') {
            Some((function, rest)) => {
                let arguments = rest
                    .strip_suffix('>')
                    .filter(|arguments| !arguments.ends_with('-'))
                    .filter(|arguments| balanced(arguments) && !arguments.trim().is_empty())
                    .ok_or_else(|| {
                        format!("`{text}`: write type arguments as function<Type, ...>")
                    })?;
                (function.trim(), Some(arguments.trim().to_owned()))
            }
            None => (selection, None),
        };
        if !identifier(function) {
            return Err(format!("`{text}`: expected a public function name"));
        }
        if let Some(alias) = alias {
            if type_arguments.is_none() {
                return Err(format!(
                    "`{text}`: a renamed export selects type arguments, as name=function<Type>"
                ));
            }
            if !identifier(alias) {
                return Err(format!("`{text}`: expected an export name before `=`"));
            }
        }
        Ok(Self {
            name: alias.unwrap_or(function).to_owned(),
            function: function.to_owned(),
            type_arguments,
        })
    }
}

fn identifier(text: &str) -> bool {
    let mut characters = text.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn balanced(text: &str) -> bool {
    let mut depth = 0usize;
    let mut previous = ' ';
    for character in text.chars() {
        let arrow = previous == '-';
        previous = character;
        match character {
            '<' => depth += 1,
            // The `>` of a function type's `->` closes no type arguments.
            '>' if arrow => {}
            '>' => match depth.checked_sub(1) {
                Some(next) => depth = next,
                None => return false,
            },
            '=' => return false,
            _ => {}
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::ProgramExport;

    #[test]
    fn parses_plain_instantiated_and_renamed_exports() {
        let export = |name: &str, function: &str, arguments: Option<&str>| ProgramExport {
            name: name.into(),
            function: function.into(),
            type_arguments: arguments.map(Into::into),
        };
        for (text, expected) in [
            ("label", export("label", "label", None)),
            ("label<Int>", export("label", "label", Some("Int"))),
            (
                "labelPair=label<Map<String, Array<Int>>>",
                export("labelPair", "label", Some("Map<String, Array<Int>>")),
            ),
            (
                " both = pair < Int , domain.Score > ",
                export("both", "pair", Some("Int , domain.Score")),
            ),
            (
                "apply<Int -> Array<Int>>",
                export("apply", "apply", Some("Int -> Array<Int>")),
            ),
        ] {
            assert_eq!(ProgramExport::parse(text).unwrap(), expected, "{text}");
        }
        for text in [
            "",
            "label<>",
            "label<Int",
            "label<Int>>",
            "label<Int>x",
            "1label",
            "alias=label",
            "bad-name=label<Int>",
            "label<a=b>",
            "label<Int ->",
            "a=b=c<Int>",
        ] {
            assert!(ProgramExport::parse(text).is_err(), "{text}");
        }
    }
}
