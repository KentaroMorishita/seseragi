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
    operation!("customTag", "web.html.custom-tag", "_ssrg_html_customTag"),
    operation!("attribute", "web.html.attribute", "_ssrg_html_attribute"),
    operation!(
        "parseWebUrl",
        "web.html.parse-url",
        "_ssrg_html_parseWebUrl"
    ),
    operation!("custom", "web.html.custom", "_ssrg_html_custom"),
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
    operation!("select", "web.html.select", "_ssrg_html_select"),
    operation!("option", "web.html.option", "_ssrg_html_option"),
    operation!("fieldset", "web.html.fieldset", "_ssrg_html_fieldset"),
    operation!("legend", "web.html.legend", "_ssrg_html_legend"),
    operation!("table", "web.html.table", "_ssrg_html_table"),
    operation!("thead", "web.html.thead", "_ssrg_html_thead"),
    operation!("tbody", "web.html.tbody", "_ssrg_html_tbody"),
    operation!("tfoot", "web.html.tfoot", "_ssrg_html_tfoot"),
    operation!("tr", "web.html.tr", "_ssrg_html_tr"),
    operation!("th", "web.html.th", "_ssrg_html_th"),
    operation!("td", "web.html.td", "_ssrg_html_td"),
    operation!("caption", "web.html.caption", "_ssrg_html_caption"),
    operation!("details", "web.html.details", "_ssrg_html_details"),
    operation!("summary", "web.html.summary", "_ssrg_html_summary"),
    operation!("dialog", "web.html.dialog", "_ssrg_html_dialog"),
    operation!(
        "IgnoreEvent",
        "web.html.event.ignore",
        "_ssrg_html_IgnoreEvent"
    ),
    operation!("Dispatch", "web.html.event.dispatch", "_ssrg_html_Dispatch"),
    operation!(
        "DispatchPreventDefault",
        "web.html.event.dispatch-prevent-default",
        "_ssrg_html_DispatchPreventDefault"
    ),
    operation!(
        "DispatchStopPropagation",
        "web.html.event.dispatch-stop-propagation",
        "_ssrg_html_DispatchStopPropagation"
    ),
    operation!(
        "DispatchPreventDefaultAndStop",
        "web.html.event.dispatch-prevent-default-and-stop",
        "_ssrg_html_DispatchPreventDefaultAndStop"
    ),
    operation!(
        "InvalidTagName",
        "web.html.error.invalid-tag-name",
        "_ssrg_html_InvalidTagName"
    ),
    operation!(
        "InvalidAttributeName",
        "web.html.error.invalid-attribute-name",
        "_ssrg_html_InvalidAttributeName"
    ),
    operation!(
        "ReservedAttributeName",
        "web.html.error.reserved-attribute-name",
        "_ssrg_html_ReservedAttributeName"
    ),
    operation!(
        "UnsafeWebUrlScheme",
        "web.html.error.unsafe-url-scheme",
        "_ssrg_html_UnsafeWebUrlScheme"
    ),
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
    dom_operation!("mount", "web.dom.mount", "_ssrg_dom_mount"),
    dom_operation!("awaitMount", "web.dom.await-mount", "_ssrg_dom_awaitMount"),
    dom_operation!("unmount", "web.dom.unmount", "_ssrg_dom_unmount"),
    dom_operation!("content", "web.dom.content", "_ssrg_dom_content"),
    dom_operation!(
        "initialHtml",
        "web.dom.initial-html",
        "_ssrg_dom_initialHtml"
    ),
    dom_operation!("bindText", "web.dom.bind-text", "_ssrg_dom_bindText"),
    dom_operation!(
        "bindAttribute",
        "web.dom.bind-attribute",
        "_ssrg_dom_bindAttribute"
    ),
    dom_operation!("bindValue", "web.dom.bind-value", "_ssrg_dom_bindValue"),
    dom_operation!(
        "bindChecked",
        "web.dom.bind-checked",
        "_ssrg_dom_bindChecked"
    ),
    dom_operation!("bindStyle", "web.dom.bind-style", "_ssrg_dom_bindStyle"),
    dom_operation!("bindRegion", "web.dom.bind-region", "_ssrg_dom_bindRegion"),
    dom_operation!(
        "mountContent",
        "web.dom.mount-content",
        "_ssrg_dom_mountContent"
    ),
    dom_operation!("runContent", "web.dom.run-content", "_ssrg_dom_runContent"),
    dom_operation!("run", "web.dom.run", "_ssrg_dom_run"),
    dom_operation!("app", "web.dom.app", "_ssrg_dom_app"),
    dom_operation!("FreshMount", "web.dom.fresh-mount", "_ssrg_dom_FreshMount"),
    dom_operation!(
        "HydrateStrict",
        "web.dom.hydrate-strict",
        "_ssrg_dom_HydrateStrict"
    ),
    dom_operation!(
        "HydrateOrReplace",
        "web.dom.hydrate-or-replace",
        "_ssrg_dom_HydrateOrReplace"
    ),
    dom_operation!(
        "ClearRenderedDom",
        "web.dom.clear-rendered-dom",
        "_ssrg_dom_ClearRenderedDom"
    ),
    dom_operation!(
        "PreserveRenderedDom",
        "web.dom.preserve-rendered-dom",
        "_ssrg_dom_PreserveRenderedDom"
    ),
    dom_operation!(
        "InvalidSelector",
        "web.dom.invalid-selector",
        "_ssrg_dom_InvalidSelector"
    ),
    dom_operation!(
        "DomTargetNotFound",
        "web.dom.target-not-found",
        "_ssrg_dom_DomTargetNotFound"
    ),
    dom_operation!(
        "DomTargetAlreadyMounted",
        "web.dom.target-already-mounted",
        "_ssrg_dom_DomTargetAlreadyMounted"
    ),
    dom_operation!(
        "HydrationMismatch",
        "web.dom.hydration-mismatch",
        "_ssrg_dom_HydrationMismatch"
    ),
    dom_operation!(
        "DomEventQueueOverflow",
        "web.dom.event-queue-overflow",
        "_ssrg_dom_DomEventQueueOverflow"
    ),
    dom_operation!(
        "DomTargetRemoved",
        "web.dom.target-removed",
        "_ssrg_dom_DomTargetRemoved"
    ),
    dom_operation!(
        "DomOperationFailed",
        "web.dom.operation-failed",
        "_ssrg_dom_DomOperationFailed"
    ),
    dom_operation!("DomFailure", "web.dom.failure", "_ssrg_dom_DomFailure"),
    dom_operation!(
        "DispatchFailure",
        "web.dom.dispatch-failure",
        "_ssrg_dom_DispatchFailure"
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

        let parse = runtime_web_html_operation("std/web/html::parseWebUrl").unwrap();
        assert_eq!(parse.runtime_feature, "web.html.parse-url");
        let unsafe_scheme = runtime_web_html_operation("std/web/html::UnsafeWebUrlScheme").unwrap();
        assert_eq!(
            unsafe_scheme.runtime_feature,
            "web.html.error.unsafe-url-scheme"
        );

        let dom = runtime_web_html_operation("std/web/dom::run").unwrap();
        assert_eq!(dom.runtime_feature, "web.dom.run");
        assert_eq!(dom.module, "@seseragi/runtime/dom");

        let mount = runtime_web_html_operation("std/web/dom::mount").unwrap();
        assert_eq!(mount.runtime_feature, "web.dom.mount");
        assert_eq!(mount.local_name, "_ssrg_dom_mount");

        let content = runtime_web_html_operation("std/web/dom::content").unwrap();
        assert_eq!(content.runtime_feature, "web.dom.content");
        assert_eq!(content.local_name, "_ssrg_dom_content");

        let leaf = runtime_web_html_operation("std/web/dom::bindValue").unwrap();
        assert_eq!(leaf.runtime_feature, "web.dom.bind-value");
        assert_eq!(leaf.local_name, "_ssrg_dom_bindValue");

        let region = runtime_web_html_operation("std/web/dom::bindRegion").unwrap();
        assert_eq!(region.runtime_feature, "web.dom.bind-region");
        assert_eq!(region.local_name, "_ssrg_dom_bindRegion");

        let mount_content = runtime_web_html_operation("std/web/dom::mountContent").unwrap();
        assert_eq!(mount_content.runtime_feature, "web.dom.mount-content");
        assert_eq!(mount_content.local_name, "_ssrg_dom_mountContent");

        let hydration = runtime_web_html_operation("std/web/dom::HydrateStrict").unwrap();
        assert_eq!(hydration.runtime_feature, "web.dom.hydrate-strict");

        let mismatch = runtime_web_html_operation("std/web/dom::HydrationMismatch").unwrap();
        assert_eq!(mismatch.runtime_feature, "web.dom.hydration-mismatch");

        let app = runtime_web_html_operation("std/web/dom::app").unwrap();
        assert_eq!(app.runtime_feature, "web.dom.app");
        assert_eq!(app.module, "@seseragi/runtime/dom");
    }
}
