// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use regex::Regex;

use crate::dom::html_source::HtmlSource;
use crate::dom::nodes::ContainerNode;
use crate::dom::parser::parse_from_source;
use crate::{ComposerModel, ComposerUpdate, DomNode, Location, UnicodeString}; // Import the trait for to_tree

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Replaces text in the current selection with new_html.
    /// Treats its input as html that is parsed into a DomNode and inserted into
    /// the document at the cursor.
    pub fn replace_html(
        &mut self,
        new_html: S,
        external_source: HtmlSource,
    ) -> ComposerUpdate<S> {
        // Remove meta tags from the HTML which caused errors in html5ever
        let meta_regex = Regex::new(r"<meta[^>]*>").unwrap();
        let mut cleaned_html = meta_regex
            .replace_all(&new_html.to_string(), "")
            .to_string();

        if external_source == HtmlSource::GoogleDoc {
            // Strip outer b tag that google docs adds
            let b_regex = Regex::new(r"<b[^>]*>(.*)<\/b>").unwrap();
            cleaned_html = b_regex.replace(&cleaned_html, "$1").to_string();
        }

        // HTML we cannot parse (e.g. an unsupported tag from a Matrix source)
        // is not inserted, and leaves the model untouched.
        let Ok(dom) = parse_from_source(&cleaned_html, external_source) else {
            return ComposerUpdate::keep();
        };
        let DomNode::Container(doc) = dom.into_document_node() else {
            return ComposerUpdate::keep();
        };
        let new_children = doc.take_children();

        self.push_state_to_history();
        if self.has_selection() {
            self.do_replace_text(S::default());
        }
        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        // We should only have 1 dom node, so add the children under a paragraph to take advantage of the exisitng
        // insert_node_at_cursor api and then delete the paragraph node promoting it's the children up a level.
        let child_count = new_children.len();
        let p = DomNode::Container(ContainerNode::new_paragraph(new_children));

        let handle = self.state.dom.insert_node_at_cursor(&range, p);
        self.state.dom.replace_node_with_its_children(&handle);
        self.state.dom.wrap_inline_nodes_into_paragraphs_if_needed(
            &self.state.dom.parent(&handle).handle(),
        );

        // Place the cursor after the last inserted node, if there was one
        if child_count > 0 {
            let last_index = handle.index_in_parent() + child_count - 1;
            let last_handle = handle.parent_handle().child_handle(last_index);
            let location = self.state.dom.location_for_node(&last_handle);

            self.state.start =
                Location::from(location.position + location.length - 1);
            self.state.end = self.state.start;
        }
        // add a trailing space in cases when we do not have a next sibling
        self.create_update_replace_all()
    }
}

#[cfg(test)]
mod test {
    use crate::dom::html_source::HtmlSource;
    use crate::dom::parser::{
        GOOGLE_DOC_HTML_PASTEBOARD, MS_DOC_HTML_PASTEBOARD,
    };
    use crate::tests::testutils_composer_model::{cm, tx};

    #[test]
    fn test_replace_html_strips_meta_tags_google_docs() {
        let mut model = cm("|");

        // This html was copied directly from google docs and we are including the meta and bold tags that google docs adds.
        let html = format!(
            r#"<meta charset='utf-8'><meta charset="utf-8"><b style="font-weight:normal;" id="docs-internal-guid-bec65465-7fff-9422-b4bc-8e35d97b3ccb">{}</b>"#,
            GOOGLE_DOC_HTML_PASTEBOARD
        );

        let _ = model.replace_html(html.into(), HtmlSource::GoogleDoc);

        // Verify the HTML doesn't contain meta or the outer b tag
        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert!(!html_str.contains("<meta"));
        assert!(!html_str.contains("docs-internal-guid"));
        assert_eq!(html_str, "<ol><li><p><i>Italic</i></p></li><li><p><b>Bold</b></p></li><li><p>Unformatted</p></li><li><p><del>Strikethrough</del></p></li><li><p><u>Underlined</u></p></li><li><p><a style=\"text-decoration:none;\" href=\"http://matrix.org\"><u>Linked</u></a></p><ul><li><p>Nested</p></li></ul></li></ol>");
    }

