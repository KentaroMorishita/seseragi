#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompileInput<'source> {
    source_name: &'source str,
    module_id: &'source str,
    source: &'source str,
}

impl<'source> CompileInput<'source> {
    /// Creates a single-module compiler input.
    ///
    /// `module_id` is an opaque logical identity supplied by the caller. The
    /// package/project resolver owns canonicalization; the driver deliberately
    /// does not infer or validate an identity from a physical source name.
    pub const fn new(
        source_name: &'source str,
        module_id: &'source str,
        source: &'source str,
    ) -> Self {
        Self {
            source_name,
            module_id,
            source,
        }
    }

    /// Diagnostic label and frontend source spelling. Later artifacts may
    /// retain only the frontend's normalized file spelling.
    pub const fn source_name(&self) -> &'source str {
        self.source_name
    }

    /// Prevalidated logical identity supplied by a project resolver or an
    /// artifact harness.
    pub const fn module_id(&self) -> &'source str {
        self.module_id
    }

    pub const fn source(&self) -> &'source str {
        self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_physical_and_logical_source_identity_separate() {
        let input = CompileInput::new(
            "/tmp/seseragi-cache/entry.ssrg",
            "artifact/driver-input",
            "pub let answer: Int = 42\n",
        );

        assert_eq!(input.source_name(), "/tmp/seseragi-cache/entry.ssrg");
        assert_eq!(input.module_id(), "artifact/driver-input");
        assert_eq!(input.source(), "pub let answer: Int = 42\n");
    }
}
