// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use regex::Regex;

use crate::dom::html_source::HtmlSource;
use crate::dom::parser::parse_from_source;
use crate::{parse, ComposerModel, ComposerUpdate, Location, UnicodeString};

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
        self.push_state_to_history();
        if self.has_selection() {
            self.do_replace_text(S::default());
        }

        let meta_regex = Regex::new(r"<meta[^>]*>").unwrap();
        let mut cleaned_html = meta_regex
            .replace_all(&new_html.to_string(), "")
            .to_string();

        if external_source == HtmlSource::GoogleDoc {
            // Strip first b tag (opening and closing)
            let b_regex = Regex::new(r"<b[^>]*>(.*)<\/b>").unwrap();
            cleaned_html = b_regex.replace(&cleaned_html, "$1").to_string();
        }

        println!("cleaned_html: {}", cleaned_html);
        let result = if external_source == HtmlSource::Matrix {
            parse(&cleaned_html.to_string())
        } else {
            parse_from_source(&cleaned_html.to_string(), external_source)
        };

        // We should have only one top level dom node, so add each of the children at the cursor.
        let dom_children = result.unwrap().into_container().take_children();

        for node in dom_children.iter() {
            let (start, end) = self.safe_selection();
            let range = self.state.dom.find_range(start, end);

            let new_cursor_index = start + node.text_len();
            let _ = self.state.dom.insert_node_at_cursor(&range, node.clone());

            // manually move the cursor to the end of the html
            self.state.start = Location::from(new_cursor_index);
            self.state.end = self.state.start;
        }

        // add a trailing space in cases when we do not have a next sibling
        self.create_update_replace_all()
    }
}

#[cfg(test)]
const GOOGLE_DOC_HTML_PASTEBOARD: &str = r#"<meta charset='utf-8'><meta charset="utf-8"><b style="font-weight:normal;" id="docs-internal-guid-c640886a-7fff-1a1b-2de3-d7820da258bf"><span style="font-size:12pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:700;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">test</span></b>"#;
#[cfg(test)]
const MS_DOC_HTML_PASTEBOARD: &str = r#"<meta charset='utf-8'><span data-contrast="auto" xml:lang="EN-GB" lang="EN-GB" class="TextRun MacChromeBold SCXW254497905 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; -webkit-font-smoothing: antialiased; font-variant-ligatures: none !important; color: rgb(0, 0, 0); font-style: normal; font-variant-caps: normal; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: pre-wrap; background-color: rgb(255, 255, 255); text-decoration: none; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif; font-weight: bold;"><span class="NormalTextRun SCXW254497905 BCX0" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text;">test</span></span><span class="EOP SCXW254497905 BCX0" data-ccp-props="{&quot;335559685&quot;:0}" style="-webkit-user-drag: none; -webkit-tap-highlight-color: transparent; margin: 0px; padding: 0px; user-select: text; color: rgb(0, 0, 0); font-style: normal; font-variant-ligatures: normal; font-variant-caps: normal; font-weight: 400; letter-spacing: normal; orphans: 2; text-align: left; text-indent: 0px; text-transform: none; widows: 2; word-spacing: 0px; -webkit-text-stroke-width: 0px; white-space: pre-wrap; background-color: rgb(255, 255, 255); text-decoration-thickness: initial; text-decoration-style: initial; text-decoration-color: initial; font-size: 12pt; line-height: 22.0875px; font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, sans-serif;"> </span>"#;

// ...existing code...

#[cfg(test)]
mod test {
    use super::*;
    use crate::dom::html_source::HtmlSource;
    use crate::tests::testutils_composer_model::cm;

    #[test]
    fn test_replace_html_strips_meta_tags_google_docs() {
        let mut model = cm("|");

        let _ = model.replace_html(
            GOOGLE_DOC_HTML_PASTEBOARD.into(),
            HtmlSource::GoogleDoc,
        );

        // Verify the HTML doesn't contain meta or the outer b tag
        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert!(!html_str.contains("<meta"));
        assert!(!html_str.contains("docs-internal-guid"));
    }

    #[test]
    fn test_replace_html_strips_only_meta_tags_ms_docs() {
        let mut model = cm("|");

        let _ = model.replace_html(
            MS_DOC_HTML_PASTEBOARD.into(),
            HtmlSource::UnknownExternal,
        );

        let html = model.get_content_as_html();
        let html_str = html.to_string();
        assert!(!html_str.contains("<meta"));
        // Should still contain span elements (though they may be processed/simplified)
        assert!(html_str.contains("test"));
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
}
