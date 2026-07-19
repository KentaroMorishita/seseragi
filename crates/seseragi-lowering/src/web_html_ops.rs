#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeWebHtmlOperation {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const MODULE: &str = "@seseragi/runtime/html";

macro_rules! operation {
    ($name:literal, $feature:literal, $local:literal) => {
        RuntimeWebHtmlOperation {
            canonical: concat!("std/web/html::", $name),
            runtime_feature: $feature,
            local_name: $local,
            module: MODULE,
            export_name: $name,
            source_map_name: $name,
        }
    };
}

const OPERATIONS: &[RuntimeWebHtmlOperation] = &[
    operation!("style", "web.html.style", "_ssrg_html_style"),
    operation!("text", "web.html.text", "_ssrg_html_text"),
    operation!("fragment", "web.html.fragment", "_ssrg_html_fragment"),
    operation!("div", "web.html.div", "_ssrg_html_div"),
    operation!("span", "web.html.span", "_ssrg_html_span"),
    operation!("p", "web.html.p", "_ssrg_html_p"),
    operation!("main", "web.html.main", "_ssrg_html_main"),
    operation!("section", "web.html.section", "_ssrg_html_section"),
    operation!("h1", "web.html.h1", "_ssrg_html_h1"),
    operation!("h2", "web.html.h2", "_ssrg_html_h2"),
    operation!("button", "web.html.button", "_ssrg_html_button"),
    operation!("input", "web.html.input", "_ssrg_html_input"),
    operation!(
        "renderToString",
        "web.html.render-to-string",
        "_ssrg_html_renderToString"
    ),
    operation!(
        "renderDocument",
        "web.html.render-document",
        "_ssrg_html_renderDocument"
    ),
];

pub(crate) fn runtime_web_html_operation(canonical: &str) -> Option<RuntimeWebHtmlOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
}

pub(crate) fn runtime_web_html_operation_for_feature(
    feature: &str,
) -> Option<RuntimeWebHtmlOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_html_calls_by_canonical_language_identity() {
        let operation = runtime_web_html_operation("std/web/html::renderToString").unwrap();

        assert_eq!(operation.runtime_feature, "web.html.render-to-string");
        assert_eq!(operation.module, "@seseragi/runtime/html");
        assert_eq!(
            runtime_web_html_operation_for_feature(operation.runtime_feature),
            Some(operation)
        );
    }
}
