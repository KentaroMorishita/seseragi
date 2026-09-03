pub fn with_unicode_header(imports: &str, declarations: &str) -> String {
    format!(
        "import {{ assertUnicodeVersion as $ssrg$assertUnicodeVersion }} from \"@seseragi/runtime/unicode-version\"\n{imports}$ssrg$assertUnicodeVersion({:?})\n\n{declarations}",
        seseragi_syntax::unicode::UNICODE_VERSION
    )
}