    #[test]
    fn test_replace_html_strips_only_meta_tags_ms_docs() {
        let mut model = cm("|");

        // This html was copied directly from ms docs and we are including the meta and bold tags that ms docs adds.
        let html =
            format!(r#"<meta charset='utf-8'>{}"#, MS_DOC_HTML_PASTEBOARD);

        let _ = model.replace_html(html.into(), HtmlSource::UnknownExternal);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert!(!html_str.contains("<meta"));
        assert_eq!(html_str, "<ol start=\"1\"><li><p><i>Italic</i></p></li><li><p><b>Bold</b></p></li><li><p>Unformatted</p></li><li><p><del>Strikethrough</del></p></li><li><p><u>Underlined</u></p></li><li><p><a class=\"Hyperlink SCXW204127278 BCX0\" target=\"_blank\" rel=\"noreferrer noopener\" style=\"-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; cursor: text; text-decoration: none; color: inherit;\" href=\"https://matrix.org/\"><u>Linked</u></a></p></li></ol><ul><li><p>Nested</p></li></ul>");
    }

    #[test]
    fn test_replace_html_matrix_html_unchanged() {
        let mut model = cm("|");
        let matrix_html = "<p><strong>test</strong></p>";

        let _ = model.replace_html(matrix_html.into(), HtmlSource::Matrix);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(html_str, "<p><strong>test</strong></p>");
    }

    #[test]
    fn test_replace_html_with_existing_selection() {
        let mut model = cm("Hello{world}|test");
        let new_html = "<p><em>replacement</em></p>";

        let _ =
            model.replace_html(new_html.into(), HtmlSource::UnknownExternal);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(
            html_str,
            "<p>Hello</p><p><em>replacement</em></p><p>test</p>"
        );
    }

    #[test]
    fn test_replace_html_cursor_position_after_insert() {
        let mut model = cm("Start|");
        let new_html = "<strong>Bold text</strong>";
        let _ = model.replace_html(new_html.into(), HtmlSource::Matrix);
        // Cursor should be positioned after the inserted content
        let (start, end) = model.safe_selection();
        assert_eq!(start, end); // No selection, just cursor
        model.bold();
        model.enter();
        // Insert more text to verify cursor position
        let _ = model.replace_text("End".into());
        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(
            html_str,
            "<p>Start</p><p><strong>Bold text</strong></p><p>End</p>"
        );
    }

    #[test]
    fn test_replace_html_multiple_meta_tags() {
        let mut model = cm("|");
        let html_with_multiple_metas = r#"<meta charset="utf-8"><meta name="viewport" content="width=device-width"><meta http-equiv="X-UA-Compatible" content="IE=edge"><p>Content after metas</p>"#;

        let _ = model.replace_html(
            html_with_multiple_metas.into(),
            HtmlSource::UnknownExternal,
        );

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert!(!html_str.contains("<meta"));
        assert_eq!(html_str, "<p>Content after metas</p>");
    }

    #[test]
    fn test_replace_html_empty_content() {
        let mut model = cm("Existing content|");
        let empty_html = "";

        let _ = model.replace_html(empty_html.into(), HtmlSource::Matrix);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(html_str, "<p>Existing content</p>");
    }

    #[test]
    fn test_insert_list_item_without_list_parent() {
        let mut model = cm("hello|");
        let html = "<li>list item</li>";

        let _ = model.replace_html(html.into(), HtmlSource::UnknownExternal);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(html_str, "<p>hello</p><p>list item</p>");
    }

    #[test]
    fn test_replace_html_with_unparseable_html_does_nothing() {
        // Used to panic on the parse error.
        let mut model = cm("Hello{world}|test");
        let _ = model.replace_html("<h1>h</h1>".into(), HtmlSource::Matrix);
        assert_eq!(tx(&model), "Hello{world}|test");
    }

    #[test]
    fn test_replace_html_with_empty_html_does_nothing() {
        // Used to panic placing the cursor after the (missing) last node.
        let mut model = cm("|");
        let _ = model.replace_html("".into(), HtmlSource::Matrix);
        assert_eq!(tx(&model), "|");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use crate::dom::html_source::HtmlSource;
    use crate::tests::testutils_composer_model::cm;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_replace_html_with_existing_selection() {
        let mut model = cm("Hello{world}|test");
        let new_html = "<p><em>replacement</em></p>";

        let _ =
            model.replace_html(new_html.into(), HtmlSource::UnknownExternal);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(
            html_str,
            "<p>Hello</p><p><em>replacement</em></p><p>test</p>"
        );
    }

    #[wasm_bindgen_test]
    fn test_replace_html_cursor_position_after_insert() {
        let mut model = cm("Start|");
        let new_html = "<strong>Bold text</strong>";
        let _ = model.replace_html(new_html.into(), HtmlSource::Matrix);
        // Cursor should be positioned after the inserted content
        let (start, end) = model.safe_selection();
        assert_eq!(start, end); // No selection, just cursor
        model.bold();
        model.enter();
        // Insert more text to verify cursor position
        let _ = model.replace_text("End".into());
        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(
            html_str,
            "<p>Start</p><p><strong>Bold text</strong></p><p>End</p>"
        );
    }

    #[wasm_bindgen_test]
    fn test_replace_html_multiple_meta_tags() {
        let mut model = cm("|");
        let html_with_multiple_metas = r#"<meta charset="utf-8"><meta name="viewport" content="width=device-width"><meta http-equiv="X-UA-Compatible" content="IE=edge"><p>Content after metas</p>"#;

        let _ = model.replace_html(
            html_with_multiple_metas.into(),
            HtmlSource::UnknownExternal,
        );

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert!(!html_str.contains("<meta"));
        assert_eq!(html_str, "<p>Content after metas</p>");
    }

    #[wasm_bindgen_test]
    fn test_replace_html_empty_content() {
        let mut model = cm("Existing content|");
        let empty_html = "";

        let _ = model.replace_html(empty_html.into(), HtmlSource::Matrix);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(html_str, "<p>Existing content</p>");
    }

    #[wasm_bindgen_test]
    fn test_insert_list_item_without_list_parent() {
        let mut model = cm("hello|");
        let html = "<li>list item</li>";

        let _ = model.replace_html(html.into(), HtmlSource::UnknownExternal);

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert_eq!(html_str, "<p>hello</p><p>list item</p>");
    }
}
