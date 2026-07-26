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
const DOM_MODULE: &str = "@seseragi/runtime/dom";

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

macro_rules! dom_operation {
    ($name:literal, $feature:literal, $local:literal) => {
        RuntimeWebHtmlOperation {
            canonical: concat!("std/web/dom::", $name),
            runtime_feature: $feature,
            local_name: $local,
            module: DOM_MODULE,
            export_name: $name,
            source_map_name: $name,
        }
    };
}

const OPERATIONS: &[RuntimeWebHtmlOperation] = &[
    operation!("style", "web.html.style", "_ssrg_html_style"),
    operation!("text", "web.html.text", "_ssrg_html_text"),
    operation!("fragment", "web.html.fragment", "_ssrg_html_fragment"),
    operation!("html", "web.html.html", "_ssrg_html_html"),
    operation!("head", "web.html.head", "_ssrg_html_head"),
    operation!("body", "web.html.body", "_ssrg_html_body"),
    operation!("title", "web.html.title", "_ssrg_html_title"),
    operation!("meta", "web.html.meta", "_ssrg_html_meta"),
    operation!("link", "web.html.link", "_ssrg_html_link"),
    operation!("header", "web.html.header", "_ssrg_html_header"),
    operation!("footer", "web.html.footer", "_ssrg_html_footer"),
    operation!("nav", "web.html.nav", "_ssrg_html_nav"),
    operation!("article", "web.html.article", "_ssrg_html_article"),
    operation!("aside", "web.html.aside", "_ssrg_html_aside"),
    operation!("h3", "web.html.h3", "_ssrg_html_h3"),
    operation!("h4", "web.html.h4", "_ssrg_html_h4"),
    operation!("h5", "web.html.h5", "_ssrg_html_h5"),
    operation!("h6", "web.html.h6", "_ssrg_html_h6"),
    operation!("strong", "web.html.strong", "_ssrg_html_strong"),
    operation!("em", "web.html.em", "_ssrg_html_em"),
    operation!("small", "web.html.small", "_ssrg_html_small"),
    operation!("code", "web.html.code", "_ssrg_html_code"),
    operation!("pre", "web.html.pre", "_ssrg_html_pre"),
    operation!("blockquote", "web.html.blockquote", "_ssrg_html_blockquote"),
    operation!("ul", "web.html.ul", "_ssrg_html_ul"),
    operation!("ol", "web.html.ol", "_ssrg_html_ol"),
    operation!("li", "web.html.li", "_ssrg_html_li"),
    operation!("br", "web.html.br", "_ssrg_html_br"),
    operation!("hr", "web.html.hr", "_ssrg_html_hr"),
    operation!("a", "web.html.a", "_ssrg_html_a"),
    operation!("img", "web.html.img", "_ssrg_html_img"),
    operation!("picture", "web.html.picture", "_ssrg_html_picture"),
    operation!("source", "web.html.source", "_ssrg_html_source"),
    operation!("video", "web.html.video", "_ssrg_html_video"),
    operation!("audio", "web.html.audio", "_ssrg_html_audio"),
    operation!("div", "web.html.div", "_ssrg_html_div"),
    operation!("span", "web.html.span", "_ssrg_html_span"),
    operation!("p", "web.html.p", "_ssrg_html_p"),
    operation!("main", "web.html.main", "_ssrg_html_main"),
    operation!("section", "web.html.section", "_ssrg_html_section"),
    operation!("h1", "web.html.h1", "_ssrg_html_h1"),
    operation!("h2", "web.html.h2", "_ssrg_html_h2"),
    operation!("button", "web.html.button", "_ssrg_html_button"),
    operation!("form", "web.html.form", "_ssrg_html_form"),
    operation!("label", "web.html.label", "_ssrg_html_label"),
    operation!("input", "web.html.input", "_ssrg_html_input"),
    operation!("textarea", "web.html.textarea", "_ssrg_html_textarea"),
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
    dom_operation!(
        "defaultOptions",
        "web.dom.default-options",
        "_ssrg_dom_defaultOptions"
    ),
    dom_operation!("query", "web.dom.query", "_ssrg_dom_query"),
    dom_operation!("run", "web.dom.run", "_ssrg_dom_run"),
    dom_operation!("app", "web.dom.app", "_ssrg_dom_app"),
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

        let dom = runtime_web_html_operation("std/web/dom::run").unwrap();
        assert_eq!(dom.runtime_feature, "web.dom.run");
        assert_eq!(dom.module, "@seseragi/runtime/dom");

        let app = runtime_web_html_operation("std/web/dom::app").unwrap();
        assert_eq!(app.runtime_feature, "web.dom.app");
        assert_eq!(app.module, "@seseragi/runtime/dom");
    }
}
